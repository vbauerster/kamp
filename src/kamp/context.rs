use super::kak;
use super::{Error, Result};
use std::io::{Cursor, prelude::*};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread;

const END_TOKEN: &str = "<<EEND>>";

#[derive(Debug, Clone)]
pub(crate) struct Context {
    fifo_out: Arc<Path>,
    fifo_err: Arc<Path>,
    session: &'static str,
    client: Option<Rc<Box<str>>>,
    debug: bool,
}

impl From<&str> for Context {
    fn from(s: &str) -> Self {
        let s = String::from(s);
        Context::new(Box::leak(s.into_boxed_str()), false)
    }
}

impl Context {
    pub fn new(session: &'static str, debug: bool) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("kamp-{session}"));

        Context {
            fifo_out: path.with_extension("out").into(),
            fifo_err: path.with_extension("err").into(),
            session,
            client: None,
            debug,
        }
    }

    pub fn set_client(&mut self, client: Option<Rc<Box<str>>>) {
        self.client = client;
    }

    pub fn client(&self) -> Option<Rc<Box<str>>> {
        self.client.clone()
    }

    pub fn session(&self) -> &'static str {
        self.session
    }

    pub fn is_draft(&self) -> bool {
        self.client.is_none()
    }

    pub fn dispatch<T, W>(self, dispatcher: T, writer: W) -> Result<()>
    where
        T: super::Dispatcher,
        W: std::io::Write,
    {
        dispatcher.dispatch(self, writer)
    }

    pub fn send_kill(self, exit_status: Option<i32>) -> Result<()> {
        let mut cmd = String::from("kill");
        if let Some(status) = exit_status {
            cmd.push(' ');
            cmd.push_str(&status.to_string());
        }

        kak::pipe(self.session, cmd)
            .map_err(From::from)
            .and_then(|status| self.check_status(status))
    }

    pub fn send(&self, body: impl AsRef<str>, buffer_ctx: Option<(String, i32)>) -> Result<String> {
        let body = body.as_ref();
        let mut buf = Cursor::new(Vec::with_capacity(512));
        writeln!(buf, "try %🐪")?;
        match (buffer_ctx, self.client()) {
            (Some((b, n)), _) => {
                writeln!(buf, "eval -buffer {b} %🐫")?;
                writeln!(buf, "{body}")?;
                if n != 1 {
                    writeln!(buf, "echo -end-of-line -to-file %opt<kamp_out>")?;
                }
                writeln!(buf, "🐫")?;
            }
            (_, Some(c)) => {
                writeln!(buf, "eval -client {c} %🐫")?;
                writeln!(buf, "{body}")?;
                writeln!(buf, "🐫")?;
            }
            _ => {
                // 'get val client_list' for example need neither buffer nor client
                writeln!(buf, "{body}")?;
            }
        }
        writeln!(buf, "echo -to-file %opt<kamp_out> {END_TOKEN}")?;
        writeln!(buf, "🐪 catch %{{")?;
        writeln!(buf, "echo -debug kamp: %val<error>")?;
        writeln!(buf, "echo -to-file %opt<kamp_err> %val<error>")?;
        writeln!(buf, "}}")?;

        let cmd = String::from_utf8(buf.into_inner())?;
        if self.debug {
            eprintln!("send: {self:#?}");
            eprintln!("{cmd}");
            eprintln!("cmd.len: {}", cmd.len());
        }
        let (tx, rx) = sync_channel(0);
        let err_h = self.read_fifo_err(tx.clone());
        let out_h = self.read_fifo_out(tx);

        kak::pipe(self.session, cmd)
            .map_err(From::from)
            .and_then(|status| self.check_status(status))?;

        match rx.recv().map_err(anyhow::Error::new)? {
            Err(e) => {
                err_h.join().unwrap()?;
                Err(e)
            }
            Ok(s) => {
                out_h.join().unwrap()?;
                Ok(s)
            }
        }
    }

    pub fn connect(self, body: impl AsRef<str>) -> Result<()> {
        let body = body.as_ref();
        let mut buf = Cursor::new(Vec::with_capacity(512));

        if body.is_empty() {
            write!(buf, "echo -to-file %opt<kamp_out> {END_TOKEN}")?;
        } else {
            writeln!(buf, "try %🐪")?;
            writeln!(buf, "eval %🐫")?;
            writeln!(buf, "{body}")?;
            writeln!(buf, "🐫")?;
            writeln!(buf, "echo -to-file %opt<kamp_out> {END_TOKEN}")?;
            writeln!(buf, "🐪 catch %{{")?;
            writeln!(buf, "echo -debug kamp: %val<error>")?;
            writeln!(buf, "echo -to-file %opt<kamp_err> %val<error>")?;
            writeln!(buf, "quit")?;
            writeln!(buf, "}}")?;
        }

        let cmd = String::from_utf8(buf.into_inner())?;
        if self.debug {
            eprintln!("connect: {self:#?}");
            eprintln!("{cmd}");
            eprintln!("cmd.len: {}", cmd.len());
        }
        let (tx, rx) = sync_channel(1);
        let err_h = self.read_fifo_err(tx.clone());
        let out_h = self.read_fifo_out(tx);

        let err_path = self.fifo_err.clone();
        let handle = thread::spawn(move || {
            match rx.recv().map_err(anyhow::Error::new)? {
                Err(kak_err) => err_h
                    .join()
                    .unwrap()
                    .map_err(From::from)
                    .and_then(|_| Err(kak_err)),
                Ok(_) => {
                    // need to write to err pipe in order to complete its read thread
                    // send on read_fifo_err side is going to be non blocking because of channel's bound = 1
                    out_h.join().unwrap().map_err(From::from).and_then(|_| {
                        std::fs::OpenOptions::new()
                            .write(true)
                            .open(err_path)
                            .and_then(|mut f| f.write_all(b"\n"))
                            .map_err(From::from)
                    })
                }
            }
        });

        let status = kak::connect(self.session, cmd)?;
        match (self.check_status(status), handle.join().unwrap()) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }

    pub fn query_kak(
        &self,
        query_ctx: impl Into<super::cmd::QueryContext>,
        buffer_ctx: Option<(String, i32)>,
    ) -> Result<Vec<String>> {
        let ctx = query_ctx.into();
        let mut buf = Cursor::new(Vec::with_capacity(64));
        write!(
            buf,
            "echo -quoting {} -to-file %opt<kamp_out> {}",
            ctx.quoting, ctx.key_val
        )?;
        let body = String::from_utf8(buf.into_inner())?;
        self.send(body, buffer_ctx).map(|output| {
            let split_by = ctx.output_delimiter();
            if ctx.verbatim {
                output.split(split_by).map(String::from).collect()
            } else {
                output.split(split_by).map(unquote_kakoune_string).collect()
            }
        })
    }

    fn check_status(&self, status: std::process::ExitStatus) -> Result<()> {
        if status.success() {
            return Ok(());
        }
        Err(match status.code() {
            Some(code) => Error::InvalidSession {
                session: self.session,
                exit_code: code,
            },
            None => anyhow::Error::msg("kak terminated by signal").into(),
        })
    }

    fn read_fifo_err(
        &self,
        send_ch: SyncSender<Result<String>>,
    ) -> thread::JoinHandle<anyhow::Result<()>> {
        let path = self.fifo_err.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            std::fs::OpenOptions::new()
                .read(true)
                .open(path)
                .and_then(|mut f| f.read_to_string(&mut buf))?;
            send_ch
                .send(Err(Error::KakEvalCatch(buf)))
                .map_err(anyhow::Error::new)
        })
    }

    fn read_fifo_out(
        &self,
        send_ch: SyncSender<Result<String>>,
    ) -> thread::JoinHandle<anyhow::Result<()>> {
        let path = self.fifo_out.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            let mut f = std::fs::OpenOptions::new().read(true).open(path)?;
            // END_TOKEN comes appended to the payload
            let res = loop {
                f.read_to_string(&mut buf)?;
                if buf.ends_with(END_TOKEN) {
                    break buf.trim_end_matches(END_TOKEN);
                }
            };
            send_ch.send(Ok(res.into())).map_err(anyhow::Error::new)
        })
    }
}

fn unquote_kakoune_string(input: &str) -> String {
    let mut buf = String::new();
    let mut state_is_open = false;
    let mut iter = input.chars().peekable();
    loop {
        match (iter.next(), state_is_open) {
            (Some('\''), false) => state_is_open = true,
            (Some('\''), true) => {
                if let Some('\'') = iter.peek() {
                    buf.push('\'');
                }
                state_is_open = false;
            }
            (Some(c), _) => buf.push(c),
            (None, _) => return buf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_kak_style_quoting() {
        let test = vec!["'''a'''", "'b'", "'echo pop'", r#"'echo "''ok''"'"#];
        let expected = vec!["'a'", "b", "echo pop", r#"echo "'ok'""#]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let map = test.into_iter().zip(expected).collect::<HashMap<_, _>>();
        for (test, expected) in map {
            assert_eq!(unquote_kakoune_string(test), expected);
        }
    }
}

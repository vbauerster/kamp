use super::cmd::{QueryContext, QueryType};
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
    fifo_out: Arc<Box<Path>>,
    fifo_err: Arc<Box<Path>>,
    session: Rc<Box<str>>,
    client: Option<Rc<Box<str>>>,
    debug: bool,
}

impl Context {
    pub fn new<S: AsRef<str>>(session: S, debug: bool) -> Self {
        let session = session.as_ref();
        let mut out = std::env::temp_dir();
        out.push(format!("kamp-{session}"));
        let mut err = out.clone();
        out.set_extension("out");
        err.set_extension("err");

        Context {
            fifo_out: Arc::new(out.into_boxed_path()),
            fifo_err: Arc::new(err.into_boxed_path()),
            session: Rc::new(session.into()),
            client: None,
            debug,
        }
    }

    pub fn set_client<S: AsRef<str>>(&mut self, client: S) {
        let client = client.as_ref();
        if client.is_empty() {
            self.client = None;
        } else {
            self.client = Some(Rc::new(client.into()));
        }
    }

    pub fn client(&self) -> Option<Rc<Box<str>>> {
        self.client.clone()
    }

    pub fn session(&self) -> Rc<Box<str>> {
        self.session.clone()
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

        kak::pipe(self.session.as_ref(), cmd)
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
        write!(buf, "}}")?;

        let cmd = String::from_utf8(buf.into_inner())?;
        if self.debug {
            dbg!(self);
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
                Ok(s) => {
                    out_h
                        .join()
                        .unwrap()
                        .map_err(From::from)
                        .and_then(|_| {
                            // need to write to err pipe in order to complete its read thread
                            // send to tx on read_fifo_err side is going to be non blocking
                            // because of buffered sync_channel (bound = 1)
                            std::fs::OpenOptions::new()
                                .write(true)
                                .open(err_path.as_ref())
                                .and_then(|mut f| f.write_all(b"\n"))
                                .map_err(From::from)
                        })
                        .map(|_| s)
                }
            }
        });

        let status = kak::pipe(self.session.as_ref(), cmd.as_bytes())?;
        match (self.check_status(status), handle.join().unwrap()) {
            (Ok(_), Ok(s)) => Ok(s),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }

    pub fn connect(&self, body: impl AsRef<str>) -> Result<()> {
        let body = body.as_ref();
        let mut buf = Cursor::new(Vec::with_capacity(512));

        if body.is_empty() {
            write!(buf, "echo -to-file %opt<kamp_out> {END_TOKEN}")?;
        } else {
            writeln!(buf, "try %🐪")?;
            writeln!(buf, "{body}")?;
            writeln!(buf, "echo -to-file %opt<kamp_out> {END_TOKEN}")?;
            writeln!(buf, "🐪 catch %{{")?;
            writeln!(buf, "echo -debug kamp: %val<error>")?;
            writeln!(buf, "echo -to-file %opt<kamp_err> %val<error>")?;
            writeln!(buf, "quit")?;
            write!(buf, "}}")?;
        }

        let cmd = String::from_utf8(buf.into_inner())?;
        if self.debug {
            dbg!(self);
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
                    out_h.join().unwrap().map_err(From::from).and_then(|_| {
                        // need to write to err pipe in order to complete its read thread
                        // send to tx on read_fifo_err side is going to be non blocking
                        // because of buffered sync_channel (bound = 1)
                        std::fs::OpenOptions::new()
                            .write(true)
                            .open(err_path.as_ref())
                            .and_then(|mut f| f.write_all(b"\n"))
                            .map_err(From::from)
                    })
                }
            }
        });

        let status = kak::connect(self.session.as_ref(), &cmd.into_boxed_str())?;
        match (self.check_status(status), handle.join().unwrap()) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }

    pub fn query_kak(
        &self,
        query_ctx: impl Into<QueryContext>,
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
        self.send(body, buffer_ctx)
            .inspect(|raw_output| {
                if self.debug {
                    dbg!(raw_output);
                }
            })
            .map(|output| match ctx.qtype {
                QueryType::List => split_kak_list(&output),
                QueryType::Map(lookup) => {
                    let mut res = Vec::new();
                    for item in split_kak_list(&output) {
                        if let Some((key, val)) = item.split_once('=')
                            && key == lookup
                        {
                            res.push(String::from(val));
                            break;
                        }
                    }
                    res
                }
                _ if !ctx.verbatim => output.split('\n').map(String::from).collect(),
                _ => vec![output],
            })
            .inspect(|parsed_output| {
                if self.debug {
                    dbg!(parsed_output);
                }
            })
    }

    fn check_status(&self, status: std::process::ExitStatus) -> Result<()> {
        if status.success() {
            return Ok(());
        }
        Err(match status.code() {
            Some(code) => Error::KakUnexpectedExit(code),
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
                .open(path.as_ref())
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
            let mut f = std::fs::OpenOptions::new().read(true).open(path.as_ref())?;
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

fn split_kak_list(input: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut open = false;
    let mut iter = input.chars().peekable();
    while let Some(c) = iter.next() {
        match c {
            '\'' => {
                if open {
                    if let Some('\'') = iter.peek() {
                        buf.push(c);
                    } else {
                        res.push(buf);
                        buf = String::new();
                    }
                    open = false;
                } else {
                    open = true;
                }
            }
            _ => {
                if open {
                    buf.push(c);
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_kak_list_1() {
        let input = r#"'''a''' '''a' 'b' 'echo pop' 'echo "''ok''"'"#;
        let expected = vec!["'a'", "'a", "b", "echo pop", r#"echo "'ok'""#]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(split_kak_list(input), expected);
    }

    #[test]
    fn test_split_kak_list_2() {
        let input = r#"'enter-user-mode pokemon' 'edit ''sp buf.txt'''"#;
        let expected = vec!["enter-user-mode pokemon", "edit 'sp buf.txt'"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(split_kak_list(input), expected);
    }
}

use super::kak;
use super::{Error, Result};
use crossbeam_channel::Sender;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

const END_TOKEN: &str = "<<EEND>>";

enum QuotingStyle {
    Raw,
    Kakoune,
}

enum ParseType {
    None(QuotingStyle),
    Kakoune,
}

impl ParseType {
    fn new(quote: bool, split: bool) -> Self {
        match (quote, split) {
            (true, _) => ParseType::None(QuotingStyle::Kakoune),
            (_, false) => ParseType::None(QuotingStyle::Raw),
            _ => ParseType::Kakoune,
        }
    }
    fn quoting(&self) -> &'static str {
        match self {
            ParseType::None(QuotingStyle::Raw) => "raw",
            _ => "kakoune",
        }
    }
    fn parse(&self, s: String) -> Vec<String> {
        match self {
            ParseType::Kakoune => parse_kak_style_quoting(&s),
            _ => vec![s],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Context {
    session: &'static str,
    client: Option<Rc<str>>,
    fifo_out: Arc<Path>,
    fifo_err: Arc<Path>,
}

impl From<&str> for Context {
    fn from(s: &str) -> Self {
        let s = String::from(s);
        Context::new(Box::leak(s.into_boxed_str()), None)
    }
}

impl Context {
    pub fn new(session: &'static str, client: Option<String>) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("kamp-{session}"));

        Context {
            session,
            client: client.map(|s| s.into()),
            fifo_out: path.with_extension("out").into(),
            fifo_err: path.with_extension("err").into(),
        }
    }

    pub fn set_client(&mut self, client: String) {
        self.client = Some(Rc::from(client.into_boxed_str()));
    }

    pub fn unset_client(&mut self) {
        self.client = None;
    }

    pub fn client(&self) -> Option<Rc<str>> {
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
            .map_err(|err| err.into())
            .and_then(|status| self.check_status(status))
    }

    pub fn send(self, buffer_ctx: Option<(String, i32)>, body: impl AsRef<str>) -> Result<String> {
        let mut body = Cow::from(body.as_ref());
        let mut cmd = String::from("try %{\n");
        cmd.push_str("eval");
        match (buffer_ctx, self.client()) {
            (Some((b, n)), _) => {
                cmd.push_str(" -buffer ");
                cmd.push_str(&b);
                if n == 0 || n > 1 {
                    body.to_mut().push_str("\necho -to-file %opt{kamp_out} ' '");
                }
            }
            (_, Some(c)) => {
                cmd.push_str(" -client ");
                cmd.push_str(&c);
            }
            _ => (), // 'get val client_list' for example need neither buffer nor client
        }
        cmd.push_str(" %{\n");
        cmd.push_str(&body);
        cmd.push_str("\n}\n");
        write_end_token(&mut cmd);
        cmd.push_str("} catch %{\n");
        cmd.push_str("echo -debug kamp: %val{error};");
        cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n}");

        let (s, r) = crossbeam_channel::bounded(0);
        let err_h = self.read_fifo_err(s.clone());
        let out_h = self.read_fifo_out(s);

        let status = kak::pipe(self.session, cmd)?;
        self.check_status(status)?;

        match r.recv().map_err(anyhow::Error::new)? {
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
        let mut cmd = String::new();
        let body = body.as_ref();
        if !body.is_empty() {
            cmd.push_str("try %{\neval -try-client '' %{\n");
            cmd.push_str(body);
            cmd.push_str("\n}\n");
            write_end_token(&mut cmd);
            cmd.push_str("} catch %{\n");
            cmd.push_str("echo -debug kamp: %val{error};");
            cmd.push_str("echo -to-file %opt{kamp_err} %val{error};");
            cmd.push_str("quit 1\n}");
        } else {
            write_end_token(&mut cmd);
            cmd.pop();
        }

        let (s, r) = crossbeam_channel::bounded(1);
        let err_h = self.read_fifo_err(s.clone());
        let out_h = self.read_fifo_out(s);

        let kak_h = thread::spawn(move || kak::connect(self.session, cmd));

        let res = match r.recv() {
            Err(recv_err) => {
                let status = kak_h.join().unwrap()?;
                return self
                    .check_status(status)
                    .and_then(|_| Err(anyhow::Error::new(recv_err).into()));
            }
            Ok(res) => res,
        };
        if res.is_ok() {
            // need to write to err pipe in order to complete its read thread
            std::fs::OpenOptions::new()
                .write(true)
                .open(&self.fifo_err)
                .and_then(|mut f| f.write_all(b""))?;
            out_h.join().unwrap()?;
        }
        err_h.join().unwrap()?;
        kak_h.join().unwrap()?;
        res.map(drop)
    }

    pub fn query_val(
        self,
        buffer_ctx: Option<(String, i32)>,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
    ) -> Result<Vec<String>> {
        self.query_kak(buffer_ctx, ("val", name.as_ref()), quote, split)
    }

    pub fn query_opt(
        self,
        buffer_ctx: Option<(String, i32)>,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
    ) -> Result<Vec<String>> {
        self.query_kak(buffer_ctx, ("opt", name.as_ref()), quote, split)
    }

    pub fn query_reg(
        self,
        buffer_ctx: Option<(String, i32)>,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
    ) -> Result<Vec<String>> {
        self.query_kak(buffer_ctx, ("reg", name.as_ref()), quote, split)
    }

    pub fn query_sh(
        self,
        buffer_ctx: Option<(String, i32)>,
        cmd: impl AsRef<str>,
    ) -> Result<Vec<String>> {
        self.query_kak(buffer_ctx, ("sh", cmd.as_ref()), false, false)
    }

    fn query_kak(
        self,
        buffer_ctx: Option<(String, i32)>,
        (key, val): (&str, &str),
        quote: bool,
        split: bool,
    ) -> Result<Vec<String>> {
        let parse_type = ParseType::new(quote, split);
        let mut buf = String::from("echo -quoting ");
        buf.push_str(parse_type.quoting());
        buf.push_str(" -to-file %opt{kamp_out} %");
        buf.push_str(key);
        buf.push('{');
        buf.push_str(val);
        buf.push('}');
        self.send(buffer_ctx, &buf).map(|s| parse_type.parse(s))
    }

    fn check_status(self, status: std::process::ExitStatus) -> Result<()> {
        if status.success() {
            return Ok(());
        }
        Err(match status.code() {
            Some(code) => Error::InvalidSession {
                session: self.session,
                exit_code: code,
            },
            None => anyhow::anyhow!("kak terminated by signal").into(),
        })
    }

    fn read_fifo_err(
        &self,
        send_ch: Sender<Result<String>>,
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
        send_ch: Sender<Result<String>>,
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

fn write_end_token(buf: &mut String) {
    buf.push_str("echo -to-file %opt{kamp_out} ");
    buf.push_str(END_TOKEN);
    buf.push('\n');
}

fn parse_kak_style_quoting(input: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut state_is_open = false;
    let mut iter = input.chars().peekable();
    loop {
        match iter.next() {
            Some('\'') => {
                if state_is_open {
                    if let Some('\'') = iter.peek() {
                        buf.push('\'');
                    } else {
                        res.push(buf);
                        buf = String::new();
                    }
                    state_is_open = false;
                } else {
                    state_is_open = true;
                }
            }
            Some(c) => {
                if state_is_open {
                    buf.push(c)
                }
            }
            None => return res,
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

        assert_eq!(parse_kak_style_quoting(&test.join(" ")), expected);

        let map = test.into_iter().zip(expected).collect::<HashMap<_, _>>();
        for (test, expected) in map {
            assert_eq!(parse_kak_style_quoting(test), vec![expected]);
        }
    }
}

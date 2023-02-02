use super::kak;
use super::{Error, Result};
use crossbeam_channel::Sender;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
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
            (true, _) => ParseType::none_quote_kak(),
            (_, false) => ParseType::none_quote_raw(),
            _ => ParseType::Kakoune,
        }
    }
    fn none_quote_raw() -> Self {
        ParseType::None(QuotingStyle::Raw)
    }
    fn none_quote_kak() -> Self {
        ParseType::None(QuotingStyle::Kakoune)
    }
    fn quoting(&self) -> &'static str {
        match self {
            ParseType::None(QuotingStyle::Raw) => "raw",
            _ => "kakoune",
        }
    }
    fn parse(&self, s: String) -> Vec<String> {
        match self {
            ParseType::Kakoune => parse_kak_style_quoting(&mut s.chars().peekable()),
            _ => vec![s],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Context<'a> {
    session: Cow<'a, str>,
    client: Option<Cow<'a, str>>,
    base_path: Rc<PathBuf>,
}

impl<'a> Context<'a> {
    pub fn new<S>(session: S, client: Option<S>) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let session = session.into();
        let mut path = std::env::temp_dir();
        path.push(format!("kamp-{session}"));

        Context {
            session,
            client: client.map(Into::into),
            base_path: Rc::new(path),
        }
    }

    pub fn set_client(&mut self, client: impl Into<Cow<'a, str>>) {
        let client = client.into();
        self.client = if client.is_empty() {
            None
        } else {
            Some(client)
        };
    }

    pub fn session(&self) -> Cow<'a, str> {
        self.session.clone()
    }

    pub fn is_draft(&self) -> bool {
        self.client.is_none()
    }

    pub fn send_kill(self, exit_status: Option<i32>) -> Result<()> {
        let mut cmd = String::from("kill");
        if let Some(status) = exit_status {
            cmd.push(' ');
            cmd.push_str(&status.to_string());
        }

        let status = kak::pipe(&self.session, cmd)?;
        self.check_status(status)
    }

    pub fn send(&self, body: impl AsRef<str>, buffer_ctx: Option<(String, i32)>) -> Result<String> {
        let mut body = Cow::from(body.as_ref());
        let mut cmd = String::from("try %{\n");
        cmd.push_str("eval");
        match (&buffer_ctx, &self.client) {
            (Some((b, n)), _) => {
                cmd.push_str(" -buffer ");
                cmd.push_str(b);
                if *n == 0 || *n > 1 {
                    body.to_mut().push_str(";echo -to-file %opt{kamp_out} ' '");
                }
            }
            (_, Some(c)) => {
                cmd.push_str(" -client ");
                cmd.push_str(c);
            }
            _ => (), // 'get val client_list' for example doesn't need neither buffer nor client
        }
        cmd.push_str(" %{\n");
        cmd.push_str(&body);
        cmd.push_str("\n}\n");
        write_end_token(&mut cmd);
        cmd.push_str("} catch %{\n");
        cmd.push_str("echo -debug kamp: %val{error};");
        cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n}");

        let (s0, r) = crossbeam_channel::bounded(0);
        let s1 = s0.clone();
        let out_h = read_out(self.get_out_path(false), s0);
        let err_h = read_err(self.get_out_path(true), s1);

        let status = kak::pipe(&self.session, cmd)?;
        self.check_status(status)?;

        let res = r.recv().map_err(anyhow::Error::new)?;
        let handle = if res.is_ok() { out_h } else { err_h };
        handle.join().unwrap()?;
        res
    }

    pub fn connect(&self, body: impl AsRef<str>) -> Result<()> {
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

        let kak_h = thread::spawn({
            let session = self.session().into_owned();
            move || kak::connect(session, cmd)
        });

        let (s0, r) = crossbeam_channel::bounded(1);
        let s1 = s0.clone();
        let out_h = read_out(self.get_out_path(false), s0);
        let err_h = read_err(self.get_out_path(true), s1);

        let res = match r.recv() {
            Ok(res) => res,
            Err(recv_err) => {
                let status = kak_h.join().unwrap()?;
                return self
                    .check_status(status)
                    .and_then(|_| Err(anyhow::Error::new(recv_err).into()));
            }
        };
        if res.is_ok() {
            std::fs::OpenOptions::new()
                .write(true)
                .open(self.get_out_path(true))
                .and_then(|mut f| f.write_all(b""))?;
            out_h.join().unwrap()?;
        }
        err_h.join().unwrap()?;
        kak_h.join().unwrap()?;
        res.map(drop)
    }

    pub fn query_val(
        &self,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
        buffer_ctx: Option<(String, i32)>,
    ) -> Result<Vec<String>> {
        self.query_kak(("val", name.as_ref()), quote, split, buffer_ctx)
    }

    pub fn query_opt(
        &self,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
        buffer_ctx: Option<(String, i32)>,
    ) -> Result<Vec<String>> {
        self.query_kak(("opt", name.as_ref()), quote, split, buffer_ctx)
    }

    pub fn query_reg(
        &self,
        name: impl AsRef<str>,
        quote: bool,
        split: bool,
    ) -> Result<Vec<String>> {
        self.query_kak(("reg", name.as_ref()), quote, split, None)
    }

    pub fn query_sh(
        &self,
        cmd: impl AsRef<str>,
        buffer_ctx: Option<(String, i32)>,
    ) -> Result<Vec<String>> {
        self.query_kak(("sh", cmd.as_ref()), false, false, buffer_ctx)
    }
}

impl<'a> Context<'a> {
    fn query_kak(
        &self,
        (key, val): (&str, &str),
        quote: bool,
        split: bool,
        buffer_ctx: Option<(String, i32)>,
    ) -> Result<Vec<String>> {
        let parse_type = ParseType::new(quote, split);
        let mut buf = String::from("echo -quoting ");
        buf.push_str(parse_type.quoting());
        buf.push_str(" -to-file %opt{kamp_out} %");
        buf.push_str(key);
        buf.push('{');
        buf.push_str(val);
        buf.push('}');
        self.send(&buf, buffer_ctx).map(|s| parse_type.parse(s))
    }

    fn get_out_path(&self, err_out: bool) -> PathBuf {
        if err_out {
            self.base_path.with_extension("err")
        } else {
            self.base_path.with_extension("out")
        }
    }

    fn check_status(&self, status: std::process::ExitStatus) -> Result<()> {
        if status.success() {
            return Ok(());
        }
        let err = match status.code() {
            Some(code) => Error::InvalidSession {
                session: self.session().into_owned(),
                exit_code: code,
            },
            None => anyhow::Error::msg("kak terminated by signal").into(),
        };
        Err(err)
    }
}

fn read_err(
    file_path: PathBuf,
    send_ch: Sender<Result<String>>,
) -> thread::JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || {
        let mut buf = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .and_then(|mut f| f.read_to_string(&mut buf))?;
        send_ch
            .send(Err(Error::KakEvalCatch(buf)))
            .map_err(anyhow::Error::new)
    })
}

fn read_out(
    file_path: PathBuf,
    send_ch: Sender<Result<String>>,
) -> thread::JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || {
        let mut buf = String::new();
        let mut f = std::fs::OpenOptions::new().read(true).open(&file_path)?;
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

fn write_end_token(buf: &mut String) {
    buf.push_str("echo -to-file %opt{kamp_out} ");
    buf.push_str(END_TOKEN);
    buf.push('\n');
}

fn parse_kak_style_quoting<I>(tokens: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = char>,
{
    enum State {
        Open,
        Close,
    }
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut state = State::Close;
    loop {
        match tokens.next() {
            Some('\'') => match state {
                State::Open => {
                    if let Some('\'') = tokens.peek() {
                        buf.push('\'');
                    } else {
                        res.push(buf);
                        buf = String::new();
                    }
                    state = State::Close;
                }
                State::Close => {
                    state = State::Open;
                }
            },
            Some(c) => {
                if let State::Open = state {
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

        let test_joined = test.join(" ");
        assert_eq!(
            parse_kak_style_quoting(&mut test_joined.chars().peekable()),
            expected
        );

        let map = test.into_iter().zip(expected).collect::<HashMap<_, _>>();
        for (test, expected) in map {
            assert_eq!(
                parse_kak_style_quoting(&mut test.chars().peekable()),
                vec![expected]
            );
        }
    }
}

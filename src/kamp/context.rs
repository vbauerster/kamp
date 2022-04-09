use crossbeam_channel::Sender;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use super::kak;
use super::Error;

const END_TOKEN: &str = "<<EEND>>";

#[derive(Debug)]
pub(crate) struct Context<'a> {
    session: Cow<'a, str>,
    client: Option<&'a str>,
    path: Rc<PathBuf>,
}

impl std::fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session: {}", self.session)?;
        if let Some(client) = &self.client {
            write!(f, "\nclient: {}", client)?;
        }
        Ok(())
    }
}

impl<'a> Context<'a> {
    pub fn new(session: impl Into<Cow<'a, str>>, client: Option<&'a str>) -> Self {
        let session = session.into();
        let mut path = std::env::temp_dir();
        let mut base = String::from("kamp-");
        base.push_str(&session);
        path.push(base);

        Context {
            session,
            client: client.filter(|&client| !client.is_empty()),
            path: Rc::new(path),
        }
    }

    pub fn clone_with_client(&self, client: Option<&'a str>) -> Self {
        Context {
            session: self.session.clone(),
            client: client.filter(|&client| !client.is_empty()),
            path: Rc::clone(&self.path),
        }
    }

    pub fn session(&self) -> String {
        self.session.clone().into_owned()
    }

    pub fn session_as_ref(&self) -> &str {
        self.session.as_ref()
    }

    pub fn is_draft(&self) -> bool {
        self.client.is_none()
    }

    pub fn send(&self, body: &str, buffer: Option<String>) -> Result<String, Error> {
        self.send_kill(body, false, buffer)
    }

    pub fn send_kill(
        &self,
        body: &str,
        kill: bool,
        buffer: Option<String>,
    ) -> Result<String, Error> {
        let mut buffer_is_star = false;
        let eval_ctx = match (buffer.as_deref(), self.client) {
            (Some(b), _) => {
                buffer_is_star = b == "*";
                Some((" -buffer ", b))
            }
            (_, Some(c)) => Some((" -client ", c)),
            // 'get val client_list' for example doesn't need neither buffer nor client
            (None, None) => None,
        };
        let mut cmd = String::new();
        cmd.push_str("try %{ eval");
        if let Some((ctx, name)) = eval_ctx {
            cmd.push_str(ctx);
            cmd.push_str(name);
        }
        cmd.push_str(" %{\n");
        if kill {
            // allow kamp to exit early, because after kill commands aren't executed
            if let Some(c) = cmd.pop() {
                write_end_token(&mut cmd);
                cmd.push(c);
            }
        }
        cmd.push_str(body);
        if !buffer_is_star {
            write_end_token(&mut cmd);
        }
        cmd.push_str("\n}} catch %{");
        cmd.push_str("\necho -debug kamp: %val{error}");
        cmd.push_str("\necho -to-file %opt{kamp_err} %val{error}");
        cmd.push_str("\n}");
        if buffer_is_star {
            // writing END_TOKEN after try, because of '-buffer *'
            // workaround for https://github.com/mawww/kakoune/issues/4586
            write_end_token(&mut cmd);
        }

        let kak_h = thread::spawn({
            let session = self.session();
            move || kak::pipe(session, cmd)
        });

        let (s0, r) = crossbeam_channel::bounded(1);
        let s1 = s0.clone();
        let out_h = read_out(self.get_out_path(false), s0);
        let err_h = read_err(self.get_out_path(true), s1);

        let res = match r.recv() {
            Ok(res) => res,
            Err(recv_err) => {
                let status = kak_h.join().unwrap()?;
                let err = match status.code() {
                    Some(code) => Error::InvalidSession {
                        session: self.session(),
                        exit_code: code,
                    },
                    None => Error::Other(anyhow::Error::new(recv_err)),
                };
                return Err(err);
            }
        };
        if res.is_ok() {
            out_h.join().unwrap()?;
        } else {
            err_h.join().unwrap()?;
        }
        kak_h.join().unwrap()?;
        res
    }

    pub fn connect(&self, body: &str) -> Result<(), Error> {
        let mut cmd = String::new();
        if !body.is_empty() {
            cmd.push_str("try %{ eval -try-client '' %{\n");
            cmd.push_str(body);
            write_end_token(&mut cmd);
            cmd.push_str("\n}} catch %{");
            cmd.push_str("\necho -to-file %opt{kamp_err} %val{error}");
            cmd.push_str("\necho -debug kamp: %val{error}");
            cmd.push_str("\nquit 1");
            cmd.push_str("\n}");
        } else {
            write_end_token(&mut cmd);
        }

        let kak_h = thread::spawn({
            let session = self.session();
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
                let err = match status.code() {
                    Some(code) => Error::InvalidSession {
                        session: self.session(),
                        exit_code: code,
                    },
                    None => Error::Other(anyhow::Error::new(recv_err)),
                };
                return Err(err);
            }
        };
        if res.is_ok() {
            std::fs::OpenOptions::new()
                .write(true)
                .open(self.get_out_path(true))
                .and_then(|mut f| f.write_all(b""))?;
        }
        out_h.join().unwrap()?;
        err_h.join().unwrap()?;
        kak_h.join().unwrap()?;
        res.map(|_| ())
    }
}

impl Context<'_> {
    fn get_out_path(&self, err_out: bool) -> PathBuf {
        if err_out {
            self.path.with_extension("err")
        } else {
            self.path.with_extension("out")
        }
    }
}

fn read_err(
    file_path: PathBuf,
    send_ch: Sender<Result<String, Error>>,
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
    send_ch: Sender<Result<String, Error>>,
) -> thread::JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || {
        let mut buf = String::new();
        let mut f = std::fs::OpenOptions::new().read(true).open(&file_path)?;
        // there is no guarantee that END_TOKEN comes separately as a single
        // token. It may come appended or not, that's why reading everything
        // into single buf
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
    buf.push_str("\necho -to-file %opt{kamp_out} ");
    buf.push_str(END_TOKEN);
}

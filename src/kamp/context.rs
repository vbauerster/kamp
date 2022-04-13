use crossbeam_channel::Sender;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use super::kak;
use super::Error;

const END_TOKEN: &str = "<<EEND>>";

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    name: String,
    pwd: String,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: String, pwd: String, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    name: String,
    bufname: String,
}

impl Client {
    fn new(name: String, bufname: String) -> Self {
        Client { name, bufname }
    }
}

#[derive(Debug)]
pub(crate) struct Context<'a> {
    session: Cow<'a, str>,
    client: Option<&'a str>,
    base_path: Rc<PathBuf>,
}

impl std::fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session: {}", self.session)?;
        if let Some(client) = self.client {
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
            base_path: Rc::new(path),
        }
    }

    pub fn session(&self) -> Cow<'a, str> {
        self.session.clone()
    }

    pub fn is_draft(&self) -> bool {
        self.client.is_none()
    }

    pub fn send_kill(&self, exit_status: Option<i32>) -> Result<(), Error> {
        let mut cmd = String::from("kill");
        if let Some(status) = exit_status {
            cmd.push(' ');
            cmd.push_str(&status.to_string());
        }

        let status = kak::pipe(&self.session, cmd)?;
        self.check_status(status)
    }

    pub fn send<S: AsRef<str>>(&self, body: S, buffer: Option<String>) -> Result<String, Error> {
        let eval_ctx = match (buffer.as_deref(), self.client) {
            (Some(b), _) => Some((" -buffer ", b)),
            (_, Some(c)) => Some((" -client ", c)),
            // 'get val client_list' for example doesn't need neither buffer nor client
            (None, None) => None,
        };
        let mut cmd = String::from("try %{\n");
        cmd.push_str("eval");
        if let Some((ctx, name)) = eval_ctx {
            cmd.push_str(ctx);
            cmd.push_str(name);
        }
        cmd.push_str(" %{\n");
        cmd.push_str(body.as_ref());
        cmd.push_str("\n}\n");
        write_end_token(&mut cmd);
        cmd.push_str("} catch %{\n");
        cmd.push_str("echo -debug kamp: %val{error}\n");
        cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n}");

        let (s0, r) = crossbeam_channel::bounded(1);
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

    pub fn connect<S: AsRef<str>>(&self, body: S) -> Result<(), Error> {
        let mut cmd = String::new();
        let body = body.as_ref();
        if !body.is_empty() {
            cmd.push_str("try %{\neval -try-client '' %{\n");
            cmd.push_str(body);
            cmd.push_str("\n}\n");
            write_end_token(&mut cmd);
            cmd.push_str("} catch %{\n");
            cmd.push_str("echo -debug kamp: %val{error}\n");
            cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n");
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
                    .and_then(|_| Err(Error::Other(anyhow::Error::new(recv_err))));
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
        res.map(|_| ())
    }

    pub fn session_struct(&self) -> Result<Session, Error> {
        use super::cmd::Get;
        Get::new_val("client_list")
            .run(self, 0, None)
            .and_then(|clients| {
                clients
                    .lines()
                    .map(|name| {
                        Get::new_val("bufname")
                            .run(&self.clone_with_client(Some(name)), 2, None)
                            .map(|bufname| Client::new(name.into(), bufname))
                    })
                    .collect::<Result<Vec<Client>, Error>>()
            })
            .and_then(|clients| {
                Get::new_sh("pwd")
                    .run(self, 2, None)
                    .map(|pwd| Session::new(self.session().into_owned(), pwd, clients))
            })
    }
}

impl<'a> Context<'a> {
    fn clone_with_client(&self, client: Option<&'a str>) -> Self {
        Context {
            session: self.session.clone(),
            client: client.filter(|&client| !client.is_empty()),
            base_path: Rc::clone(&self.base_path),
        }
    }
    fn get_out_path(&self, err_out: bool) -> PathBuf {
        if err_out {
            self.base_path.with_extension("err")
        } else {
            self.base_path.with_extension("out")
        }
    }
    fn check_status(&self, status: std::process::ExitStatus) -> Result<(), Error> {
        if status.success() {
            return Ok(());
        }
        let err = match status.code() {
            Some(code) => Error::InvalidSession {
                session: self.session().into_owned(),
                exit_code: code,
            },
            None => Error::Other(anyhow::Error::msg("kak terminated by signal")),
        };
        Err(err)
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

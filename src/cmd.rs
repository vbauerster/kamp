// mod ctx;
// pub(super) use ctx::Ctx;

use crossbeam_channel::Sender;
use std::fs::File;
use std::io::prelude::*;
use std::thread;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession(#[from] std::env::VarError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("kak exited with error: {0}")]
    KakProcessFailure(std::process::ExitStatus),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

#[derive(Debug)]
pub(super) struct Context {
    pub session: String,
    pub client: Option<String>,
}

impl Context {
    pub fn new(session: String) -> Self {
        Context {
            session,
            client: None,
        }
    }
    pub fn from_env() -> Result<Self, Error> {
        let mut ctx = std::env::var(KAKOUNE_SESSION).map(Context::new)?;
        ctx.set_client_if_any(std::env::var(KAKOUNE_CLIENT).ok());
        Ok(ctx)
    }
    pub fn set_client_if_any(&mut self, client: Option<String>) {
        if client.is_some() {
            self.client = client;
        }
    }
    pub fn send(&self, body: &str) -> Result<String, Error> {
        let buffer: Option<String> = None;

        let buffer = buffer.as_deref().and_then(|arg| {
            let switch = " -buffer ";
            let mut buf = String::with_capacity(switch.len() + arg.len());
            buf.push_str(switch);
            buf.push_str(arg);
            Some(buf)
        });
        let client = self.client.as_deref().and_then(|arg| {
            let switch = " -try-client ";
            let mut buf = String::with_capacity(switch.len() + arg.len());
            buf.push_str(switch);
            buf.push_str(arg);
            Some(buf)
        });

        let cmd = format!(
            // "try %§ eval{} {} § catch %§ echo -debug %val{{error}} §",
            "try %§ eval{} {} § catch %§ echo -to-file %opt{{kamp_err}} %val{{error}} §",
            buffer.or(client).unwrap_or_default(),
            body
        );

        dbg!(&cmd);
        kak_exec(&self.session, &cmd)?;

        let (s, r) = crossbeam_channel::bounded(0);
        let err_jh = read_output(&self.session, true, s.clone());
        let out_jh = read_output(&self.session, false, s);

        let res = r.recv().map_err(anyhow::Error::new)?;

        let jh = if res.is_err() { err_jh } else { out_jh };
        jh.join()
            .expect("output reader thread halted in unexpected way")?;
        res
    }
}

fn kak_exec<T: AsRef<[u8]>>(session: &str, cmd: T) -> Result<(), Error> {
    use std::process::{Command, Stdio};

    let mut child = Command::new("kak")
        .arg("-p")
        .arg(session)
        .stdin(Stdio::piped())
        .spawn()?;

    let kak_stdin = match child.stdin.as_mut() {
        Some(stdin) => stdin,
        None => {
            use std::io::{Error, ErrorKind};
            Err(Error::new(
                ErrorKind::Other,
                "kak's process stdin has not been captured",
            ))?
        }
    };

    kak_stdin.write_all(cmd.as_ref())?;

    let status = child.wait()?;

    if !status.success() {
        Err(Error::KakProcessFailure(status))?;
    }

    Ok(())
}

fn read_output(
    session: &str,
    is_err: bool,
    send_ch: Sender<Result<String, Error>>,
) -> thread::JoinHandle<Result<(), Error>> {
    let mut path = std::env::temp_dir();
    if is_err {
        path.push(session.to_owned() + "-kamp-err");
    } else {
        path.push(session.to_owned() + "-kamp-out");
    }
    thread::spawn(move || {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        send_ch
            .send(if is_err {
                Err(Error::KakEvalCatch(buf))
            } else {
                Ok(buf)
            })
            .map_err(anyhow::Error::new)?;
        Ok(())
    })
}

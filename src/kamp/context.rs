use crossbeam_channel::Sender;
use std::io::prelude::*;
use std::path::PathBuf;
use std::thread;

use super::error::Error;
use super::kak;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";
const END_TOKEN: &str = "<<EEND>>";

#[derive(Debug)]
pub(crate) struct Context {
    pub session: String,
    pub client: Option<String>,
    path: PathBuf,
}

impl Context {
    pub fn new(session: String, client: Option<String>) -> Self {
        let mut path = std::env::temp_dir();
        path.push(session.clone() + "-kamp");
        Context {
            session,
            client,
            path,
        }
    }

    pub fn from_env(client: Option<String>) -> Option<Self> {
        use std::env::var;
        var(KAKOUNE_SESSION)
            .map(|s| Context::new(s, client.or_else(|| var(KAKOUNE_CLIENT).ok())))
            .ok()
    }

    pub fn send(&self, body: &str, buffer: Option<String>) -> Result<String, Error> {
        let kak_jh = thread::spawn({
            let mut cmd = String::from("try %{ eval");
            if let Some(buffer) = buffer.as_deref() {
                cmd.push_str(" -buffer ");
                cmd.push_str(buffer);
            } else if let Some(client) = self.client.as_deref() {
                cmd.push_str(" -client ");
                cmd.push_str(client);
            }
            cmd.push_str(" %{");
            if !body.is_empty() {
                cmd.push_str("\n");
                cmd.push_str(body);
            }
            cmd.push_str("\n}} catch %{\n");
            cmd.push_str("echo -debug kamp: %val{error}\n");
            cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n");
            cmd.push_str("}\n");
            cmd.push_str("echo -to-file %opt{kamp_out} ");
            cmd.push_str(END_TOKEN);

            let session = self.session.clone();
            move || kak::pipe(&session, &cmd)
        });

        let (s0, r) = crossbeam_channel::bounded(1);
        let s1 = s0.clone();
        let out_jh = read_out(self.get_out_path(false), s0);
        let err_jh = read_err(self.get_out_path(true), s1);

        let res = r
            .recv()
            .map_err(|_| Error::InvalidSession(self.session.clone()))?;

        if res.is_err() {
            err_jh.join().unwrap()?;
        } else {
            out_jh.join().unwrap()?;
        }

        kak_jh.join().unwrap()?;
        res
    }

    pub fn connect(&self, body: &str) -> Result<(), Error> {
        let kak_jh = thread::spawn({
            let mut cmd = String::from("try %{ eval -try-client '' %{");
            if !body.is_empty() {
                cmd.push_str("\n");
                cmd.push_str(body);
            }
            cmd.push_str("\n}} catch %{\n");
            cmd.push_str("echo -debug kamp: %val{error}\n");
            cmd.push_str("echo -to-file %opt{kamp_err} %val{error}\n");
            cmd.push_str("quit 1\n");
            cmd.push_str("}\n");
            cmd.push_str("echo -to-file %opt{kamp_out} ");
            cmd.push_str(END_TOKEN);

            let session = self.session.clone();
            move || kak::connect(&session, &cmd)
        });

        let (s0, r) = crossbeam_channel::bounded(1);
        let s1 = s0.clone();
        let out_jh = read_out(self.get_out_path(false), s0);
        let err_jh = read_err(self.get_out_path(true), s1);

        let res = r
            .recv()
            .map_err(|_| Error::InvalidSession(self.session.clone()))?;

        if let Err(e) = res {
            err_jh.join().unwrap()?;
            kak_jh.join().unwrap().map_err(|_| e)
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .open(self.get_out_path(true))
                .and_then(|mut f| f.write_all(b""))?;
            out_jh.join().unwrap()?;
            kak_jh.join().unwrap()
        }
    }
}

impl Context {
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
) -> thread::JoinHandle<Result<(), Error>> {
    thread::spawn(move || {
        let mut buf = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .and_then(|mut f| f.read_to_string(&mut buf))?;
        send_ch
            .send(Err(Error::KakEvalCatch(buf)))
            .map_err(anyhow::Error::new)?;
        Ok(())
    })
}

fn read_out(
    file_path: PathBuf,
    send_ch: Sender<Result<String, Error>>,
) -> thread::JoinHandle<Result<(), Error>> {
    thread::spawn(move || {
        let mut buf = String::new();
        let mut f = std::fs::OpenOptions::new().read(true).open(&file_path)?;
        let res = loop {
            f.read_to_string(&mut buf)?;
            if buf.ends_with(END_TOKEN) {
                break buf.trim_end_matches(END_TOKEN);
            }
        };
        send_ch.send(Ok(res.into())).map_err(anyhow::Error::new)?;
        Ok(())
    })
}

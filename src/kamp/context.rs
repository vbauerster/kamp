use crossbeam_channel::Sender;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread;

use super::error::Error;
use super::kak;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

#[derive(Debug)]
pub(crate) struct Context<'a> {
    pub session: String,
    pub client: Option<String>,
    out_path: Cow<'a, Path>,
}

impl Context<'_> {
    pub fn new(session: String, client: Option<String>) -> Self {
        let mut path = std::env::temp_dir();
        path.push(session.clone() + "-kamp");
        Context {
            session,
            client,
            out_path: Cow::from(path),
        }
    }
    pub fn from_env(client: Option<String>) -> Option<Self> {
        use std::env::var;
        var(KAKOUNE_SESSION)
            .map(|s| Context::new(s, client.or_else(|| var(KAKOUNE_CLIENT).ok())))
            .ok()
    }
    pub fn send(&self, body: &str, buffer: Option<String>) -> Result<String, Error> {
        let context = buffer
            .as_deref()
            .and_then(|arg| {
                let switch = " -buffer ";
                let mut buf = String::with_capacity(switch.len() + arg.len());
                buf.push_str(switch);
                buf.push_str(arg);
                Some(buf)
            })
            .or_else(|| {
                self.client.as_deref().and_then(|arg| {
                    let switch = " -try-client ";
                    let mut buf = String::with_capacity(switch.len() + arg.len());
                    buf.push_str(switch);
                    buf.push_str(arg);
                    Some(buf)
                })
            })
            .unwrap_or_default();

        let cmd = format!(
            "eval{} -verbatim -- try %§ {} § catch %§ echo -debug kamp: %val{{error}}; echo -to-file %opt{{kamp_err}} %val{{error}} §",
            context,
            body
        );

        let (s0, r) = crossbeam_channel::bounded(0);
        let s1 = s0.clone();
        let out_jh = self.read_output(false, s0);
        let err_jh = self.read_output(true, s1);

        dbg!(&cmd);
        kak::pipe(&self.session, &cmd)?;

        let res = r.recv().map_err(anyhow::Error::new)?;

        let jh = if res.is_err() { err_jh } else { out_jh };
        jh.join().unwrap().and(res)
    }
    pub fn connect(&self, body: &str) -> Result<(), Error> {
        let kak_jh = thread::spawn({
            let session = self.session.clone();
            let cmd = format!(
                "try %§ {} § catch %§ echo -debug kamp: %val{{error}}; echo -to-file %opt{{kamp_err}} %val{{error}}; quit 1 §",
                body
            );
            dbg!(&cmd);
            move || kak::connect(&session, &cmd)
        });

        let (s0, r) = crossbeam_channel::bounded(0);
        let s1 = s0.clone();
        let out_jh = self.read_output(false, s0);
        let err_jh = self.read_output(true, s1);

        for (i, res) in r.iter().enumerate() {
            match res {
                Ok(_) => {
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open(self.get_out_path(true))
                        .and_then(|mut f| f.write_all(b""))?;
                }
                Err(e) if i == 0 => {
                    return err_jh
                        .join()
                        .unwrap()
                        .and(kak_jh.join().expect("couldn't join kak thread"))
                        .map_err(|_| e);
                }
                Err(_) => {
                    return out_jh
                        .join()
                        .unwrap()
                        .and(err_jh.join().unwrap())
                        .and(kak_jh.join().expect("couldn't join kak thread"));
                }
            }
        }

        Ok(())
    }
}

impl Context<'_> {
    fn get_out_path(&self, err_out: bool) -> PathBuf {
        if err_out {
            self.out_path.with_extension("err")
        } else {
            self.out_path.with_extension("out")
        }
    }
    fn read_output(
        &self,
        err_out: bool,
        send_ch: Sender<Result<String, Error>>,
    ) -> thread::JoinHandle<Result<(), Error>> {
        let file_path = self.get_out_path(err_out);
        thread::spawn(move || {
            eprintln!("start read: {}", file_path.display());
            let mut buf = String::new();
            std::fs::OpenOptions::new()
                .read(true)
                .open(file_path)
                .and_then(|mut f| f.read_to_string(&mut buf))?;
            send_ch
                .send(if err_out {
                    Err(Error::KakEvalCatch(buf))
                } else {
                    Ok(buf)
                })
                .map_err(anyhow::Error::new)?;
            eprintln!("{} read thread done", if err_out { "err" } else { "out" });
            Ok(())
        })
    }
}

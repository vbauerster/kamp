// mod ctx;
// pub(super) use ctx::Ctx;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession(#[from] std::env::VarError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("kak exited with error: {0}")]
    KakFailure(std::process::ExitStatus),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

#[derive(Debug)]
pub(super) struct Context {
    pub session: String,
    pub client: Option<String>,
}

// impl std::fmt::Display for Context {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "session: {}\nclient: {}",
//             self.session,
//             self.client.as_deref().unwrap_or_default()
//         )
//     }
// }

impl Context {
    pub fn new(session: String) -> Self {
        Context {
            session,
            client: None,
        }
    }
    pub fn from_env() -> Result<Self, Error> {
        Ok(Context {
            session: std::env::var(KAKOUNE_SESSION)?,
            client: std::env::var(KAKOUNE_CLIENT).ok(),
        })
    }
    pub fn set_client_if_any(&mut self, client: Option<String>) {
        if client.is_some() {
            self.client = client;
        }
    }
    pub fn send(&self, body: &str) -> Result<(), Error> {
        let buffer: Option<String> = None;
        // let temp_dir = std::env::temp_dir();
        // println!("Temporary directory: {}", temp_dir.display());

        // let client = self.client.and_then(|mut client| {
        //     client.insert_str(0, "-try-client ");
        //     Some(client)
        // });

        // let client = format_args!("-try-client {}", self)

        // let _buffer_arg = buffer
        //     .as_deref()
        //     .map(|s| format_args!("-buffer {}", String::from(s)));
        // let _client_arg = self.client.map(|s| format_args!("-try-client {}", s));

        let buffer = buffer.as_deref().and_then(|arg| {
            let switch = " -buffer ";
            let mut tmp = String::with_capacity(switch.len() + arg.len());
            tmp.push_str(switch);
            tmp.push_str(arg);
            Some(tmp)
        });
        let client = self.client.as_deref().and_then(|arg| {
            let switch = " -try-client ";
            let mut tmp = String::with_capacity(switch.len() + arg.len());
            tmp.push_str(switch);
            tmp.push_str(arg);
            Some(tmp)
        });

        let eval_cmd = format!(
            "try %§ eval{} {} §",
            buffer.or(client).unwrap_or_default(),
            body
        );

        dbg!(&eval_cmd);
        kak_exec(&self.session, &eval_cmd)
    }
}

fn kak_exec<T: AsRef<[u8]>>(session: &str, cmd: T) -> Result<(), Error> {
    use std::io::Write;
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
        Err(Error::KakFailure(status))?;
    }

    // if !output.status.success() {
    //     let kak_err = String::from_utf8(output.stderr).map_err(anyhow::Error::new)?;
    //     // let msg = format!(
    //     //     "kak command failed:\n {}\nHave you quit kak with session {}?",
    //     //     kak_err, session
    //     // );
    //     // Err(Error::new(ErrorKind::Other, msg))?;
    //     Err(Error::KakFailure(kak_err))?;
    // }

    Ok(())
}

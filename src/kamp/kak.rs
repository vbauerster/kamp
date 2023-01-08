use std::io::Write;
use std::process::{Command, ExitStatus, Stdio};

use super::Result;

pub(crate) struct Sessions(String);

impl Sessions {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.lines().collect::<Vec<_>>().into_iter()
    }
}

pub(crate) fn sessions() -> Result<Sessions> {
    use anyhow::Error;
    let output = Command::new("kak").arg("-l").output()?;

    if !output.status.success() {
        if let Some(code) = output.status.code() {
            return Err(Error::msg(format!("kak exited with code: {}", code)).into());
        }
        return Err(Error::msg("kak terminated by signal").into());
    }

    String::from_utf8(output.stdout)
        .map_err(|e| Error::new(e).into())
        .map(Sessions)
}

pub(crate) fn pipe<S, T>(session: S, cmd: T) -> Result<ExitStatus>
where
    S: AsRef<str>,
    T: AsRef<[u8]>,
{
    let mut child = Command::new("kak")
        .arg("-p")
        .arg(session.as_ref())
        .stdin(Stdio::piped())
        .spawn()?;

    let kak_stdin = match child.stdin.as_mut() {
        Some(stdin) => stdin,
        None => {
            use std::io::{Error, ErrorKind};
            return Err(Error::new(ErrorKind::Other, "cannot capture stdin of kak process").into());
        }
    };

    kak_stdin.write_all(cmd.as_ref())?;

    let status = child.wait()?;

    Ok(status)
}

pub(crate) fn connect<S: AsRef<str>>(session: S, cmd: S) -> Result<ExitStatus> {
    let status = Command::new("kak")
        .arg("-c")
        .arg(session.as_ref())
        .arg("-e")
        .arg(cmd.as_ref())
        .status()?;

    Ok(status)
}

pub(crate) fn proxy(args: Vec<String>) -> Result<()> {
    use std::os::unix::prelude::CommandExt;
    let err = Command::new("kak").args(args).exec();
    Err(err.into())
}

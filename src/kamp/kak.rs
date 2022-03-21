use std::{ffi::OsStr, io::Write};

use super::error::Error;
use std::process::{Command, Stdio};

pub(crate) fn pipe<T: AsRef<[u8]>>(session: &str, cmd: T) -> Result<(), Error> {
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
                "cannot capture stdin of kak process",
            ))?
        }
    };

    kak_stdin.write_all(cmd.as_ref())?;

    let status = child.wait()?;

    if !status.success() {
        return Err(Error::KakProcess(status));
    }

    Ok(())
}

pub(crate) fn connect<S: AsRef<OsStr>>(session: &str, e_cmd: S) -> Result<(), Error> {
    let status = Command::new("kak")
        .arg("-c")
        .arg(session)
        .arg("-e")
        .arg(e_cmd)
        .status()?;

    if !status.success() {
        return Err(Error::KakProcess(status));
    }

    Ok(())
}

pub(crate) fn proxy(args: Vec<String>) -> Result<(), Error> {
    let status = Command::new("kak").args(args).status()?;

    if !status.success() {
        return Err(Error::KakProcess(status));
    }

    Ok(())
}

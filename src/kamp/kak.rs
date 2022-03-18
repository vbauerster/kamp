use std::io::Write;

use super::error::Error;

pub(crate) fn kak_p<T: AsRef<[u8]>>(session: &str, cmd: T) -> Result<(), Error> {
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

pub(crate) fn kak_c(session: &str, e_cmd: &str) -> Result<(), Error> {
    use std::process::Command;
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

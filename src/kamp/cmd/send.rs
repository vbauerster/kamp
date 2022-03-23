use super::Context;
use super::Error;

pub(crate) fn send(
    ctx: Context,
    cmd: Vec<String>,
    buffers: Option<Vec<String>>,
) -> Result<(), Error> {
    if cmd.is_empty() {
        return Err(Error::InvalidArgument("some command is expected".into()));
    }
    let cmd = cmd.iter().fold(String::new(), |mut buf, next| {
        buf.push_str(next);
        buf.push_str(" ");
        buf
    });
    ctx.send(&cmd, buffers.map(super::to_csv_buffers).flatten())
        .map(|_| ())
}

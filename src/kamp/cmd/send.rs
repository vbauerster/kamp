use super::Context;
use super::Error;

pub(crate) fn send(ctx: Context, buffers: Vec<String>, cmd: &str) -> Result<(), Error> {
    ctx.send(cmd, super::to_csv_buffers(buffers)).map(|_| ())
}

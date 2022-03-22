use super::Context;
use super::Error;

pub(crate) fn send(ctx: Context, buffers: Vec<String>, cmd: String) -> Result<(), Error> {
    let cmd = format!("  {}\n", cmd);
    ctx.send(&cmd, super::to_csv_buffers(buffers)).map(|_| ())
}

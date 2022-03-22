use super::Context;
use super::Error;

pub(crate) fn cat(ctx: Context, buffers: Vec<String>) -> Result<String, Error> {
    let cmd = "write %opt{kamp_out}";
    ctx.send(&cmd, super::to_csv_buffers(buffers))
}

use super::Context;
use super::Error;

pub(crate) fn cat(ctx: &Context, buffers: Option<Vec<String>>) -> Result<String, Error> {
    let cmd = "write %opt{kamp_out}";
    ctx.send(&cmd, buffers.map(super::to_csv_buffers).flatten())
}

use super::Context;
use super::Error;

pub(crate) fn cat(ctx: &Context, buffer: Option<String>) -> Result<String, Error> {
    let cmd = "write %opt{kamp_out}";
    ctx.send(&cmd, buffer)
}

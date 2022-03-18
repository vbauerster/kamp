use super::Context;
use super::Error;

pub(crate) fn attach(ctx: Context) -> Result<(), Error> {
    ctx.connect("echo -to-file %opt{kamp_out}")
}
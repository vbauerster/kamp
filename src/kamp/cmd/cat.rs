use super::Context;
use super::Error;

pub(crate) fn cat(ctx: &Context, buffer: Option<String>) -> Result<String, Error> {
    if ctx.client.is_none() && buffer.is_none() {
        return Err(Error::InvalidContext);
    }
    ctx.send("write %opt{kamp_out}", buffer)
}

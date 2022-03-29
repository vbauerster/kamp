use super::Context;
use super::Error;

pub(crate) fn cat(ctx: &Context, buffer: Option<String>) -> Result<String, Error> {
    if ctx.client.as_deref().or(buffer.as_deref()).is_none() {
        return Err(Error::InvalidContext);
    }
    ctx.send("write %opt{kamp_out}", buffer)
}

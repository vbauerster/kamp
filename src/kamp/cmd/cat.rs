use super::Context;
use super::{Error, Result};

pub(crate) fn cat(ctx: Context, buffer: Option<String>) -> Result<String> {
    if ctx.is_draft() && buffer.is_none() {
        return Err(Error::InvalidContext("either client or buffer is required"));
    }
    ctx.send("write %opt{kamp_out}", buffer)
}

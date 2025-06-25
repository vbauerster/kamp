use super::Context;
use super::{Error, Result};

pub(crate) fn cat(ctx: Context, buffer_ctx: Option<(String, i32)>) -> Result<String> {
    if ctx.is_draft() && buffer_ctx.is_none() {
        return Err(Error::InvalidContext("either client or buffer is required"));
    }
    ctx.send("write %opt{kamp_out}", buffer_ctx)
}

use super::Context;
use super::{Error, Result};

pub(crate) fn cat(mut ctx: Context, buffer_ctx: Option<(String, i32)>) -> Result<String> {
    if ctx.is_draft() && buffer_ctx.is_none() {
        return Err(Error::InvalidContext("either client or buffer is required"));
    }
    ctx.send(buffer_ctx, "write %opt{kamp_out}")
}

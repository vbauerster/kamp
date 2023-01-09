use super::Context;
use super::Result;

pub(crate) fn kill(ctx: &Context, exit_status: Option<i32>) -> Result<()> {
    ctx.send_kill(exit_status)
}

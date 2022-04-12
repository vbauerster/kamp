use super::Context;
use super::Error;

pub(crate) fn kill(ctx: &Context, exit_status: Option<i32>) -> Result<(), Error> {
    ctx.send_kill(exit_status)
}

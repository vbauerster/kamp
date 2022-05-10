use super::Context;
use super::Error;

pub(crate) fn send(ctx: &Context, cmd: String, buffer: Option<String>) -> Result<(), Error> {
    ctx.send(cmd, buffer).map(drop)
}

use super::Context;
use super::Error;

pub(crate) fn send(ctx: &Context, cmd: &str, buffer: Option<String>) -> Result<(), Error> {
    ctx.send(&cmd, buffer).map(|_| ())
}

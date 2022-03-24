use super::Context;
use super::Error;

pub(crate) fn send(ctx: &Context, cmd: &str, buffers: Option<Vec<String>>) -> Result<(), Error> {
    ctx.send(&cmd, buffers.map(super::to_csv_buffers).flatten())
        .map(|_| ())
}

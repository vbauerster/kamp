use super::Context;
use super::Error;
use std::fmt::Write;

pub(crate) fn ctx(ctx: Context) -> Result<String, Error> {
    let mut buf = String::new();
    writeln!(&mut buf, "session: {}", ctx.session)?;
    writeln!(&mut buf, "client: {}", ctx.client.unwrap_or_default())?;
    Ok(buf)
}

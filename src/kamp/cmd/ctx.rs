use super::Context;
use super::Error;
use std::fmt::Write;

pub(crate) fn ctx(ctx: &Context) -> Result<String, Error> {
    let mut buf = String::new();
    writeln!(&mut buf, "{}", ctx)?;
    Ok(buf)
}

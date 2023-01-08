use super::Context;
use super::Result;
use super::Session;
use std::fmt::Write;

fn get_sessions() -> Result<Vec<Session>> {
    crate::kamp::kak::sessions()?
        .iter()
        .map(|session| Context::new(session, None).session_struct())
        .collect()
}

pub(crate) fn list_all() -> Result<String> {
    let mut buf = String::new();
    for session in get_sessions()? {
        writeln!(&mut buf, "{:#?}", session)?;
    }
    Ok(buf)
}

pub(crate) fn list(ctx: &Context) -> Result<String> {
    let mut buf = String::new();
    let session = ctx.session_struct()?;
    writeln!(&mut buf, "{:#?}", session)?;
    Ok(buf)
}

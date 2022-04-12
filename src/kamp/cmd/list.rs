use super::Context;
use super::Error;
use super::Session;
use std::fmt::Write;

fn get_sessions<P>(predicate: P) -> Result<Vec<Session>, Error>
where
    P: FnMut(&&str) -> bool,
{
    crate::kamp::kak::sessions()?
        .iter()
        .filter(predicate)
        .map(|session| Context::new(session, None).session_struct())
        .collect()
}

pub(crate) fn list_all(ctx: Option<Context>) -> Result<String, Error> {
    let mut buf = String::new();
    if let Some(ctx) = &ctx {
        for session in get_sessions(|&s| s != ctx.session())? {
            writeln!(&mut buf, "{:#?}", session)?;
        }
        let current = list(ctx)?;
        buf.push_str(&current);
    } else {
        for session in get_sessions(|_| true)? {
            writeln!(&mut buf, "{:#?}", session)?;
        }
    }
    Ok(buf)
}

pub(crate) fn list(ctx: &Context) -> Result<String, Error> {
    let mut buf = String::new();
    let session = ctx.session_struct()?;
    writeln!(&mut buf, "{:#?}", session)?;
    Ok(buf)
}

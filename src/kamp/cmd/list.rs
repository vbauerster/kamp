use super::Context;
use super::Error;
use super::Get;
use crate::kamp::kak;
use std::fmt::Write;

#[allow(unused)]
#[derive(Debug)]
struct Session {
    session: String,
    clients: Vec<String>,
}

pub(crate) fn list() -> Result<String, Error> {
    let mut buf = String::new();
    for session in get_sessions()? {
        writeln!(&mut buf, "{:#?}", session)?;
    }
    Ok(buf)
}

fn get_sessions() -> Result<Vec<Session>, Error> {
    kak::sessions()?
        .iter()
        .map(|session| Context::new(String::from(session), None))
        .map(|ctx| {
            let session = ctx.session.clone();
            Get::Val(String::from("client_list"))
                .run(ctx, false, None)
                .map(|clients| Session {
                    session: session,
                    clients: clients.lines().map(String::from).collect(),
                })
        })
        .collect()
}

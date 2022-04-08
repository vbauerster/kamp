use super::Context;
use super::Error;
use super::Get;
use crate::kamp::kak;
use std::fmt::Write;

#[allow(unused)]
#[derive(Debug)]
struct Session {
    name: String,
    pwd: String,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: String, pwd: String, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Client {
    name: String,
    buffile: String,
}

impl Client {
    fn new(name: String, buffile: String) -> Self {
        Client { name, buffile }
    }
}

fn get_sessions<P>(predicate: P) -> Result<Vec<Session>, Error>
where
    P: FnMut(&&str) -> bool,
{
    kak::sessions()?
        .iter()
        .filter(predicate)
        .map(|session| {
            let ctx = Context::new(session, None);
            get_ctx_session(&ctx)
        })
        .collect()
}

fn get_ctx_session(ctx: &Context) -> Result<Session, Error> {
    Get::new_val("client_list")
        .run(ctx, 0, None)
        .and_then(|clients| {
            clients
                .lines()
                .map(|name| {
                    let ctx = Context::new(ctx.session_as_ref(), Some(name));
                    Get::new_val("buffile")
                        .run(&ctx, 2, None)
                        .map(|bf| Client::new(name.into(), bf))
                })
                .collect::<Result<Vec<Client>, Error>>()
        })
        .and_then(|clients| {
            Get::new_sh("pwd")
                .run(ctx, 2, None)
                .map(|pwd| Session::new(ctx.session_clone(), pwd, clients))
        })
}

pub(crate) fn list_all(ctx: Option<Context>) -> Result<String, Error> {
    let mut buf = String::new();
    if let Some(ctx) = &ctx {
        for session in get_sessions(|&s| s != ctx.session_as_ref())? {
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
    let session = get_ctx_session(ctx)?;
    writeln!(&mut buf, "{:#?}", session)?;
    Ok(buf)
}

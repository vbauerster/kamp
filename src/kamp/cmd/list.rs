use super::Context;
use super::Result;
use std::fmt::Write;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
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
pub struct Client {
    name: String,
    bufname: String,
}

impl Client {
    fn new(name: String, bufname: String) -> Self {
        Client { name, bufname }
    }
}

fn get_sessions() -> Result<Vec<Session>> {
    crate::kamp::kak::sessions()?
        .iter()
        .map(|session| to_session_struct(Context::new(session, None)))
        .collect()
}

pub(crate) fn list_all() -> Result<String> {
    let mut buf = String::new();
    for session in get_sessions()? {
        writeln!(&mut buf, "{:#?}", session)?;
    }
    Ok(buf)
}

pub(crate) fn list(ctx: Context) -> Result<String> {
    let mut buf = String::new();
    let session = to_session_struct(ctx)?;
    writeln!(&mut buf, "{:#?}", session)?;
    Ok(buf)
}

fn to_session_struct(ctx: Context) -> Result<Session> {
    ctx.query_val("client_list", 0, None)
        .and_then(|clients| {
            clients
                .lines()
                .map(|name| {
                    let mut ctx_clone = ctx.clone();
                    ctx_clone.set_client(name);
                    ctx_clone
                        .query_val("bufname", 2, None)
                        .map(|bufname| Client::new(name.into(), bufname))
                })
                .collect::<Result<Vec<Client>>>()
        })
        .and_then(|clients| {
            ctx.query_sh("pwd", 2, None)
                .map(|pwd| Session::new(ctx.session().into_owned(), pwd, clients))
        })
}

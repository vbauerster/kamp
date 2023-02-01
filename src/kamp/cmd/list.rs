use super::Result;
use super::{Context, SplitType};

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

pub(crate) fn list_all() -> Result<Vec<Session>> {
    crate::kamp::kak::sessions()?
        .iter()
        .map(|session| to_session_struct(Context::new(session, None)))
        .collect()
}

pub(crate) fn list_current(ctx: Context) -> Result<Session> {
    to_session_struct(ctx)
}

fn to_session_struct(ctx: Context) -> Result<Session> {
    ctx.query_val("client_list", SplitType::Kakoune, None)
        .and_then(|clients| {
            clients
                .iter()
                .map(|name| {
                    let mut ctx_clone = ctx.clone();
                    ctx_clone.set_client(name);
                    ctx_clone
                        .query_val("bufname", SplitType::none_quote_raw(), None)
                        .map(|mut v| Client::new(name.into(), v.remove(0)))
                })
                .collect::<Result<Vec<Client>>>()
        })
        .and_then(|clients| {
            ctx.query_sh("pwd", SplitType::none_quote_raw(), None)
                .map(|mut v| Session::new(ctx.session().into_owned(), v.remove(0), clients))
        })
}

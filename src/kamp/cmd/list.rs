use super::Context;
use super::Result;

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
    let v = crate::kamp::kak::list_sessions()?;
    let s = String::from_utf8(v).map_err(anyhow::Error::new)?;
    s.lines()
        .map(|session| to_session_struct(Context::new(session, None)))
        .collect()
}

pub(crate) fn list_current(ctx: Context) -> Result<Session> {
    to_session_struct(ctx)
}

fn to_session_struct(ctx: Context) -> Result<Session> {
    let clients = ctx.query_val("client_list", false, true, None)?;
    let clients = clients
        .into_iter()
        .flat_map(|name| {
            let mut ctx = ctx.clone();
            ctx.set_client(Some(name));
            ctx.query_val("bufname", false, false, None)
                .map(|mut v| (ctx.take_client(), v.pop()))
        })
        .flat_map(|(client, bufname)| match (client, bufname) {
            (Some(client), Some(bufname)) => Some(Client::new(client, bufname)),
            (Some(client), None) => Some(Client::new(client, String::new())),
            _ => None,
        })
        .collect();
    ctx.query_sh("pwd", None)
        .map(|mut pwd| Session::new(ctx.session().into(), pwd.pop().unwrap_or_default(), clients))
}

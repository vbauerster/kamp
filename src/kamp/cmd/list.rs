use std::rc::Rc;

use super::Context;
use super::Result;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    name: Box<str>,
    pwd: Box<str>,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: Box<str>, pwd: Box<str>, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    name: Rc<str>,
    bufname: Box<str>,
}

impl Client {
    fn new(name: Rc<str>, bufname: Box<str>) -> Client {
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

fn to_session_struct(mut ctx: Context) -> Result<Session> {
    let clients = ctx.query_val("client_list", false, true, None)?;
    let clients = clients
        .into_iter()
        .flat_map(|name| {
            let mut ctx = ctx.clone();
            let client: Rc<str> = name.into();
            ctx.set_client(client.clone());
            ctx.query_val("bufname", false, false, None)
                .map(|mut v| Client::new(client, v.pop().unwrap_or_default().into()))
        })
        .collect();
    ctx.query_sh("pwd", None).map(|mut pwd| {
        Session::new(
            ctx.own_session(),
            pwd.pop().unwrap_or_default().into(),
            clients,
        )
    })
}

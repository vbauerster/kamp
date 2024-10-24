use std::rc::Rc;

use super::Context;
use super::Result;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    name: &'static str,
    pwd: String,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: &'static str, pwd: String, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    name: Rc<str>,
    bufname: String,
}

impl Client {
    fn new(name: Rc<str>, bufname: String) -> Client {
        Client { name, bufname }
    }
}

pub(crate) fn list_all() -> Result<Vec<Session>> {
    let v = crate::kamp::kak::list_sessions()?;
    String::from_utf8(v)
        .map_err(anyhow::Error::new)?
        .lines()
        .map(|s| to_session_struct(String::from(s).leak()))
        .collect()
}

pub(crate) fn list_current(session: &'static str) -> Result<Session> {
    to_session_struct(session)
}

fn to_session_struct(session: &'static str) -> Result<Session> {
    let clients = Context::from(session).query_val(None, "client_list", false, true)?;
    let clients = clients
        .into_iter()
        .flat_map(|name| {
            let client: Rc<str> = Rc::from(name.into_boxed_str());
            let mut ctx = Context::from(session);
            ctx.set_client(client.clone());
            ctx.query_val(None, "bufname", false, false)
                .map(|mut v| Client::new(client, v.pop().unwrap_or_default()))
        })
        .collect();
    Context::from(session)
        .query_sh(None, "pwd")
        .map(|mut pwd| Session::new(session, pwd.pop().unwrap_or_default(), clients))
}

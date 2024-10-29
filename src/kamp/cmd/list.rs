use std::rc::Rc;

use super::Context;
use super::Result;

#[allow(unused)]
#[derive(Debug)]
pub struct Session<'a> {
    name: &'a str,
    pwd: String,
    clients: Vec<Client>,
}

impl<'a> Session<'a> {
    fn new(name: &'a str, pwd: String, clients: Vec<Client>) -> Self {
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

pub(crate) fn list_all<'a>(sessions: impl Iterator<Item = &'a str>) -> Result<Vec<Session<'a>>> {
    sessions.map(|s| list_current(Context::from(s))).collect()
}

pub(crate) fn list_current(mut ctx: Context) -> Result<Session<'static>> {
    let clients = ctx.query_val(None, "client_list", false, true)?;
    let clients = clients
        .into_iter()
        .flat_map(|name| {
            ctx.set_client(name);
            ctx.query_val(None, "bufname", false, false)
                .map(|mut v| Client::new(ctx.client().unwrap(), v.pop().unwrap_or_default()))
        })
        .collect();
    ctx.unset_client();
    ctx.query_sh(None, "pwd")
        .map(|mut pwd| Session::new(ctx.session(), pwd.pop().unwrap_or_default(), clients))
}

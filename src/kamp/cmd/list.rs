use std::rc::Rc;

use super::QueryContext;
use super::QueryKeyVal;

use super::Context;
use super::Result;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    name: Rc<Box<str>>,
    pwd: String,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: Rc<Box<str>>, pwd: String, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    name: Rc<Box<str>>,
    bufname: String,
}

impl Client {
    fn new(name: Rc<Box<str>>, bufname: String) -> Client {
        Client { name, bufname }
    }
}

pub(crate) fn list_all(
    sessions: impl Iterator<Item = String>,
    debug: bool,
) -> Result<Vec<Session>> {
    sessions
        .map(|s| list_current(Context::new(s, debug)))
        .collect()
}

pub(crate) fn list_current(mut ctx: Context) -> Result<Session> {
    let qctx = QueryContext::new(
        QueryKeyVal::Val("client_list".into()),
        Default::default(),
        Default::default(),
        false,
    );
    let clients = ctx
        .query_kak(qctx, None)?
        .into_iter()
        .flat_map(|name| {
            ctx.set_client(name);
            ctx.query_kak(
                QueryContext::new(
                    QueryKeyVal::Val("bufname".into()),
                    Default::default(),
                    Default::default(),
                    false,
                ),
                None,
            )
            .map(|mut v| Client::new(ctx.client().unwrap(), v.pop().unwrap_or_default()))
        })
        .collect();
    ctx.set_client("");
    ctx.query_kak(QueryContext::new_sh(vec!["pwd".into()], true), None)
        .map(|mut pwd| Session::new(ctx.session(), pwd.pop().unwrap_or_default(), clients))
}

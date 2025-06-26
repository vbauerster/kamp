use std::rc::Rc;

use super::QueryContext;
use super::QueryKeyVal;
use super::QueryType;
use super::Quoting;

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
    name: Rc<Box<str>>,
    bufname: String,
}

impl Client {
    fn new(name: Rc<Box<str>>, bufname: String) -> Client {
        Client { name, bufname }
    }
}

pub(crate) fn list_all<'a>(
    sessions: impl Iterator<Item = &'a str>,
    debug: bool,
) -> Result<Vec<Session<'a>>> {
    sessions.map(|s| list_current(Context::from(s))).collect()
}

pub(crate) fn list_current(mut ctx: Context) -> Result<Session<'static>> {
    let qctx = QueryContext::new(
        QueryKeyVal::Val("client_list".into()),
        QueryType::List,
        Quoting::Kakoune,
        false,
    );
    let clients = ctx
        .query_kak(qctx, None)?
        .into_iter()
        .flat_map(|name| {
            ctx.set_client(Some(Rc::new(name.into_boxed_str())));
            ctx.query_kak(
                QueryContext::new(
                    QueryKeyVal::Val("bufname".into()),
                    QueryType::Plain,
                    Quoting::Kakoune,
                    false,
                ),
                None,
            )
            .map(|mut v| Client::new(ctx.client().unwrap(), v.pop().unwrap_or_default()))
        })
        .collect();
    ctx.set_client(None);
    ctx.query_kak(QueryContext::new_sh(vec!["pwd".into()], true), None)
        .map(|mut pwd| Session::new(ctx.session(), pwd.pop().unwrap_or_default(), clients))
}

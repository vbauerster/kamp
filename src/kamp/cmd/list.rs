use super::QueryContext;
use super::QueryKeyVal;

use super::Context;
use super::Result;

#[allow(unused)]
#[derive(Debug)]
pub struct Session {
    name: &'static str,
    pwd: Box<str>,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: &'static str, pwd: Box<str>, clients: Vec<Client>) -> Self {
        Session { name, pwd, clients }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Client {
    name: Box<str>,
    bufname: String,
}

impl Client {
    fn new(name: Box<str>, bufname: String) -> Client {
        Client { name, bufname }
    }
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
            let name = name.into_boxed_str();
            ctx.set_client(Some(name.clone()));
            ctx.query_kak(
                QueryContext::new(
                    QueryKeyVal::Val("bufname".into()),
                    Default::default(),
                    Default::default(),
                    false,
                ),
                None,
            )
            .map(|mut v| Client::new(name, v.pop().unwrap_or_default()))
        })
        .collect();
    ctx.set_client(None);
    ctx.query_kak(QueryContext::new_sh(vec!["pwd".into()], true), None)
        .map(|mut pwd| {
            let pwd = pwd.pop().unwrap_or_default().into_boxed_str();
            Session::new(ctx.session(), pwd, clients)
        })
}

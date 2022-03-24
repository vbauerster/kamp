use super::Context;
use super::Error;
use super::Get;
use crate::kamp::kak;
use std::fmt::Write;

#[allow(unused)]
#[derive(Debug)]
struct Session {
    name: String,
    clients: Vec<Client>,
}

impl Session {
    fn new(name: String, clients: Vec<Client>) -> Self {
        Session { name, clients }
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

fn get_sessions() -> Result<Vec<Session>, Error> {
    kak::sessions()?
        .iter()
        .map(|session| Context::new(String::from(session), None))
        .map(|mut ctx| {
            let session = ctx.session.clone();
            Get::Val(String::from("client_list"))
                .run(&ctx, 0, None)
                .and_then(|clients| {
                    let res = clients
                        .lines()
                        .map(|name| {
                            ctx.client = Some(String::from(name));
                            Get::Val(String::from("buffile"))
                                .run(&ctx, 2, None)
                                .map(|bf| Client::new(ctx.client.take().unwrap(), String::from(bf)))
                        })
                        .collect::<Result<Vec<_>, Error>>();
                    res.map(|clients| Session::new(session, clients))
                })
        })
        .collect()
}

pub(crate) fn list() -> Result<String, Error> {
    let mut buf = String::new();
    for session in get_sessions()? {
        writeln!(&mut buf, "{:#?}", session)?;
    }
    Ok(buf)
}

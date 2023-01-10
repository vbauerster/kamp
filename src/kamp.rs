mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use argv::{Kampliment, SubCommand::*};
use context::Context;
use error::{Error, Result};

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

pub(super) fn run() -> Result<Option<String>> {
    let kamp: Kampliment = argh::from_env();
    let (session, client) = match (kamp.session, kamp.client.filter(|s| !s.is_empty())) {
        (Some(s), client) => (Some(s), client),
        (None, client) => (
            std::env::var(KAKOUNE_SESSION).ok(),
            client.or_else(|| std::env::var(KAKOUNE_CLIENT).ok()),
        ),
    };

    match (kamp.subcommand, session.as_deref()) {
        (Init(opt), _) => cmd::init(opt.export, opt.alias).map(Some),
        (Attach(opt), Some(session)) => {
            let ctx = Context::new(session, client.as_deref());
            cmd::attach(ctx, opt.buffer).map(|_| None)
        }
        (Edit(opt), Some(session)) => {
            let ctx = Context::new(session, client.as_deref());
            cmd::edit(ctx, opt.files).map(|_| None)
        }
        (Edit(opt), None) => kak::proxy(opt.files).map(|_| None),
        (Send(opt), Some(session)) => {
            if opt.command.is_empty() {
                return Err(Error::CommandRequired);
            }
            let ctx = Context::new(session, client.as_deref());
            ctx.send(
                opt.command.join(" "),
                to_csv_buffers_or_asterisk(opt.buffers),
            )
            .map(|_| None)
        }
        (List(opt), _) if opt.all => cmd::list_all().map(Some),
        (List(_), Some(session)) => {
            let ctx = Context::new(session, client.as_deref());
            cmd::list(ctx).map(Some)
        }
        (Kill(opt), Some(session)) => {
            let ctx = Context::new(session, client.as_deref());
            ctx.send_kill(opt.exit_status).map(|_| None)
        }
        (Get(opt), Some(session)) => {
            use argv::GetSubCommand::*;
            let ctx = Context::new(session, client.as_deref());
            let res = match opt.subcommand {
                Val(o) => ctx.query_val(o.name, o.rawness, to_csv_buffers_or_asterisk(o.buffers)),
                Opt(o) => ctx.query_opt(o.name, o.rawness, to_csv_buffers_or_asterisk(o.buffers)),
                Reg(o) => ctx.query_reg(o.name),
                Shell(o) => {
                    if o.command.is_empty() {
                        return Err(Error::CommandRequired);
                    }
                    ctx.query_sh(
                        o.command.join(" "),
                        o.rawness,
                        to_csv_buffers_or_asterisk(o.buffers),
                    )
                }
            };
            res.map(Some)
        }
        (Cat(opt), Some(session)) => {
            let ctx = Context::new(session, client.as_deref());
            cmd::cat(ctx, to_csv_buffers_or_asterisk(opt.buffers)).map(Some)
        }
        (Ctx(_), Some(session)) => {
            let mut buf = format!("session: {}\n", session);
            if let Some(client) = &client {
                buf.push_str("client: ");
                buf.push_str(client);
                buf.push('\n');
            }
            Ok(Some(buf))
        }
        (Version(_), _) => Ok(Some(format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))),
        _ => Err(Error::InvalidContext("session is required")),
    }
}

fn to_csv_buffers_or_asterisk(buffers: Vec<String>) -> Option<String> {
    if buffers.is_empty() {
        return None;
    }
    if buffers[0] == "*" {
        return buffers.into_iter().rev().last();
    }
    let mut res =
        buffers
            .into_iter()
            .filter(|s| s != "*")
            .fold(String::from('\''), |mut buf, next| {
                buf.push_str(&next);
                buf.push(',');
                buf
            });
    res.pop(); // pops last ','
    res.push('\'');
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_csv_buffers_or_asterisk() {
        assert_eq!(to_csv_buffers_or_asterisk(vec![]), None);
        assert_eq!(
            to_csv_buffers_or_asterisk(vec!["*".into()]),
            Some("*".into())
        );
        assert_eq!(
            to_csv_buffers_or_asterisk(vec!["*".into(), "a".into()]),
            Some("*".into())
        );
        assert_eq!(
            to_csv_buffers_or_asterisk(vec!["a".into(), "*".into()]),
            Some("'a'".into())
        );
        assert_eq!(
            to_csv_buffers_or_asterisk(vec!["a".into(), "b".into()]),
            Some("'a,b'".into())
        );
        assert_eq!(
            to_csv_buffers_or_asterisk(vec!["a".into(), "b".into(), "c".into()]),
            Some("'a,b,c'".into())
        );
    }
}

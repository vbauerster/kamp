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

pub(super) fn run() -> Result<()> {
    let kamp: Kampliment = argh::from_env();
    if kamp.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let (session, client) = match (kamp.session, kamp.client.filter(|s| !s.is_empty())) {
        (Some(s), client) => (Some(s), client),
        (None, client) => (
            std::env::var(KAKOUNE_SESSION).ok(),
            client.or_else(|| std::env::var(KAKOUNE_CLIENT).ok()),
        ),
    };

    if let Some(subcommand) = kamp.subcommand {
        match (subcommand, session.as_deref()) {
            (Init(opt), _) => {
                let res = cmd::init(opt.export, opt.alias)?;
                print!("{res}");
            }
            (Attach(opt), Some(session)) => {
                let ctx = Context::new(session, client.as_deref());
                return cmd::attach(ctx, opt.buffer);
            }
            (Edit(opt), Some(session)) => {
                let ctx = Context::new(session, client.as_deref());
                return cmd::edit(ctx, opt.focus, opt.files);
            }
            (Edit(opt), None) => {
                return kak::proxy(opt.files);
            }
            (Send(opt), Some(session)) => {
                if opt.command.is_empty() {
                    return Err(Error::CommandRequired);
                }
                let ctx = Context::new(session, client.as_deref());
                let buffer_ctx = to_buffer_ctx(opt.buffers);
                let res = ctx.send(opt.command.join(" "), buffer_ctx)?;
                print!("{res}");
            }
            (List(opt), _) if opt.all => {
                for session in cmd::list_all()? {
                    println!("{session:#?}");
                }
            }
            (List(_), Some(session)) => {
                let ctx = Context::new(session, client.as_deref());
                let session = cmd::list_current(ctx)?;
                println!("{session:#?}");
            }
            (Kill(opt), Some(session)) => {
                let ctx = Context::new(session, client.as_deref());
                return ctx.send_kill(opt.exit_status);
            }
            (Get(opt), Some(session)) => {
                use argv::GetSubCommand::*;
                let ctx = Context::new(session, client.as_deref());
                let res = match opt.subcommand {
                    Val(o) => {
                        let buffer_ctx = to_buffer_ctx(o.buffers);
                        ctx.query_val(o.name, o.quote, o.split || o.zplit, buffer_ctx)
                            .map(|v| (v, !o.quote && o.zplit))
                    }
                    Opt(o) => {
                        let buffer_ctx = to_buffer_ctx(o.buffers);
                        ctx.query_opt(o.name, o.quote, o.split || o.zplit, buffer_ctx)
                            .map(|v| (v, !o.quote && o.zplit))
                    }
                    Reg(o) => ctx
                        .query_reg(o.name, o.quote, o.split || o.zplit)
                        .map(|v| (v, !o.quote && o.zplit)),
                    Shell(o) => {
                        if o.command.is_empty() {
                            return Err(Error::CommandRequired);
                        }
                        let buffer_ctx = to_buffer_ctx(o.buffers);
                        ctx.query_sh(o.command.join(" "), buffer_ctx)
                            .map(|v| (v, false))
                    }
                };
                let (items, zplit) = res?;
                let split_char = if zplit { '\0' } else { '\n' };
                for item in items {
                    print!("{item}{split_char}");
                }
            }
            (Cat(opt), Some(session)) => {
                let ctx = Context::new(session, client.as_deref());
                let buffer_ctx = to_buffer_ctx(opt.buffers);
                let res = cmd::cat(ctx, buffer_ctx)?;
                print!("{res}");
            }
            _ => return Err(Error::InvalidContext("session is required")),
        }
    } else if let Some(session) = session {
        println!("session: {session}");
        if let Some(client) = client {
            println!("client: {client}");
        }
    } else {
        return Err(Error::InvalidContext("session is required"));
    }

    Ok(())
}

fn to_buffer_ctx(buffers: Vec<String>) -> Option<(String, i32)> {
    if buffers.is_empty() {
        return None;
    }
    let mut count = 0;
    if buffers[0] == "*" {
        return buffers.into_iter().rev().last().map(|s| (s, count));
    }
    let mut res =
        buffers
            .into_iter()
            .filter(|s| s != "*")
            .fold(String::from('\''), |mut buf, next| {
                count += 1;
                buf.push_str(&next);
                buf.push(',');
                buf
            });
    res.pop(); // pops last ','
    res.push('\'');
    Some((res, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_buffer_ctx() {
        assert_eq!(to_buffer_ctx(vec![]), None);
        assert_eq!(to_buffer_ctx(vec!["*".into()]), Some(("*".into(), 0)));
        assert_eq!(
            to_buffer_ctx(vec!["*".into(), "a".into()]),
            Some(("*".into(), 0))
        );
        assert_eq!(
            to_buffer_ctx(vec!["a".into(), "*".into()]),
            Some(("'a'".into(), 1))
        );
        assert_eq!(to_buffer_ctx(vec!["a".into()]), Some(("'a'".into(), 1)));
        assert_eq!(
            to_buffer_ctx(vec!["a".into(), "b".into()]),
            Some(("'a,b'".into(), 2))
        );
    }
}

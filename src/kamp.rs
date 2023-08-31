mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use context::Context;
use error::{Error, Result};

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

pub(super) fn run() -> Result<()> {
    let kamp: argv::Kampliment = argh::from_env();
    if kamp.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let (session, client) = match (kamp.session, kamp.client) {
        (None, Some(c)) if c.is_empty() => (std::env::var(KAKOUNE_SESSION).ok(), None),
        (None, Some(c)) => (std::env::var(KAKOUNE_SESSION).ok(), Some(c)),
        (None, None) => (
            std::env::var(KAKOUNE_SESSION).ok(),
            std::env::var(KAKOUNE_CLIENT).ok(),
        ),
        (session, client) => (
            session.filter(|s| !s.is_empty()),
            client.filter(|c| !c.is_empty()),
        ),
    };

    let Some(command) = kamp.subcommand else {
        return match (session, client) {
            (Some(s), Some(c)) => {
                println!("session: {s}");
                println!("client: {c}");
                Ok(())
            }
            (Some(s), None) => {
                println!("session: {s}");
                Ok(())
            }
            _ => Err(Error::InvalidContext("session is required")),
        };
    };

    use argv::SubCommand as sub;
    match (command, session.as_deref()) {
        (sub::Init(opt), _) => {
            let res = cmd::init(opt.export, opt.alias)?;
            print!("{res}");
        }
        (sub::Attach(opt), Some(session)) => {
            let ctx = Context::new(session, client);
            return cmd::attach(ctx, opt.buffer);
        }
        (sub::Edit(opt), Some(session)) => {
            let ctx = Context::new(session, client);
            return cmd::edit(ctx, opt.focus, opt.files);
        }
        (sub::Edit(opt), None) => {
            return kak::proxy(opt.files).map_err(|err| err.into());
        }
        (sub::Send(opt), Some(session)) => {
            if opt.command.is_empty() {
                return Err(Error::CommandRequired);
            }
            let ctx = Context::new(session, client);
            let buffer_ctx = to_buffer_ctx(opt.buffers);
            let res = ctx.send(opt.command.join(" "), buffer_ctx)?;
            print!("{res}");
        }
        (sub::List(opt), _) if opt.all => {
            for session in cmd::list_all()? {
                println!("{session:#?}");
            }
        }
        (sub::List(_), Some(session)) => {
            let ctx = Context::new(session, client);
            let session = cmd::list_current(ctx)?;
            println!("{session:#?}");
        }
        (sub::Kill(opt), Some(session)) => {
            let ctx = Context::new(session, client);
            return ctx.send_kill(opt.exit_status);
        }
        (sub::Get(opt), Some(session)) => {
            use argv::GetSubCommand as get;
            let ctx = Context::new(session, client);
            let res = match opt.subcommand {
                get::Val(o) => {
                    let buffer_ctx = to_buffer_ctx(o.buffers);
                    ctx.query_val(o.name, o.quote, o.split || o.zplit, buffer_ctx)
                        .map(|v| (v, !o.quote && o.zplit))
                }
                get::Opt(o) => {
                    let buffer_ctx = to_buffer_ctx(o.buffers);
                    ctx.query_opt(o.name, o.quote, o.split || o.zplit, buffer_ctx)
                        .map(|v| (v, !o.quote && o.zplit))
                }
                get::Reg(o) => ctx
                    .query_reg(o.name, o.quote, o.split || o.zplit)
                    .map(|v| (v, !o.quote && o.zplit)),
                get::Shell(o) => {
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
        (sub::Cat(opt), Some(session)) => {
            let ctx = Context::new(session, client);
            let buffer_ctx = to_buffer_ctx(opt.buffers);
            let res = cmd::cat(ctx, buffer_ctx)?;
            print!("{res}");
        }
        _ => return Err(Error::InvalidContext("session is required")),
    }

    Ok(())
}

fn to_buffer_ctx(buffers: Vec<String>) -> Option<(String, i32)> {
    let mut iter = buffers.into_iter();
    let first = iter.next()?;
    let mut res = String::from('\'');
    res.push_str(&first);
    if first == "*" {
        res.push('\'');
        return Some((res, 0));
    }

    let mut count = 1;
    let mut res = iter.filter(|s| s != "*").fold(res, |mut buf, next| {
        buf.push(',');
        buf.push_str(&next);
        count += 1;
        buf
    });
    res.push('\'');
    Some((res, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_buffer_ctx() {
        assert_eq!(to_buffer_ctx(vec![]), None);
        assert_eq!(to_buffer_ctx(vec!["*".into()]), Some(("'*'".into(), 0)));
        assert_eq!(
            to_buffer_ctx(vec!["*".into(), "a".into()]),
            Some(("'*'".into(), 0))
        );
        assert_eq!(
            to_buffer_ctx(vec!["a".into(), "*".into()]),
            Some(("'a'".into(), 1))
        );
        assert_eq!(
            to_buffer_ctx(vec!["a".into(), "*".into(), "b".into()]),
            Some(("'a,b'".into(), 2))
        );
        assert_eq!(to_buffer_ctx(vec!["a".into()]), Some(("'a'".into(), 1)));
        assert_eq!(
            to_buffer_ctx(vec!["a".into(), "b".into()]),
            Some(("'a,b'".into(), 2))
        );
    }
}

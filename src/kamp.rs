mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use context::Context;
use error::{Error, Result};
use std::io::Write;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

trait Dispatcher {
    fn dispatch<W: Write>(self, writer: W, ctx: Option<Context>) -> Result<()>;
}

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

    let mut output = std::io::stdout();

    let Some(command) = kamp.subcommand else {
        return if session.is_some() {
            [session, client]
                .into_iter()
                .zip(["session", "client"])
                .try_for_each(|(opt, name)| match opt {
                    Some(val) => writeln!(output, "{name}: {val}"),
                    None => Ok(()),
                })
                .map_err(|e| e.into())
        } else {
            Err(Error::InvalidContext("session is required"))
        };
    };

    use argv::SubCommand as sub;
    match command {
        sub::Init(opt) => cmd::init(opt.export, opt.alias)
            .and_then(|res| write!(output, "{res}").map_err(|e| e.into())),
        sub::List(opt) if opt.all => {
            let res = cmd::list_all()?;
            res.into_iter()
                .try_for_each(|session| writeln!(output, "{session:#?}"))
                .map_err(|e| e.into())
        }
        _ => {
            let ctx = session.map(|session| Context::new(session.into(), client));
            command.dispatch(output, ctx)
        }
    }
}

impl Dispatcher for argv::SubCommand {
    fn dispatch<W: Write>(self, mut writer: W, ctx: Option<Context>) -> Result<()> {
        use argv::SubCommand as sub;
        match (self, ctx) {
            (sub::Attach(opt), Some(ctx)) => cmd::attach(ctx, opt.buffer),
            (sub::Edit(opt), Some(ctx)) => cmd::edit(ctx, opt.focus, opt.files),
            (sub::Edit(opt), None) => kak::proxy(opt.files).map_err(|err| err.into()),
            (sub::Send(opt), Some(ctx)) => {
                if opt.command.is_empty() {
                    return Err(Error::CommandRequired);
                }
                ctx.send(opt.command.join(" "), to_buffer_ctx(opt.buffers))
                    .and_then(|res| write!(writer, "{res}").map_err(|e| e.into()))
            }
            (sub::List(_), Some(ctx)) => cmd::list_current(ctx)
                .and_then(|session| writeln!(writer, "{session:#?}").map_err(|e| e.into())),
            (sub::Kill(opt), Some(ctx)) => ctx.send_kill(opt.exit_status),
            (sub::Get(opt), Some(ctx)) => {
                use argv::GetSubCommand as get;
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
                items
                    .into_iter()
                    .try_for_each(|item| write!(writer, "{item}{split_char}"))
                    .map_err(|e| e.into())
            }
            (sub::Cat(opt), Some(ctx)) => {
                let buffer_ctx = to_buffer_ctx(opt.buffers);
                cmd::cat(ctx, buffer_ctx)
                    .and_then(|res| write!(writer, "{res}").map_err(|e| e.into()))
            }
            _ => Err(Error::InvalidContext("session is required")),
        }
    }
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

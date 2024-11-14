mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use argv::SubCommand;
use context::Context;
use error::{Error, Result};
use std::io::Write;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

pub(crate) trait Dispatcher {
    fn dispatch<W: Write>(self, ctx: Context, writer: W) -> Result<()>;
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
        return if session.is_none() {
            Err(Error::InvalidContext("session is required"))
        } else {
            [session, client]
                .into_iter()
                .zip(["session", "client"])
                .try_for_each(|(opt, name)| match opt {
                    Some(val) => writeln!(output, "{name}: {val}"),
                    None => Ok(()),
                })
                .map_err(From::from)
        };
    };

    match command {
        SubCommand::Init(opt) => cmd::init(opt.export, opt.alias)
            .and_then(|res| write!(output, "{res}").map_err(From::from)),
        SubCommand::List(opt) if opt.all => {
            let sessions = kak::list_sessions()?;
            let sessions = String::from_utf8(sessions).map_err(anyhow::Error::new)?;
            cmd::list_all(sessions.lines()).and_then(|v| {
                v.into_iter()
                    .try_for_each(|session| writeln!(output, "{session:#?}"))
                    .map_err(From::from)
            })
        }
        SubCommand::Edit(opt) if session.is_none() => kak::proxy(opt.files).map_err(From::from),
        _ => session
            .ok_or_else(|| Error::InvalidContext("session is required"))
            .map(|s| Context::new(Box::leak(s.into_boxed_str()), client))
            .and_then(|ctx| ctx.dispatch(command, output)),
    }
}

impl Dispatcher for SubCommand {
    fn dispatch<W: Write>(self, ctx: Context, mut writer: W) -> Result<()> {
        match self {
            SubCommand::Attach(opt) => cmd::attach(ctx, opt.buffer),
            SubCommand::Edit(opt) => {
                let session = ctx.session();
                let client = ctx.client();
                let scratch = cmd::edit(ctx, opt.focus, opt.files)?;
                if let (Some(client), false) = (client, opt.focus) {
                    writeln!(
                        writer,
                        "{} is opened in client: {client}, session: {session}",
                        if scratch { "scratch" } else { "file" }
                    )?;
                }
                Ok(())
            }
            SubCommand::Send(opt) => {
                if opt.command.is_empty() {
                    return Err(Error::CommandRequired);
                }
                ctx.send(to_buffer_ctx(opt.buffers), opt.command.join(" "))
                    .and_then(|res| write!(writer, "{res}").map_err(From::from))
            }
            SubCommand::List(_) => cmd::list_current(ctx)
                .and_then(|session| writeln!(writer, "{session:#?}").map_err(From::from)),
            SubCommand::Kill(opt) => ctx.send_kill(opt.exit_status),
            SubCommand::Get(opt) => {
                use argv::get::SubCommand;
                let buffer_ctx = to_buffer_ctx(opt.buffers);
                let res = match opt.subcommand {
                    SubCommand::Value(o) => ctx
                        .query_val(buffer_ctx, o.name, o.quote, o.split || o.zplit)
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Option(o) => ctx
                        .query_opt(buffer_ctx, o.name, o.quote, o.split || o.zplit)
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Register(o) => ctx
                        .query_reg(buffer_ctx, o.name, o.quote, o.split || o.zplit)
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Shell(o) => {
                        if o.command.is_empty() {
                            return Err(Error::CommandRequired);
                        }
                        ctx.query_sh(buffer_ctx, o.command.join(" "))
                            .map(|v| (v, false))
                    }
                };
                res.and_then(|(items, zplit)| {
                    let split_char = if zplit { '\0' } else { '\n' };
                    items
                        .into_iter()
                        .try_for_each(|item| write!(writer, "{item}{split_char}"))
                        .map_err(From::from)
                })
            }
            SubCommand::Cat(opt) => cmd::cat(ctx, to_buffer_ctx(opt.buffers))
                .and_then(|res| write!(writer, "{res}").map_err(From::from)),
            _ => unreachable!(),
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

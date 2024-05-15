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

    match command {
        SubCommand::Init(opt) => cmd::init(opt.export, opt.alias)
            .and_then(|res| write!(output, "{res}").map_err(|e| e.into())),
        SubCommand::List(opt) if opt.all => cmd::list_all().and_then(|res| {
            res.into_iter()
                .try_for_each(|session| writeln!(output, "{session:#?}"))
                .map_err(|e| e.into())
        }),
        SubCommand::Edit(opt) if session.is_none() => kak::proxy(opt.files).map_err(|e| e.into()),
        _ => match session {
            Some(session) => Context::new(session, client).dispatch(command, output),
            None => Err(Error::InvalidContext("session is required")),
        },
    }
}

impl Dispatcher for SubCommand {
    fn dispatch<W: Write>(self, mut ctx: Context, mut writer: W) -> Result<()> {
        match self {
            SubCommand::Attach(opt) => cmd::attach(ctx, opt.buffer),
            SubCommand::Edit(opt) => cmd::edit(ctx, opt.focus, opt.files),
            SubCommand::Send(opt) => {
                let body = to_send_body(opt.command)?;
                ctx.send(to_buffer_ctx(opt.buffers), body)
                    .and_then(|res| write!(writer, "{res}").map_err(|e| e.into()))
            }
            SubCommand::List(_) => cmd::list_current(ctx)
                .and_then(|session| writeln!(writer, "{session:#?}").map_err(|e| e.into())),
            SubCommand::Kill(opt) => ctx.send_kill(opt.exit_status),
            SubCommand::Get(opt) => {
                use argv::get::SubCommand;
                match opt.subcommand {
                    SubCommand::Val(o) => ctx
                        .query_val(
                            to_buffer_ctx(o.buffers),
                            o.name,
                            o.quote,
                            o.split || o.zplit,
                        )
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Opt(o) => ctx
                        .query_opt(
                            to_buffer_ctx(o.buffers),
                            o.name,
                            o.quote,
                            o.split || o.zplit,
                        )
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Reg(o) => ctx
                        .query_reg(None, o.name, o.quote, o.split || o.zplit)
                        .map(|v| (v, !o.quote && o.zplit)),
                    SubCommand::Shell(o) => {
                        let command = to_send_body(o.command)?;
                        ctx.query_sh(to_buffer_ctx(o.buffers), command)
                            .map(|v| (v, false))
                    }
                }
                .and_then(|(items, zplit)| {
                    let split_char = if zplit { '\0' } else { '\n' };
                    items
                        .into_iter()
                        .try_for_each(|item| write!(writer, "{item}{split_char}"))
                        .map_err(|e| e.into())
                })
            }
            SubCommand::Cat(opt) => {
                let buffer_ctx = to_buffer_ctx(opt.buffers);
                cmd::cat(ctx, buffer_ctx)
                    .and_then(|res| write!(writer, "{res}").map_err(|e| e.into()))
            }
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

fn to_send_body(command: Vec<String>) -> Result<String> {
    if command.is_empty() {
        return Err(Error::CommandRequired);
    }
    let mut body = String::new();
    body.push_str(&command[0]);
    for s in &command[1..] {
        body.push(' ');
        if s.contains(' ') {
            body.push('\'');
            body.push_str(s);
            body.push('\'');
        } else {
            body.push_str(s);
        }
    }
    Ok(body)
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

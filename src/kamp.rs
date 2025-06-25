mod cmd;
mod context;
mod error;
mod kak;

use super::argv::{Kampliment, SubCommand};
use context::Context;
use error::{Error, Result};
use std::io::Write;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

pub(crate) trait Dispatcher {
    fn dispatch<W: Write>(self, ctx: Context, writer: W) -> Result<()>;
}

pub(super) fn run() -> Result<()> {
    let kamp: Kampliment = argh::from_env();
    if kamp.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let session = kamp
        .session
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var(KAKOUNE_SESSION).ok());

    let client = match session {
        None => None,
        Some(_) => kamp
            .client
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var(KAKOUNE_CLIENT).ok()),
    };

    let command = kamp
        .subcommand
        .unwrap_or_else(|| SubCommand::Ctx(Default::default()));

    let mut output = std::io::stdout();

    match command {
        SubCommand::Init(opt) => cmd::init(opt.export, opt.alias)
            .and_then(|res| write!(output, "{res}").map_err(From::from)),
        SubCommand::Ctx(opt) => {
            let Some(session) = session else {
                return Err(Error::InvalidContext("session is required"));
            };
            match (opt.client, client) {
                (true, None) => Err(Error::InvalidContext("client is required")),
                (true, Some(client)) => writeln!(output, "client: {client}").map_err(From::from),
                (false, _) => writeln!(output, "session: {session}").map_err(From::from),
            }
        }
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
                let body = if opt.verbatim {
                    opt.command.join(" ")
                } else {
                    opt.command.iter().fold(String::new(), |mut buf, x| {
                        if !buf.is_empty() {
                            buf.push(' ');
                        }
                        if x.contains([' ', '"', '\'']) {
                            let s = x.replace("'", "''");
                            buf.push('\'');
                            buf.push_str(&s);
                            buf.push('\'');
                        } else if x.is_empty() {
                            buf.push('\'');
                            buf.push_str(x);
                            buf.push('\'');
                        } else {
                            buf.push_str(x);
                        }
                        buf
                    })
                };
                ctx.send(body, to_buffer_ctx(opt.buffers)).map(drop)
            }
            SubCommand::List(_) => cmd::list_current(ctx)
                .and_then(|session| writeln!(writer, "{session:#?}").map_err(From::from)),
            SubCommand::Kill(opt) => ctx.send_kill(opt.exit_status),
            SubCommand::Get(opt) => {
                let split_by = if opt.zplit { '\0' } else { '\n' };
                ctx.query_kak(opt.subcommand, to_buffer_ctx(opt.buffers))
                    .and_then(|items| {
                        items
                            .into_iter()
                            .try_for_each(|item| write!(writer, "{item}{split_by}"))
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

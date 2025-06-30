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

    let command = kamp
        .subcommand
        .unwrap_or_else(|| SubCommand::Ctx(Default::default()));

    let stdout = std::io::stdout();
    let mut output = stdout.lock();

    match command {
        SubCommand::Init(opt) => {
            let init = cmd::init(opt.export, opt.alias)?;
            write!(output, "{init}")?;
        }
        SubCommand::List(opt) if opt.all => {
            let sessions = kak::list_sessions()?;
            let sessions = String::from_utf8(sessions)?;
            let sessions = cmd::list_all(sessions.lines().map(String::from), kamp.debug)?;
            for session in sessions {
                writeln!(output, "{session:#?}")?;
            }
        }
        SubCommand::Edit(opt) if session.is_none() => {
            kak::proxy(opt.files)?;
        }
        _ => {
            let Some(session) = session else {
                return Err(Error::InvalidContext("session is required"));
            };
            let mut ctx = Context::new(session.into_boxed_str(), kamp.debug);
            if let Some(client) = kamp.client.or_else(|| std::env::var(KAKOUNE_CLIENT).ok()) {
                ctx.set_client(client.into_boxed_str());
            }
            ctx.dispatch(command, output)?;
        }
    };
    Ok(())
}

impl Dispatcher for SubCommand {
    fn dispatch<W: Write>(self, ctx: Context, mut writer: W) -> Result<()> {
        match self {
            SubCommand::Ctx(opt) => match ctx.client() {
                None if opt.client => {
                    return Err(Error::InvalidContext("client is required"));
                }
                None => {
                    writeln!(writer, "session: {}", ctx.session())?;
                }
                Some(client) => {
                    writeln!(writer, "session: {}", ctx.session())?;
                    writeln!(writer, "client: {client}")?;
                }
            },
            SubCommand::Attach(opt) => {
                cmd::attach(ctx, opt.buffer)?;
            }
            SubCommand::Edit(opt) => {
                let session = ctx.session();
                let client = ctx.client();
                let scratch = cmd::edit(ctx, opt.new, opt.focus, opt.files)?;
                if let (Some(client), false) = (client, opt.focus) {
                    writeln!(
                        writer,
                        "{} is opened in client: {client}, session: {session}",
                        if scratch { "scratch" } else { "file" }
                    )?;
                }
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
                ctx.send(body, to_buffer_ctx(opt.buffers)).map(drop)?;
            }
            SubCommand::List(_) => {
                let session = cmd::list_current(ctx)?;
                writeln!(writer, "{session:#?}")?;
            }
            SubCommand::Kill(opt) => {
                ctx.send_kill(opt.exit_status)?;
            }
            SubCommand::Get(opt) => {
                let split_by = if opt.zplit { '\0' } else { '\n' };
                let items = ctx.query_kak(opt.subcommand, to_buffer_ctx(opt.buffers))?;
                for item in items {
                    write!(writer, "{item}{split_by}")?;
                }
            }
            SubCommand::Cat(opt) => {
                let res = cmd::cat(ctx, to_buffer_ctx(opt.buffers))?;
                write!(writer, "{res}")?;
            }
            _ => unreachable!(),
        };
        Ok(())
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

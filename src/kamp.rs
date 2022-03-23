mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use argv::{Kampliment, SubCommand::*};
use context::Context;
use error::Error;

pub(super) fn run() -> Result<Option<String>, Error> {
    let kampliment: Kampliment = argh::from_env();
    let mut client = kampliment.client;

    let ctx = kampliment
        .session
        .map(|s| Context::new(s, client.take()))
        .or_else(|| Context::from_env(client.take()))
        .ok_or_else(|| Error::NoSession);

    match kampliment.subcommand {
        Init(opt) => {
            cmd::init(opt.export, opt.alias);
            Ok(None)
        }
        Attach(_) => cmd::attach(ctx?).map(|_| None),
        Edit(opt) => {
            if let Ok(ctx) = ctx {
                cmd::edit(ctx, opt.files).map(|_| None)
            } else {
                kak::proxy(opt.files).map(|_| None)
            }
        }
        Send(mut opt) => {
            if opt.all_buffers {
                opt.buffers.clear();
                opt.buffers.push("*".into());
            }
            cmd::send(ctx?, opt.buffers, opt.command).map(|_| None)
        }
        Get(opt) => cmd::Get::from(opt.subcommand)
            .run(ctx?, opt.raw, opt.buffers)
            .map(Some),
        Cat(opt) => cmd::cat(ctx?, opt.buffers).map(Some),
        Ctx(_) => ctx.map(|ctx| {
            use std::fmt::Write;
            let mut buf = String::new();
            writeln!(&mut buf, "session: {}", ctx.session).unwrap();
            writeln!(&mut buf, "client: {}", ctx.client.unwrap_or_default()).unwrap();
            Some(buf)
        }),
    }
}

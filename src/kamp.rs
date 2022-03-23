mod argv;
mod cmd;
mod context;
mod error;
mod kak;

use argv::{Kampliment, SubCommand::*};
use context::Context;
use error::Error;

pub(super) fn run() -> Result<Option<String>, Error> {
    let kamp: Kampliment = argh::from_env();
    let mut client = kamp.client;

    let ctx = kamp
        .session
        .map(|s| Context::new(s, client.take()))
        .or_else(|| Context::from_env(client.take()))
        .ok_or_else(|| Error::NoSession);

    match kamp.subcommand {
        Init(opt) => cmd::init(opt.export, opt.alias).map(Some),
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
        Ctx(_) => cmd::ctx(ctx?).map(Some),
    }
}

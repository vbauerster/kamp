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
        .ok_or(Error::NoSession);

    match kamp.subcommand {
        Init(opt) => cmd::init(opt.export, opt.alias).map(Some),
        Attach(opt) => cmd::attach(&ctx?, opt.buffer).map(|_| None),
        Edit(opt) => {
            if let Ok(ctx) = ctx {
                cmd::edit(&ctx, opt.files).map(|_| None)
            } else {
                kak::proxy(opt.files).map_err(Error::Other).map(|_| None)
            }
        }
        Send(opt) => cmd::send(&ctx?, &opt.command, to_csv_buffers(opt.buffers)).map(|_| None),
        List(opt) => {
            if opt.all {
                cmd::list_all(ctx.ok()).map(Some)
            } else {
                cmd::list(&mut ctx?).map(Some)
            }
        }
        Get(opt) => cmd::Get::from(opt.subcommand)
            .run(&ctx?, opt.raw, to_csv_buffers(opt.buffers))
            .map(Some),
        Cat(opt) => cmd::cat(&ctx?, to_csv_buffers(opt.buffers)).map(Some),
        Ctx(_) => cmd::ctx(&ctx?).map(Some),
        Version(_) => cmd::version().map(Some),
    }
}

fn to_csv_buffers(buffers: Vec<String>) -> Option<String> {
    if buffers.is_empty() {
        return None;
    }
    if buffers[0] == "*" {
        return Some("*".into());
    }
    let buffers = buffers.into_iter().filter(|s| s != "*").collect::<Vec<_>>();
    let mut res =
        buffers
            .iter()
            .take(buffers.len() - 1)
            .fold(String::from("'"), |mut buf, next| {
                buf.push_str(next);
                buf.push(',');
                buf
            });
    res.push_str(&buffers[buffers.len() - 1]);
    res.push('\'');
    Some(res)
}

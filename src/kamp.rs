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
        Attach(_) => cmd::attach(&ctx?).map(|_| None),
        Edit(opt) => {
            if let Ok(ctx) = ctx {
                cmd::edit(&ctx, opt.files).map(|_| None)
            } else {
                kak::proxy(opt.files).map(|_| None)
            }
        }
        Send(opt) => cmd::send(&ctx?, &opt.command, to_csv_buffers(opt.buffers)).map(|_| None),
        List(_) => cmd::list().map(Some),
        Get(opt) => cmd::Get::from(opt.subcommand)
            .run(&ctx?, opt.raw, to_csv_buffers(opt.buffers))
            .map(Some),
        Cat(opt) => cmd::cat(&ctx?, to_csv_buffers(opt.buffers)).map(Some),
        Ctx(_) => cmd::ctx(&ctx?).map(Some),
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
                buf.push_str(",");
                buf
            });
    res.push_str(&buffers[buffers.len() - 1]);
    res.push_str("'");
    Some(res)
}

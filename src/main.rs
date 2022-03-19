mod argv;
mod kamp;

use anyhow::{Error, Result};
use argv::{Kampliment, SubCommand::*};
use kamp::{cmd, Context};

fn main() -> Result<()> {
    let kampliment: Kampliment = argh::from_env();
    let mut client = kampliment.client;

    let ctx = kampliment
        .session
        .map(|s| Context::new(s, client.take()))
        .or_else(|| Context::from_env(client.take()))
        .ok_or_else(|| Error::msg("no session in context"));

    match kampliment.subcommand {
        Init(opt) => {
            cmd::init(opt.export, opt.alias);
        }
        Attach(_) => {
            cmd::attach(ctx?)?;
        }
        Edit(opt) => {
            if let Ok(ctx) = ctx {
                cmd::edit(ctx, opt.files)?;
            } else {
                kamp::proxy(opt.files)?;
            }
        }
        Get(opt) => {
            let res = cmd::Get::from(opt.subcommand).run(ctx?, &opt.quoting)?;
            println!("{}", res);
        }
        Ctx(_) => {
            let ctx = ctx?;
            println!("session: {}", ctx.session);
            println!("client: {}", ctx.client.unwrap_or_default());
        }
    }

    Ok(())
}

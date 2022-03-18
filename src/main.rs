mod argv;
mod kamp;

use anyhow::{bail, Result};
use argv::{Kampliment, SubCommand::*};
use kamp::{cmd, Context};

fn main() -> Result<()> {
    let kampliment: Kampliment = argh::from_env();

    let ctx = kampliment
        .session
        .map(Context::new)
        .or_else(Context::from_env)
        .and_then(|mut ctx| {
            ctx.set_client_if_any(kampliment.client);
            Some(ctx)
        });

    match kampliment.subcommand {
        Init(opt) => {
            cmd::init(opt.export);
        }
        Edit(opt) => {
            if let Some(ctx) = ctx {
                cmd::edit(ctx, opt.files)?;
            } else {
                kamp::proxy(opt.files)?;
            }
        }
        Ctx(_) => {
            if let Some(ctx) = ctx {
                println!("session: {}", ctx.session);
                println!("client: {}", ctx.client.unwrap_or_default());
            } else {
                bail!("no session in context");
            }
        }
    }

    Ok(())
}

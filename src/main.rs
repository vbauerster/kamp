mod argv;
mod kamp;

use anyhow::Result;
use argv::{Kampliment, SubCommand::*};
use kamp::cmd;
use kamp::Context;

fn main() -> Result<()> {
    let kamp: Kampliment = argh::from_env();

    let ctx = kamp
        .session
        .map(Context::new)
        .or_else(Context::from_env)
        .and_then(|mut ctx| {
            ctx.set_client_if_any(kamp.client);
            Some(ctx)
        });

    match kamp.subcommand {
        Edit(opt) => {
            let cmd = cmd::Edit::new(opt.files);
            cmd.run(ctx)?;
        }
        Ctx(_) => {
            todo!();
            // println!("session: {}", ctx.session);
            // println!("client: {}", ctx.client.as_deref().unwrap_or_default());
        }
    }

    Ok(())
}

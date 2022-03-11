mod argv;
mod cmd;

use anyhow::Result;
use argv::{Kampliment, SubCommand::*};
use cmd::Context;

fn main() -> Result<()> {
    let kamp: Kampliment = argh::from_env();
    let mut ctx = match kamp.session.map(Context::new) {
        Some(ctx) => ctx,
        None => Context::from_env()?,
    };
    ctx.set_client_if_any(kamp.client);
    match kamp.subcommand {
        Edit(opt) => {
            let cmd = format!("edit -existing '{}'; echo -to-file %opt{{kamp_out}}", opt.file_name);
            ctx.send(&cmd)?;
        }
        Ctx(_) => {
            println!("session: {}", ctx.session);
            println!("client: {}", ctx.client.as_deref().unwrap_or_default());
        }
    };

    // let dir = env::temp_dir();
    // println!("Temporary directory: {}", dir.display());

    Ok(())
}

mod argv;
mod cmd;

use std::fmt::Write;

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
            let mut buf = String::from("edit ");
            if let Some(p) = opt.file {
                buf.write_fmt(format_args!("-existing '{}'", p.display()))?;
            } else {
                buf.push_str("-scratch");
            }
            buf.push_str("; echo -to-file %opt{kamp_out}");
            if ctx.client.is_some() {
                let _r = ctx.send(&buf)?;
            } else {
                ctx.connect(&buf)?;
            }
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

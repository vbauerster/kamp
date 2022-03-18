mod argv;
mod cmd;

use anyhow::Result;
use argv::{Kampliment, SubCommand::*};
use cmd::Context;

fn main() -> Result<()> {
    let kamp: Kampliment = argh::from_env();
    // let mut ctx = match kamp.session.map(Context::new) {
    //     Some(ctx) => ctx,
    //     None => Context::from_env()?,
    // };

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
            if let Some(ctx) = ctx {
                let mut buf = String::new();
                if opt.files.is_empty() {
                    buf.push_str("edit -scratch; ");
                } else {
                    let names = opt.files.iter().fold(String::new(), |mut buf, item| {
                        if !item.starts_with("+") {
                            buf.push_str("\n");
                        }
                        buf.push_str(item);
                        buf
                    });
                    for name in names.split("\n").skip_while(|&s| s.is_empty()) {
                        let mut edit = String::from("edit -existing ");
                        for (i, item) in name.splitn(2, "+").enumerate() {
                            match i {
                                0 => {
                                    edit.push_str("'");
                                    edit.push_str(item);
                                    edit.push_str("'");
                                }
                                1 => item
                                    .splitn(2, ":")
                                    .take_while(|&s| s.parse::<i32>().is_ok())
                                    .for_each(|n| {
                                        edit.push_str(" ");
                                        edit.push_str(&n.to_string());
                                    }),
                                _ => unreachable!(),
                            }
                        }
                        buf.push_str(&edit);
                        buf.push_str("; ");
                    }
                }
                buf.push_str("echo -to-file %opt{kamp_out}");
                if ctx.client.is_some() {
                    let _r = ctx.send(&buf)?;
                } else {
                    ctx.connect(&buf)?;
                }
            } else {
                // just proxy to kak
                todo!();
            }
        }
        Ctx(_) => {
            todo!();
            // println!("session: {}", ctx.session);
            // println!("client: {}", ctx.client.as_deref().unwrap_or_default());
        }
    };

    // let dir = env::temp_dir();
    // println!("Temporary directory: {}", dir.display());

    Ok(())
}

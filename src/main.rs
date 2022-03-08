mod argv;
mod cmd;

use anyhow::Result;
use cmd::Context;

fn main() -> Result<()> {
    use argv::{Kampliment, SubCommand::*};
    let kamp: Kampliment = argh::from_env();
    match kamp.subcommand {
        Env(opt) => {
            let cmd = cmd::Env::from(opt);
            cmd.run(Context::from_env()?);
        }
    };

    // let dir = env::temp_dir();
    // println!("Temporary directory: {}", dir.display());

    Ok(())
}

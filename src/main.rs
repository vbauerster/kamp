mod commands;

use anyhow::Result;
use argh::FromArgs;
use std::env;

// const KAKOUNE_SESSION: &'static str = "KAKOUNE_SESSION";
// const KAKOUNE_CLIENT: &'static str = "KAKOUNE_CLIENT";

#[derive(FromArgs, PartialEq, Debug)]
/// Kakoune kampliment
struct KampCli {
    #[argh(subcommand)]
    subcommand: SubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Env(EnvOptions),
}

/// print env
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "env")]
struct EnvOptions {
    /// json output
    #[argh(switch, short = 'j')]
    json: bool,
}

fn main() -> Result<()> {
    // let session = env::var(KAKOUNE_SESSION).expect(&format!("{KAKOUNE_SESSION} is not set"));
    // let session = env::var(KAKOUNE_SESSION).expect(KAKOUNE_SESSION);
    // let client = env::var(KAKOUNE_CLIENT).unwrap_or_default();
    // println!("session is {session}");
    // println!("client is {client}");

    let cli: KampCli = argh::from_env();
    match cli.subcommand {
        SubCommands::Env(_) => {
            let ctx = commands::env::get()?;
            println!("{:?}", ctx);
        }
    };

    let dir = env::temp_dir();
    println!("Temporary directory: {}", dir.display());

    Ok(())
}

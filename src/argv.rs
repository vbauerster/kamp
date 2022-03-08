use argh::FromArgs;

/// Kakoune kampliment
#[derive(FromArgs, PartialEq, Debug)]
pub(super) struct Kampliment {
    /// session
    #[argh(option)]
    pub session: Option<String>,

    /// client
    #[argh(option)]
    pub client: Option<String>,

    #[argh(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub(super) enum SubCommand {
    Env(EnvOptions),
}

/// print env
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "env")]
pub(crate) struct EnvOptions {
    /// json output
    #[argh(switch, short = 'j')]
    json: bool,
}

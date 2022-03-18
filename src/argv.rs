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
    Init(InitOptions),
    Edit(EditOptions),
    Ctx(CtxOptions),
}

/// kakoune init
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
pub(crate) struct InitOptions {}

/// show execution context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ctx")]
pub(crate) struct CtxOptions {}

/// edit a file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "edit")]
pub(crate) struct EditOptions {
    /// path to file
    #[argh(positional)]
    pub files: Vec<String>,
}

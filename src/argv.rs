use argh::{FromArgValue, FromArgs};

/// Kakoune kampliment
#[derive(FromArgs, PartialEq, Debug)]
pub(super) struct Kampliment {
    /// session
    #[argh(option, short = 's')]
    pub session: Option<String>,

    /// client
    #[argh(option, short = 'c')]
    pub client: Option<String>,

    #[argh(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub(super) enum SubCommand {
    Init(InitOptions),
    Attach(AttachOptions),
    Edit(EditOptions),
    Ctx(CtxOptions),
}

/// kakoune init
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
pub(crate) struct InitOptions {
    /// export 'VAR=VALUE'
    #[argh(option, short = 'e')]
    pub export: Vec<KeyValue>,
}

/// show execution context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ctx")]
pub(crate) struct CtxOptions {}

/// attach to a context session
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "attach")]
pub(crate) struct AttachOptions {}

/// edit a file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "edit")]
pub(crate) struct EditOptions {
    /// path to file
    #[argh(positional)]
    pub files: Vec<String>,
}

#[derive(PartialEq, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl FromArgValue for KeyValue {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        value
            .split_once("=")
            .ok_or(String::from("invalid key=value"))
            .map(|(key, value)| KeyValue {
                key: key.into(),
                value: value.trim_matches(|c| c == '\'' || c == '"').into(),
            })
    }
}

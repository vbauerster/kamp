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
    Get(GetOptions),
    Ctx(CtxOptions),
}

/// kakoune init
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
pub(crate) struct InitOptions {
    /// alias global connect kamp-connect
    #[argh(switch, short = 'a')]
    pub alias: bool,

    /// inject 'export VAR=VALUE' into the kamp-connect
    #[argh(option, short = 'e')]
    pub export: Vec<KeyValue>,
}

/// show execution context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ctx")]
pub(crate) struct CtxOptions {}

/// attach to a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "attach")]
pub(crate) struct AttachOptions {}

/// edit a file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "edit")]
pub(crate) struct EditOptions {
    /// path to file
    #[argh(positional, arg_name = "file")]
    pub files: Vec<String>,
}

/// get state from a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "get")]
pub(crate) struct GetOptions {
    /// quoting style (raw|kakoune|shell), default is raw
    #[argh(option, default = r#"String::from("raw")"#)]
    pub quoting: String,

    #[argh(subcommand)]
    pub subcommand: GetSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub(crate) enum GetSubCommand {
    Val(ValueName),
    Opt(OptionName),
    Reg(RegisterName),
    Shell(ShellCmdName),
}

/// value name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "val")]
pub(crate) struct ValueName {
    /// value name
    #[argh(positional)]
    pub name: String,
}

/// option name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "opt")]
pub(crate) struct OptionName {
    /// option name
    #[argh(positional)]
    pub name: String,
}

/// register name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "reg")]
pub(crate) struct RegisterName {
    /// register name
    #[argh(positional)]
    pub name: String,
}

/// shell command
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "sh")]
pub(crate) struct ShellCmdName {
    /// shell command
    #[argh(positional, arg_name = "command")]
    pub name: String,
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
            .ok_or("invalid KEY=VALUE pair".into())
            .and_then(|kv| {
                if kv.0.is_empty() || kv.0.contains(' ') {
                    Err("invalid key format".into())
                } else {
                    Ok(kv)
                }
            })
            .map(|(key, value)| KeyValue {
                key: key.into(),
                value: value.trim_matches(|c| c == '\'' || c == '"').into(),
            })
    }
}

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
    Send(SendOptions),
    List(ListOptions),
    Get(GetOptions),
    Cat(CatOptions),
    Ctx(CtxOptions),
    Version(VersionOptions),
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

/// display version
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "version")]
pub(crate) struct VersionOptions {}

/// attach to a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "attach")]
pub(crate) struct AttachOptions {
    /// switch to buffer
    #[argh(option, short = 'b')]
    pub buffer: Option<String>,
}

/// list sessions
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
pub(crate) struct ListOptions {
    /// switch to buffer
    #[argh(switch, short = 'a')]
    pub all: bool,
}

/// edit a file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "edit")]
pub(crate) struct EditOptions {
    /// path to file
    #[argh(positional)]
    pub file: String,

    /// optional coordinates
    #[argh(positional, arg_name = "+<line>[:<col>]")]
    pub coordinates: Option<String>,
}

/// send command to a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "send")]
pub(crate) struct SendOptions {
    /// buffer context
    #[argh(option, short = 'b', arg_name = "buffer")]
    pub buffers: Vec<String>,

    /// command to send
    #[argh(positional)]
    pub command: String,
}

/// get state from a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "get")]
pub(crate) struct GetOptions {
    /// raw output
    #[argh(switch, short = 'r')]
    pub raw: u8,

    /// buffer context, repeat for several buffers
    /// or * for all non-debug buffers
    #[argh(option, short = 'b', arg_name = "buffer")]
    pub buffers: Vec<String>,

    #[argh(subcommand)]
    pub subcommand: GetSubCommand,
}

/// print buffer content
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "cat")]
pub(crate) struct CatOptions {
    /// buffer context
    #[argh(option, short = 'b', arg_name = "buffer")]
    pub buffers: Vec<String>,
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
            .split_once('=')
            .ok_or_else(|| "invalid KEY=VALUE pair".into())
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

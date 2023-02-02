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

    /// display version and exit
    #[argh(switch, short = 'v')]
    pub version: bool,

    #[argh(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub(super) enum SubCommand {
    Init(InitOptions),
    Attach(AttachOptions),
    Edit(EditOptions),
    Send(SendOptions),
    Kill(KillOptions),
    List(ListOptions),
    Get(GetOptions),
    Cat(CatOptions),
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

/// attach to a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "attach")]
pub(crate) struct AttachOptions {
    /// switch to buffer
    #[argh(option, short = 'b')]
    pub buffer: Option<String>,
}

/// list session(s)
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
pub(crate) struct ListOptions {
    /// all sessions
    #[argh(switch, short = 'a')]
    pub all: bool,
}

/// kill session
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "kill")]
pub(crate) struct KillOptions {
    /// exit status
    #[argh(positional)]
    pub exit_status: Option<i32>,
}

/// edit a file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "edit")]
pub(crate) struct EditOptions {
    /// focus client
    #[argh(switch, short = 'f')]
    pub focus: bool,

    /// path to file
    #[argh(positional, greedy, arg_name = "file")]
    pub files: Vec<String>,
}

/// send command to a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "send")]
pub(crate) struct SendOptions {
    /// buffer context
    /// or '*' for all non-debug buffers
    #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
    pub buffers: Vec<String>,

    /// command to send
    #[argh(positional, greedy)]
    pub command: Vec<String>,
}

/// get state from a session in context
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "get")]
pub(crate) struct GetOptions {
    #[argh(subcommand)]
    pub subcommand: GetSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub(crate) enum GetSubCommand {
    Val(ValueName),
    Opt(OptionName),
    Reg(RegisterName),
    Shell(ShellCommand),
}

/// value name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "val")]
pub(crate) struct ValueName {
    /// buffer context
    /// or '*' for all non-debug buffers
    #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
    pub buffers: Vec<String>,

    /// quoting style kakoune, discards any --split
    #[argh(switch, short = 'q')]
    pub quote: bool,

    /// split by new line, for example 'buflist' value
    #[argh(switch, short = 's')]
    pub split: bool,

    /// split by null character
    #[argh(switch, short = 'z')]
    pub zplit: bool,

    /// value name to query
    #[argh(positional)]
    pub name: String,
}

/// option name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "opt")]
pub(crate) struct OptionName {
    /// buffer context
    /// or '*' for all non-debug buffers
    #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
    pub buffers: Vec<String>,

    /// quoting style kakoune, discards any --split
    #[argh(switch, short = 'q')]
    pub quote: bool,

    /// split by new line, for example 'str-list' type option
    #[argh(switch, short = 's')]
    pub split: bool,

    /// split by null character
    #[argh(switch, short = 'z')]
    pub zplit: bool,

    /// option name to query
    #[argh(positional)]
    pub name: String,
}

/// register name
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "reg")]
pub(crate) struct RegisterName {
    /// quoting style kakoune, discards any --split
    #[argh(switch, short = 'q')]
    pub quote: bool,

    /// split by new line, for example ':' register
    #[argh(switch, short = 's')]
    pub split: bool,

    /// split by null character
    #[argh(switch, short = 'z')]
    pub zplit: bool,

    /// register name to query, " is default
    #[argh(positional, default = r#"String::from("dquote")"#)]
    pub name: String,
}

/// shell command
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "sh")]
pub(crate) struct ShellCommand {
    /// buffer context
    /// or '*' for all non-debug buffers
    #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
    pub buffers: Vec<String>,

    /// shell command to evaluate
    #[argh(positional, greedy)]
    pub command: Vec<String>,
}

/// print buffer content
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "cat")]
pub(crate) struct CatOptions {
    /// buffer context
    /// or '*' for all non-debug buffers
    #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
    pub buffers: Vec<String>,
}

#[derive(PartialEq, Eq, Debug)]
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

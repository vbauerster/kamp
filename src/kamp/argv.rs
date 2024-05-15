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
    Init(init::Options),
    Attach(attach::Options),
    Edit(edit::Options),
    Send(send::Options),
    Kill(kill::Options),
    List(list::Options),
    Get(get::Options),
    Cat(cat::Options),
}

pub(super) mod init {
    use super::*;
    /// kakoune init
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "init")]
    pub struct Options {
        /// alias global connect kamp-connect
        #[argh(switch, short = 'a')]
        pub alias: bool,

        /// inject 'export VAR=VALUE' into the kamp-connect
        #[argh(option, short = 'e')]
        pub export: Vec<KeyValue>,
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
}

mod attach {
    use super::*;
    /// attach to a session in context
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "attach")]
    pub struct Options {
        /// switch to buffer
        #[argh(option, short = 'b')]
        pub buffer: Option<String>,
    }
}

mod edit {
    use super::*;
    /// edit a file
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "edit")]
    pub struct Options {
        /// focus client
        #[argh(switch, short = 'f')]
        pub focus: bool,

        /// path to file
        #[argh(positional, arg_name = "file")]
        pub files: Vec<String>,
    }
}

mod send {
    use super::*;
    /// send command to a session in context
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "send")]
    pub struct Options {
        /// buffer context
        /// or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,

        /// command to send
        #[argh(positional, greedy)]
        pub command: Vec<String>,
    }
}

mod kill {
    use super::*;
    /// kill session
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "kill")]
    pub struct Options {
        /// exit status
        #[argh(positional)]
        pub exit_status: Option<i32>,
    }
}

mod list {
    use super::*;
    /// list session(s)
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "list")]
    pub struct Options {
        /// all sessions
        #[argh(switch, short = 'a')]
        pub all: bool,
    }
}

pub(super) mod get {
    use super::*;
    /// get state from a session in context
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "get")]
    pub struct Options {
        #[argh(subcommand)]
        pub subcommand: SubCommand,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    pub enum SubCommand {
        Val(ValueName),
        Opt(OptionName),
        Reg(RegisterName),
        Shell(ShellCommand),
    }

    /// value name
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "val")]
    pub struct ValueName {
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
    pub struct OptionName {
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
    pub struct RegisterName {
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
    pub struct ShellCommand {
        /// buffer context
        /// or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,

        /// shell command to evaluate
        #[argh(positional, greedy)]
        pub command: Vec<String>,
    }
}

mod cat {
    use super::*;
    /// print buffer content
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "cat")]
    pub struct Options {
        /// buffer context
        /// or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,
    }
}

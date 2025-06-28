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

    /// print version and exit
    #[argh(switch, short = 'v')]
    pub version: bool,

    /// print command passed to kakoune
    #[argh(switch, short = 'd')]
    pub debug: bool,

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
    Ctx(ctx::Options),
}

pub(super) mod init {
    use super::*;
    /// Print init script for kakrc.
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
                .map(|(key, value)| KeyValue {
                    key: key.trim().into(),
                    value: value.trim_matches(|c| c == '\'' || c == '"').into(),
                })
                .ok_or_else(|| "invalid KEY=VALUE pair".into())
        }
    }
}

mod attach {
    use super::*;
    /// Attach to a session in context by creating new client.
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
    /// Edit a file in context client or create new one if
    /// client isn't specified or -n switch is provided.
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "edit")]
    pub struct Options {
        /// create new client and wait for it to exit
        #[argh(switch, short = 'n')]
        pub new: bool,

        /// focus client in context
        #[argh(switch, short = 'f')]
        pub focus: bool,

        /// path to file
        #[argh(positional, arg_name = "file")]
        pub files: Vec<String>,
    }
}

mod send {
    use super::*;
    /// Send command to a session in context.
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "send")]
    pub struct Options {
        /// do not parse/escape command
        #[argh(switch, short = 'v')]
        pub verbatim: bool,

        /// buffer context or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,

        /// command to send
        #[argh(positional, greedy)]
        pub command: Vec<String>,
    }
}

mod kill {
    use super::*;
    /// Kill a session in context.
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
    /// List a session in context.
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
    /// Get state from a session in context.
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "get")]
    pub struct Options {
        /// buffer context or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,

        /// split by null character instead of new line
        #[argh(switch, short = 'z')]
        pub zplit: bool,

        #[argh(subcommand)]
        pub subcommand: SubCommand,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    pub enum SubCommand {
        Value(value::Options),
        Option(option::Options),
        Register(register::Options),
        Shell(shell::Options),
    }

    #[derive(PartialEq, Debug)]
    pub enum QuotingMethod {
        Raw,
        Kakoune,
        Shell,
    }

    impl argh::FromArgValue for QuotingMethod {
        fn from_arg_value(value: &str) -> Result<Self, String> {
            Ok(match value {
                "raw" => QuotingMethod::Raw,
                "kakoune" => QuotingMethod::Kakoune,
                "shell" => QuotingMethod::Shell,
                _ => {
                    return Err("expected one of 'raw', 'kakoune', 'shell'".to_owned());
                }
            })
        }
    }

    mod value {
        use super::*;
        /// Get value as %val<name>.
        #[derive(FromArgs, PartialEq, Debug)]
        #[argh(subcommand, name = "val")]
        pub struct Options {
            /// quote output (raw|kakoune|shell) default=kakoune
            #[argh(option, short = 'q', default = "QuotingMethod::Kakoune")]
            pub quoting: QuotingMethod,

            /// split list type value like buflist or client_list
            #[argh(switch, short = 'l')]
            pub list: bool,

            /// don't parse output
            #[argh(switch, short = 'v')]
            pub verbatim: bool,

            /// value name to query (required)
            #[argh(positional)]
            pub name: String,
        }
    }

    mod option {
        use super::*;
        /// Get option as %opt<name>.
        #[derive(FromArgs, PartialEq, Debug)]
        #[argh(subcommand, name = "opt")]
        pub struct Options {
            /// quote output (raw|kakoune|shell) default=kakoune
            #[argh(option, short = 'q', default = "QuotingMethod::Kakoune")]
            pub quoting: QuotingMethod,

            /// split list type value like ui_options
            #[argh(switch, short = 'l')]
            pub list: bool,

            /// don't unquote output
            #[argh(switch, short = 'v')]
            pub verbatim: bool,

            /// option name to query (required)
            #[argh(positional)]
            pub name: String,
        }
    }

    mod register {
        use super::*;
        /// get register as %reg<name>
        #[derive(FromArgs, PartialEq, Debug)]
        #[argh(subcommand, name = "reg")]
        pub struct Options {
            /// quote output (raw|kakoune|shell) default=kakoune
            #[argh(option, short = 'q', default = "QuotingMethod::Kakoune")]
            pub quoting: QuotingMethod,

            /// split list type register like colon or slash
            #[argh(switch, short = 'l')]
            pub list: bool,

            /// don't parse output
            #[argh(switch, short = 'v')]
            pub verbatim: bool,

            /// register name to query, " is default
            #[argh(positional, default = r#"String::from("dquote")"#)]
            pub name: String,
        }
    }

    mod shell {
        use super::*;
        /// Evaluate shell command as %sh<command>.
        #[derive(FromArgs, PartialEq, Eq, Debug)]
        #[argh(subcommand, name = "sh")]
        pub struct Options {
            /// do not parse/escape command
            #[argh(switch, short = 'v')]
            pub verbatim: bool,

            /// shell command to evaluate
            #[argh(positional, greedy)]
            pub command: Vec<String>,
        }
    }
}

mod cat {
    use super::*;
    /// Print buffer content.
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "cat")]
    pub struct Options {
        /// buffer context or '*' for all non-debug buffers
        #[argh(option, short = 'b', long = "buffer", arg_name = "buffer")]
        pub buffers: Vec<String>,
    }
}

mod ctx {
    use super::*;
    /// Print session context (default).
    #[derive(FromArgs, PartialEq, Debug, Default)]
    #[argh(subcommand, name = "ctx")]
    pub struct Options {
        /// check client if none exit with error
        #[argh(switch, short = 'c')]
        pub client: bool,
    }
}

use crate::argv::get::{QuotingMethod, SubCommand};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) struct QueryContext {
    pub key_val: QueryKeyVal,
    pub qtype: QueryType,
    pub quoting: Quoting,
    pub verbatim: bool,
}

impl QueryContext {
    pub fn new(key_val: QueryKeyVal, qtype: QueryType, quoting: Quoting, verbatim: bool) -> Self {
        QueryContext {
            key_val,
            qtype,
            quoting,
            verbatim,
        }
    }
    pub fn new_sh(command: Vec<String>, verbatim: bool) -> Self {
        QueryContext {
            key_val: QueryKeyVal::Shell((command, verbatim)),
            qtype: QueryType::Plain,
            quoting: Quoting::Raw,
            verbatim: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum QueryKeyVal {
    Val(String),
    Opt(String),
    Reg(String),
    Shell((Vec<String>, bool)),
}

impl From<SubCommand> for QueryContext {
    fn from(value: SubCommand) -> QueryContext {
        match value {
            SubCommand::Value(o) => QueryContext::new(
                QueryKeyVal::Val(o.name),
                QueryType::new(o.list, o.map_key),
                o.quoting.into(),
                o.verbatim,
            ),
            SubCommand::Option(o) => QueryContext::new(
                QueryKeyVal::Opt(o.name),
                QueryType::new(o.list, o.map_key),
                o.quoting.into(),
                o.verbatim,
            ),
            SubCommand::Register(o) => QueryContext::new(
                QueryKeyVal::Reg(o.name),
                QueryType::new(o.list, None),
                o.quoting.into(),
                o.verbatim,
            ),
            SubCommand::Shell(o) => QueryContext::new_sh(o.command, o.verbatim),
        }
    }
}

impl Display for QueryKeyVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryKeyVal::Val(v) => write!(f, "%val<{v}>"),
            QueryKeyVal::Opt(v) => write!(f, "%opt<{v}>"),
            QueryKeyVal::Reg(v) => write!(f, "%reg<{v}>"),
            QueryKeyVal::Shell((v, verbatim)) => {
                let body = if *verbatim {
                    v.join(" ")
                } else {
                    v.iter().fold(String::new(), |mut buf, x| {
                        if !buf.is_empty() {
                            buf.push(' ');
                        }
                        if x.contains([' ', '"', '\'']) {
                            let s = x.replace("'", "''");
                            buf.push('\'');
                            buf.push_str(&s);
                            buf.push('\'');
                        } else if x.is_empty() {
                            buf.push('\'');
                            buf.push_str(x);
                            buf.push('\'');
                        } else {
                            buf.push_str(x);
                        }
                        buf
                    })
                };
                write!(f, "%sh<{body}>")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum QueryType {
    Plain,
    List,
    Map(String),
}

impl QueryType {
    pub fn new(list: bool, map: Option<String>) -> Self {
        if let Some(key) = map {
            QueryType::Map(key)
        } else if list {
            QueryType::List
        } else {
            QueryType::Plain
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Quoting {
    Raw,
    Kakoune,
    Shell,
}

impl From<QuotingMethod> for Quoting {
    fn from(value: QuotingMethod) -> Self {
        match value {
            QuotingMethod::Raw => Quoting::Raw,
            QuotingMethod::Kakoune => Quoting::Kakoune,
            QuotingMethod::Shell => Quoting::Shell,
        }
    }
}

impl Display for Quoting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quoting::Raw => write!(f, "raw"),
            Quoting::Kakoune => write!(f, "kakoune"),
            Quoting::Shell => write!(f, "shell"),
        }
    }
}

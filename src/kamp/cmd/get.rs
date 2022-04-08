use std::borrow::Cow;

use crate::kamp::argv::GetSubCommand;

use super::Context;
use super::Error;

pub(crate) enum Get<'a> {
    Val(Cow<'a, str>),
    Opt(Cow<'a, str>),
    Reg(Cow<'a, str>),
    Shell(Cow<'a, str>),
}

impl<'a> Get<'a> {
    pub(crate) fn new_val(s: impl Into<Cow<'a, str>>) -> Self {
        Get::Val(s.into())
    }
    pub(crate) fn new_opt(s: impl Into<Cow<'a, str>>) -> Self {
        Get::Opt(s.into())
    }
    pub(crate) fn new_reg(s: impl Into<Cow<'a, str>>) -> Self {
        Get::Reg(s.into())
    }
    pub(crate) fn new_sh(s: impl Into<Cow<'a, str>>) -> Self {
        Get::Shell(s.into())
    }
}

impl From<GetSubCommand> for Get<'_> {
    fn from(argv_cmd: GetSubCommand) -> Self {
        use GetSubCommand::*;
        match argv_cmd {
            Val(opt) => Get::new_val(opt.name),
            Opt(opt) => Get::new_opt(opt.name),
            Reg(opt) => Get::new_reg(opt.name),
            Shell(opt) => Get::new_sh(opt.name),
        }
    }
}

impl Get<'_> {
    pub fn run(&self, ctx: &Context, rawness: u8, buffer: Option<String>) -> Result<String, Error> {
        let mut buf = String::from("echo -quoting ");
        match rawness {
            0 | 1 => buf.push_str("kakoune"),
            _ => buf.push_str("raw"),
        }
        buf.push_str(" -to-file %opt{kamp_out} %");
        match self {
            Get::Val(name) => {
                buf.push_str("val{");
                buf.push_str(name);
            }
            Get::Opt(name) => {
                buf.push_str("opt{");
                buf.push_str(name);
            }
            Get::Reg(name) => {
                buf.push_str("reg{");
                buf.push_str(name);
            }
            Get::Shell(name) => {
                buf.push_str("sh{");
                buf.push_str(name);
            }
        }
        buf.push('}');
        let res = ctx.send(&buf, buffer);
        if rawness == 0 {
            res.map(|s| {
                s.split('\'').filter(|&s| !s.trim().is_empty()).fold(
                    String::new(),
                    |mut buf, next| {
                        buf.push_str(next);
                        buf.push('\n');
                        buf
                    },
                )
            })
        } else {
            res
        }
    }
}

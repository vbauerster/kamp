use crate::kamp::argv::GetSubCommand;

use super::Context;
use super::Error;

pub(crate) enum Get {
    Val(String),
    Opt(String),
    Reg(String),
    Shell(String),
}

impl From<GetSubCommand> for Get {
    fn from(argv_cmd: GetSubCommand) -> Self {
        use GetSubCommand::*;
        match argv_cmd {
            Val(opt) => Get::Val(opt.name),
            Opt(opt) => Get::Opt(opt.name),
            Reg(opt) => Get::Reg(opt.name),
            Shell(opt) => Get::Shell(opt.name),
        }
    }
}

impl Get {
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

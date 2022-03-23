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
    pub fn run(
        &self,
        ctx: Context,
        raw: bool,
        buffers: Option<Vec<String>>,
    ) -> Result<String, Error> {
        let mut buf = String::from("echo -quoting kakoune -to-file %opt{kamp_out} %");
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
        buf.push_str("}");
        let res = ctx.send(&buf, buffers.map(super::to_csv_buffers).flatten());
        if raw {
            res
        } else {
            res.map(|s| {
                s.split("'").filter(|&s| !s.trim().is_empty()).fold(
                    String::new(),
                    |mut buf, next| {
                        buf.push_str(next);
                        buf.push_str("\n");
                        buf
                    },
                )
            })
        }
    }
}

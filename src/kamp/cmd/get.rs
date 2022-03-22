use crate::argv::GetSubCommand;

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
    pub fn run(&self, ctx: Context, quoting: &str, buffers: Vec<String>) -> Result<String, Error> {
        let mut buf = String::from("  echo -quoting ");
        buf.push_str(quoting);
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
        buf.push_str("}\n");
        ctx.send(&buf, to_csv_buffers(buffers))
    }
}

fn to_csv_buffers(buffers: Vec<String>) -> Option<String> {
    let buffers = buffers.into_iter().filter(|s| s != "*").collect::<Vec<_>>();
    if buffers.is_empty() {
        return None;
    }
    let mut res =
        buffers
            .iter()
            .take(buffers.len() - 1)
            .fold(String::from("'"), |mut buf, next| {
                buf.push_str(next);
                buf.push_str(",");
                buf
            });
    res.push_str(&buffers[buffers.len() - 1]);
    res.push_str("'");
    Some(res)
}

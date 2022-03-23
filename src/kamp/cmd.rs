mod attach;
mod cat;
mod edit;
mod get;
mod init;
mod send;

use super::Context;
use super::Error;

pub(super) use attach::attach;
pub(super) use cat::cat;
pub(super) use edit::edit;
pub(super) use get::Get;
pub(super) use init::init;
pub(super) use send::send;

fn to_csv_buffers(buffers: Vec<String>) -> Option<String> {
    if buffers.is_empty() {
        return None;
    }
    if buffers[0] == "*" {
        return Some("*".into());
    }
    let buffers = buffers.into_iter().filter(|s| s != "*").collect::<Vec<_>>();
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

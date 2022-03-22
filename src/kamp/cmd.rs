mod attach;
mod edit;
mod get;
mod init;
mod send;

use super::context::Context;
use super::error::Error;

pub(crate) use attach::attach;
pub(crate) use edit::edit;
pub(crate) use get::Get;
pub(crate) use init::init;
pub(crate) use send::send;

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

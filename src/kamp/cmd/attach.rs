use super::Context;
use super::Result;

pub(crate) fn attach(ctx: Context, buffer: Option<String>) -> Result<()> {
    let mut cmd = String::new();
    if let Some(buffer) = &buffer {
        cmd.push_str("buffer '");
        cmd.push_str(buffer);
        cmd.push('\'');
    }
    ctx.connect(cmd)
}

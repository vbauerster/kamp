use super::Context;
use super::Error;

pub(crate) fn attach(ctx: &Context, buffer: Option<String>) -> Result<(), Error> {
    let mut cmd = String::new();
    if let Some(buffer) = buffer {
        cmd.push_str(r#"eval %sh{ case "$kak_bufname" in (\*stdin*) echo delete-buffer ;; esac }"#);
        cmd.push('\n');
        cmd.push_str("buffer '");
        cmd.push_str(&buffer);
        cmd.push('\'');
    }
    ctx.connect(&cmd).map(|_| ())
}

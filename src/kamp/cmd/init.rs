use super::Result;
use crate::kamp::argv::KeyValue;
use std::fmt::Write;

const KAKOUNE_INIT: &str = r#"
define-command -hidden -override kamp-init %{
    declare-option -hidden str kamp_out
    declare-option -hidden str kamp_err
    evaluate-commands %sh{
        kamp_out="${TMPDIR:-/tmp/}kamp-${kak_session}.out"
        kamp_err="${TMPDIR:-/tmp/}kamp-${kak_session}.err"
        mkfifo "$kamp_out" "$kamp_err"
        echo "set-option global kamp_out '$kamp_out'"
        echo "set-option global kamp_err '$kamp_err'"
    }
}

define-command -hidden -override kamp-end %{
    nop %sh{ rm -f "$kak_opt_kamp_out" "$kak_opt_kamp_err" }
}

hook global KakBegin .* kamp-init
hook global KakEnd .* kamp-end
"#;

pub(crate) fn init(export: Vec<KeyValue>, alias: bool) -> Result<String> {
    let user_exports = export.into_iter().fold(String::new(), |mut buf, next| {
        buf.push_str("export ");
        buf.push_str(&next.key);
        buf.push_str("=\"");
        buf.push_str(&next.value);
        buf.push_str("\"\n");
        (0..8).for_each(|_| buf.push(' '));
        buf
    });

    let mut buf = String::new();

    #[rustfmt::skip]
    writeln!(&mut buf,
r#"define-command -override kamp-connect -params 1.. -command-completion %{{
    %arg{{1}} sh -c %{{
        {user_exports}export KAKOUNE_SESSION="$1"
        export KAKOUNE_CLIENT="$2"
        shift 3

        [ $# = 0 ] && set "$SHELL"

        "$@"
    }} -- %val{{session}} %val{{client}} %arg{{@}}
}} -docstring 'run Kakoune command in connected context'"#)?;

    buf.push_str(KAKOUNE_INIT);

    if alias {
        buf.push_str("alias global connect kamp-connect\n");
    }

    Ok(buf)
}

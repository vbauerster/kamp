const KAKOUNE_INIT: &str = r#"define-command -hidden kamp-init %{
    declare-option -hidden str kamp_out
    declare-option -hidden str kamp_err
    evaluate-commands %sh{
        kamp_out="${TMPDIR:-/tmp/}${kak_session}-kamp.out"
        kamp_err="${TMPDIR:-/tmp/}${kak_session}-kamp.err"
        mkfifo "$kamp_out" "$kamp_err"
        echo "set-option global kamp_out '$kamp_out'"
        echo "set-option global kamp_err '$kamp_err'"
    }
}

define-command -hidden kamp-end %{
    nop %sh{ rm -f "$kak_opt_kamp_out" "$kak_opt_kamp_err" }
}

hook global KakBegin .* kamp-init
hook global KakEnd .* kamp-end
"#;

pub(crate) fn init() {
    println!("{}", KAKOUNE_INIT);
}

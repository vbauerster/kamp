use std::{fmt::Display, mem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("missing closing quote")
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
enum State {
    Delimiter,
    Unquoted,
    Quoted,
}

/// Splits output produced with 'echo -quoting kakoune'
pub(super) fn split(s: &str) -> Result<Vec<String>, ParseError> {
    use State::*;

    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = s.chars();
    let mut state = Delimiter;

    loop {
        let c = chars.next();
        state = match state {
            Delimiter => match c {
                None => break,
                Some('\'') => Quoted,
                Some(_) => Delimiter,
            },
            Unquoted => match c {
                None => {
                    words.push(mem::take(&mut word));
                    break;
                }
                Some(c @ '\'') => {
                    word.push(c);
                    Quoted
                }
                Some(_) => {
                    words.push(mem::take(&mut word));
                    Delimiter
                }
            },
            Quoted => match c {
                None => return Err(ParseError),
                Some('\'') => Unquoted,
                Some(c) => {
                    word.push(c);
                    Quoted
                }
            },
        };
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_ok(cases: &[(&str, &[&str])]) {
        for &(input, expected) in cases {
            let actual = split(input).unwrap();
            assert!(
                expected == actual.as_slice(),
                "split({input:?}).unwrap()\nexpected: {expected:?}\n  actual: {actual:?}\n",
            );
        }
    }

    #[test]
    #[allow(unconditional_panic, unused_must_use)]
    #[should_panic(expected = "ParseError")]
    fn split_unterminated() {
        split_ok(&[("'", &[])]);
    }

    #[test]
    fn split_empty() {
        split_ok(&[("", &[])]);
        split_ok(&[(" ", &[])]);
        split_ok(&[("a", &[])]);
        split_ok(&[(" a", &[])]);
        split_ok(&[("ab", &[])]);
        split_ok(&[(" ab", &[])]);
        split_ok(&[("a b", &[])]);
        split_ok(&[(" a b", &[])]);
    }

    #[test]
    fn split_single() {
        split_ok(&[("''a", &[""])]);
        split_ok(&[("'' a", &[""])]);
        split_ok(&[("'a'", &["a"])]);
        split_ok(&[("'''''a'", &["''a"])]);
        split_ok(&[("'''a'''", &["'a'"])]);
        split_ok(&[(" '''a'''", &["'a'"])]);
        split_ok(&[("\n'''a'''", &["'a'"])]);
        split_ok(&[(r#"'echo "''ok''"'"#, &[r#"echo "'ok'""#])]);
    }

    #[test]
    fn split_multi() {
        split_ok(&[("'' ' '", &["", " "])]);
        split_ok(&[("'a' 'b'", &["a", "b"])]);
        split_ok(&[(
            "'echo ok''s' 'edit ''sp buf.txt'''",
            &["echo ok's", "edit 'sp buf.txt'"],
        )]);
        split_ok(&[(
            "'echo ok''s'\n'edit ''sp buf.txt'''",
            &["echo ok's", "edit 'sp buf.txt'"],
        )]);
    }
}

use std::{borrow::Cow, path::Path};

use super::{Context, Error, Result};

pub(crate) fn edit(ctx: Context, new: bool, focus: bool, files: Vec<String>) -> Result<bool> {
    let mut buf = String::new();
    let mut pair = [None; 2];
    let mut coord = None;
    let mut iter = files.iter();

    for (i, item) in iter.by_ref().take(2).enumerate() {
        if Path::new(item).is_file() || !item.starts_with('+') {
            pair[1 - i] = Some(item);
            continue;
        }
        if coord.is_some() {
            return Err(Error::UnexpectedCoordPosition(item.clone()));
        }
        coord = Some(parse(item)?);
    }

    for (i, item) in iter.rev().chain(pair.into_iter().flatten()).enumerate() {
        let path = {
            let path = Path::new(item);
            if path.is_relative() {
                Cow::Owned(path.canonicalize()?)
            } else {
                Cow::Borrowed(path)
            }
        };
        if let Some(p) = path.to_str() {
            if i != 0 {
                buf.push('\n');
            }
            buf.push_str("edit -existing ");
            if p.contains(' ') {
                buf.push_str(&p.replace(' ', "\\ "));
            } else {
                buf.push_str(p);
            }
        }
    }

    let scratch = if buf.is_empty() {
        buf.push_str("edit -scratch");
        true
    } else {
        if let Some(coord) = coord {
            for n in coord {
                buf.push(' ');
                buf.push_str(&n.to_string());
            }
        }
        false
    };

    if new || ctx.is_draft() {
        ctx.connect(buf).map(|_| scratch)
    } else {
        if focus {
            buf.push_str("\nfocus");
        }
        ctx.send(buf, None).map(|_| scratch)
    }
}

// assuming coord starts with '+'
fn parse(coord: &str) -> Result<Vec<i32>> {
    // parsing first value as '+n' so '+:<n>' will fail
    coord
        .splitn(2, ':')
        .take_while(|&s| !s.is_empty()) // make sure '+n:' is valid
        .map(|s| {
            s.parse().map_err(|e| Error::InvalidCoordinates {
                coord: String::from(coord),
                source: anyhow::Error::new(e),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_ok() -> Result<()> {
        assert_eq!(parse("+1")?, vec![1]);
        assert_eq!(parse("+1:")?, vec![1]);
        assert_eq!(parse("+1:1")?, vec![1, 1]);
        Ok(())
    }
    #[test]
    fn test_parse_err() {
        assert!(parse("+").is_err());
        assert!(parse("+:").is_err());
        assert!(parse("+:+").is_err());
        assert!(parse("+:1").is_err());
        assert!(parse("++").is_err());
        assert!(parse("++:").is_err());
        assert!(parse("++:1").is_err());
        assert!(parse("++1:").is_err());
        assert!(parse("++1:1").is_err());
        assert!(parse("+a").is_err());
        assert!(parse("+a:").is_err());
        assert!(parse("+a:1").is_err());
        assert!(parse("+1:a").is_err());
    }
}

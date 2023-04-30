use std::path::Path;
use std::path::PathBuf;

use super::{Context, Error, Result};

pub(crate) fn edit(ctx: Context, focus: bool, files: Vec<String>) -> Result<()> {
    let mut buf = String::new();
    let mut pair = [None; 2];
    let mut coord = None;
    let mut iter = files.iter();

    for (i, item) in iter.by_ref().take(2).enumerate() {
        if Path::new(item).exists() || !item.starts_with('+') {
            pair[1 - i] = Some(item);
            continue;
        }
        if coord.is_none() {
            coord = Some(parse(item)?);
        } else {
            return Err(Error::InvalidCoordinates {
                coord: item.clone(),
                source: anyhow::Error::msg("invalid position"),
            });
        }
    }

    for (i, file) in iter.rev().chain(pair.into_iter().flatten()).enumerate() {
        let p = std::fs::canonicalize(file).unwrap_or_else(|_| PathBuf::from(file));
        if let Some(p) = p.as_path().to_str() {
            if i != 0 {
                buf.push('\n');
            }
            buf.push_str("edit -existing '");
            if p.contains('\'') {
                buf.push_str(&p.replace('\'', "''"));
            } else {
                buf.push_str(p);
            }
            buf.push('\'');
        }
    }

    if let Some(v) = coord {
        for item in v {
            buf.push(' ');
            buf.push_str(&item.to_string());
        }
    }

    if buf.is_empty() {
        buf.push_str("edit -scratch");
    }

    if ctx.is_draft() {
        ctx.connect(&buf) // this one acts like attach
    } else {
        if focus {
            buf.push_str("\nfocus");
        }
        ctx.send(&buf, None).map(drop)
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

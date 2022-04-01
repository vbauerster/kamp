use std::fmt::Write;
use std::fs::canonicalize;
use std::num::ParseIntError;
use std::path::PathBuf;

use super::Context;
use super::Error;

type ParseResult<T> = Result<T, ParseIntError>;

pub(crate) fn edit(ctx: &Context, file: String, coord: Option<String>) -> Result<(), Error> {
    let mut buf = String::from("edit ");
    let (file, coord) = check_both(file, coord);
    if file.starts_with('+') {
        buf.push_str("-scratch");
    } else {
        let p = canonicalize(&file).unwrap_or_else(|_| PathBuf::from(&file));
        write!(&mut buf, "-existing '{}'", p.display())?;
        if let Some(coord) = coord {
            let v = coord.map_err(|source| Error::InvalidCoordinates { source })?;
            for coord in v {
                write!(&mut buf, " {}", coord)?;
            }
        }
    }
    if ctx.client.is_some() {
        ctx.send(&buf, None).map(|_| ())
    } else {
        ctx.connect(&buf)
    }
}

fn check_both(file: String, coord: Option<String>) -> (String, Option<ParseResult<Vec<i32>>>) {
    let res = coord.as_deref().and_then(parse);
    if res.is_none() && file.starts_with('+') {
        if let Some(coord) = coord {
            return check_both(coord, Some(file));
        }
    }
    (file, res)
}

fn parse(coord: &str) -> Option<ParseResult<Vec<i32>>> {
    if !coord.starts_with('+') {
        return None;
    }
    // parsing first value as '+n' so '+:<n>' will fail
    let res = coord
        .splitn(2, ':')
        .take_while(|&s| !s.is_empty()) // make sure '+n:' is valid
        .map(|s| s.parse())
        .collect::<ParseResult<Vec<_>>>();
    Some(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_both_valid_file_and_some_not_valid_coord() {
        assert_eq!(
            (String::from("file1"), None),
            check_both(String::from("file1"), Some(String::from("file2")))
        );
    }

    #[test]
    fn check_both_coord_none() {
        assert_eq!(
            (String::from("file"), None),
            check_both(String::from("file"), None)
        );
    }

    #[test]
    fn check_both_invalid_coord_full() {
        let (_, coord) = check_both(String::new(), Some(String::from("+a:b")));
        assert!(coord.unwrap().is_err());
    }

    #[test]
    fn check_both_invalid_coord_line() {
        let (_, coord) = check_both(String::new(), Some(String::from("+a:4")));
        assert!(coord.unwrap().is_err());
    }

    #[test]
    fn check_both_invalid_coord_col() {
        let (_, coord) = check_both(String::new(), Some(String::from("+12:a")));
        assert!(coord.unwrap().is_err());
    }

    #[test]
    fn check_both_coord_full() {
        let (_, coord) = check_both(String::new(), Some(String::from("+12:4")));
        assert_eq!(Some(Ok(vec![12, 4])), coord);
    }

    #[test]
    fn check_both_coord_line() {
        let (_, coord) = check_both(String::new(), Some(String::from("+12")));
        assert_eq!(Some(Ok(vec![12])), coord);
    }

    #[test]
    fn check_both_coord_line_with_ending_colon() {
        let (_, coord) = check_both(String::new(), Some(String::from("+12:")));
        assert_eq!(Some(Ok(vec![12])), coord);
    }

    #[test]
    fn check_both_coord_full_reversed() {
        let expected_file = "file";
        let (file, coord) = check_both(String::from("+12:4"), Some(String::from(expected_file)));
        assert_eq!(expected_file, file.as_str());
        assert_eq!(Some(Ok(vec![12, 4])), coord);
    }
}

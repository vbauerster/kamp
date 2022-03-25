use std::fmt::Write;
use std::fs::canonicalize;
use std::path::PathBuf;

use super::Context;
use super::Error;

pub(crate) fn edit(ctx: &Context, file: String, coordinates: Option<String>) -> Result<(), Error> {
    let mut buf = String::from("edit -existing ");
    let p = canonicalize(&file).unwrap_or_else(|_| PathBuf::from(&file));
    write!(&mut buf, "'{}'", p.display())?;
    let res = coordinates.as_deref().and_then(parse);
    if let Some(res) = res {
        let v = res.map_err(|source| Error::InvalidCoordinates { source })?;
        for coord in v {
            write!(&mut buf, " {}", coord)?;
        }
    }
    // eprintln!("edit: {:?}", buf);
    if ctx.client.is_some() {
        ctx.send(&buf, None).map(|_| ())
    } else {
        ctx.connect(&buf)
    }
}

type ParseResult<T> = Result<T, std::num::ParseIntError>;

fn parse(coordinates: &str) -> Option<ParseResult<Vec<i32>>> {
    if !coordinates.starts_with('+') {
        return None;
    }
    // parsing '+n' is ok so no need to slice &coordinates[1..]
    let res = coordinates
        .splitn(2, ':')
        .map(|s| s.parse())
        .collect::<ParseResult<Vec<_>>>();
    Some(res)
}

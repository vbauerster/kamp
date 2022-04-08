use std::fmt::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;

use super::Context;
use super::Error;

pub(crate) fn edit(ctx: &Context, files: Vec<String>) -> Result<(), Error> {
    let mut buf = String::new();
    let mut coord_buf = String::new();
    let mut tmp = Vec::new();
    let mut i = 0;

    for item in files.iter().take(2) {
        i += 1;
        if !item.starts_with('+') {
            tmp.push(item);
            continue;
        }
        match parse(item) {
            Err(source) => {
                if Path::new(item).exists() {
                    tmp.push(item);
                } else {
                    return Err(Error::InvalidCoordinates {
                        coord: item.clone(),
                        source,
                    });
                }
            }
            Ok(v) if coord_buf.is_empty() => {
                for item in v {
                    write!(&mut coord_buf, " {}", item)?;
                }
            }
            Ok(_) => tmp.push(item),
        }
    }

    for file in files[i..].iter().rev().chain(tmp.into_iter().rev()) {
        let p = std::fs::canonicalize(file).unwrap_or_else(|_| PathBuf::from(file));
        writeln!(&mut buf, "edit -existing '{}'", p.display())?;
    }
    buf.pop();
    buf.push_str(&coord_buf);

    if buf.is_empty() {
        buf.push_str("edit -scratch");
    }

    if ctx.client.is_none() {
        ctx.connect(&buf) // this one acts like attach
    } else {
        buf.push_str("\nfocus");
        ctx.send(&buf, None).map(|_| ())
    }
}

// prerequisite: coord should start with '+'
fn parse(coord: &str) -> Result<Vec<i32>, ParseIntError> {
    // parsing first value as '+n' so '+:<n>' will fail
    coord
        .splitn(2, ':')
        .take_while(|&s| !s.is_empty()) // make sure '+n:' is valid
        .map(|s| s.parse())
        .collect()
}

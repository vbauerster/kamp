use std::fmt::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;

use super::Context;
use super::Error;

pub(crate) fn edit(ctx: &Context, mut files: Vec<String>) -> Result<(), Error> {
    let mut buf = String::new();
    let mut append_buf = String::new();
    let mut tmp = Vec::new();
    files.reverse();

    for i in 0..2 {
        match files.pop() {
            Some(coord) if coord.starts_with('+') => {
                match parse(&coord) {
                    Err(source) => {
                        if Path::new(&coord).exists() {
                            tmp.push(coord);
                        } else {
                            return Err(Error::InvalidCoordinates { coord, source });
                        }
                    }
                    Ok(v) => {
                        for coord in v {
                            write!(&mut append_buf, " {}", coord)?;
                        }
                    }
                }
            }
            Some(file) => {
                tmp.push(file);
            }
            None => {
                if i == 0 {
                    append_buf.push_str("edit -scratch");
                    break;
                }
            }
        }
    }
    files.extend(tmp.into_iter().rev());
    for file in &files {
        let p = std::fs::canonicalize(file).unwrap_or_else(|_| PathBuf::from(file));
        writeln!(&mut buf, "edit -existing '{}'", p.display())?;
    }
    buf.pop();
    buf.push_str(&append_buf);
    if ctx.client.is_some() {
        ctx.send(&buf, None).map(|_| ())
    } else {
        ctx.connect(&buf)
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

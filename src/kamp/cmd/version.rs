use super::Error;
use std::fmt::Write;

pub(crate) fn version() -> Result<String, Error> {
    let mut buf = String::new();
    writeln!(
        &mut buf,
        "{} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;
    Ok(buf)
}

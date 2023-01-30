mod kamp;

use anyhow::{Error, Result};

fn main() -> Result<()> {
    kamp::run().map_err(Error::new)
}

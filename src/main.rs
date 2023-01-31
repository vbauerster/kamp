mod kamp;

use anyhow::Result;

fn main() -> Result<()> {
    kamp::run().map_err(From::from)
}

mod kamp;

use anyhow::Result;

fn main() -> Result<()> {
    let res = kamp::run()?;
    if let Some(res) = res {
        print!("{}", res);
    }
    Ok(())
}

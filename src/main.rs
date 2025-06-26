mod argv;
mod kamp;

fn main() -> anyhow::Result<()> {
    kamp::run().map_err(From::from)
}

use anyhow::Result;
use std::env;

const KAKOUNE_SESSION: &'static str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &'static str = "KAKOUNE_CLIENT";

#[derive(Debug)]
pub struct Context {
    pub session: String,
    pub client: Option<String>,
}

pub fn get() -> Result<Context, super::CommandError> {
    let session = env::var(KAKOUNE_SESSION)?;
    let client = env::var(KAKOUNE_CLIENT).ok();
    Ok(Context { session, client })
}

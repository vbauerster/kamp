mod env;

pub(super) use env::Env;

const KAKOUNE_SESSION: &str = "KAKOUNE_SESSION";
const KAKOUNE_CLIENT: &str = "KAKOUNE_CLIENT";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession(#[from] std::env::VarError),
}

#[derive(Debug)]
pub struct Context {
    pub session: String,
    pub client: Option<String>,
}

impl Context {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Context {
            session: std::env::var(KAKOUNE_SESSION)?,
            client: std::env::var(KAKOUNE_CLIENT).ok(),
        })
    }
}

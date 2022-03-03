pub mod env;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("no session in context")]
    EnvError(#[from] std::env::VarError),
}

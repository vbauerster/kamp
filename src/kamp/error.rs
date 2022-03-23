#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession,

    #[error("kak exited with error: {0}")]
    KakProcess(std::process::ExitStatus),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("invalid session: {0}")]
    InvalidSession(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

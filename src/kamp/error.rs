#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession,

    #[error("invalid session: {0}")]
    InvalidSession(String),

    #[error("invalid context: either client or buffer is required")]
    InvalidContext,

    #[error("kak exited with error: {0}")]
    KakProcess(std::process::ExitStatus),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

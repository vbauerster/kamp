#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no session in context")]
    NoSession,

    #[error("invalid session {session:?} kak exited with code: {exit_code}")]
    InvalidSession { session: String, exit_code: i32 },

    #[error("invalid context: either client or buffer is required")]
    InvalidContext,

    #[error("invalid coordinates: {coord:?}")]
    InvalidCoordinates {
        coord: String,
        source: std::num::ParseIntError,
    },

    #[error("kak exited with error: {0}")]
    KakProcess(std::process::ExitStatus),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

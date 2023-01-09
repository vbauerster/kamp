#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid context: {0}")]
    InvalidContext(&'static str),

    #[error("invalid session {session:?} kak exited with code: {exit_code}")]
    InvalidSession { session: String, exit_code: i32 },

    #[error("invalid coordinates: {coord:?}")]
    InvalidCoordinates {
        coord: String,
        source: anyhow::Error,
    },

    #[error("command is required")]
    CommandRequired,

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

pub type Result<T> = std::result::Result<T, Error>;

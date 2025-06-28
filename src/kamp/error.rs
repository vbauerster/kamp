#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid context: {0}")]
    InvalidContext(&'static str),

    #[error("command is required")]
    CommandRequired,

    #[error("kak exited with code: {0}")]
    KakUnexpectedExit(i32),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error("unexpected coordinates position: {0}")]
    UnexpectedCoordPosition(String),

    #[error("invalid coordinates: {coord:?}")]
    InvalidCoordinates {
        coord: String,
        source: anyhow::Error,
    },

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

pub type Result<T> = std::result::Result<T, Error>;

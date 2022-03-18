#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("kak exited with error: {0}")]
    KakProcess(std::process::ExitStatus),

    #[error("kak eval error: {0}")]
    KakEvalCatch(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

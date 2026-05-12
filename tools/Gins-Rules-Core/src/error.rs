use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("Deserialization Error: {0}")]
    Deserialization(String),

    #[error("Network Error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Compilation Error: {0}")]
    Compilation(String),
}

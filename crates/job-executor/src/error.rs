pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("docker api error: {0}")]
    Bollard(#[from] bollard::errors::Error),
    #[error("reporter error: {0}")]
    Reporter(#[from] Box<dyn std::error::Error + Send + Sync>),
}

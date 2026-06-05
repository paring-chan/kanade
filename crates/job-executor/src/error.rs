pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("docker api error: {0}")]
    Bollard(#[from] bollard::errors::Error),
}

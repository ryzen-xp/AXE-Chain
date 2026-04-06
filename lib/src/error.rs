use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoinError {
    #[error("Invalid transaction")]
    InvalidTransaction,

    #[error("Invalid block")]
    InvalidBlock,
}

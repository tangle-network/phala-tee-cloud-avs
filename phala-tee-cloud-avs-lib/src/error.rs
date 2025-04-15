use thiserror::Error;

/// Custom error type for the Phala AVS blueprint.
#[derive(Debug, Error)]
pub enum PhalaAvsError {
    #[error("EVM interaction error: {0}")]
    EvmError(String),

    #[error("TEE interaction error: {0}")]
    TeeError(String),

    #[error("Aggregator error: {0}")]
    AggregatorError(String),

    #[error("Task error: {0}")]
    TaskError(String),

    #[error("Keystore error: {0}")]
    KeystoreError(#[from] blueprint_sdk::keystore::Error),

    #[error("Cron parsing error: {0}")]
    CronError(#[from] cron::error::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

// Implement conversion from alloy RpcError if needed, for example:
// impl From<blueprint_sdk::alloy::rpc::RpcError<alloy_transport_http::HttpError>> for PhalaAvsError {
//     fn from(err: blueprint_sdk::alloy::rpc::RpcError<alloy_transport_http::HttpError>) -> Self {
//         PhalaAvsError::EvmError(format!("Alloy RPC Error: {}", err))
//     }
// }

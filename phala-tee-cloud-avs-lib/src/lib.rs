pub mod context;
pub mod error;
pub mod jobs;
pub mod tee;

// Re-export key types for easy access in the binary
use blueprint_sdk::{
    alloy::{
        primitives::{Address, address},
        sol,
    },
    std::env,
};
pub use context::PhalaAvsContext;
pub use error::PhalaAvsError;
pub use jobs::{
    HEARTBEAT_JOB_ID, RESPOND_TO_CHALLENGE_JOB_ID, heartbeat_job, respond_to_challenge_job,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
pub use tee::TeeHandler;

lazy_static! {
    pub static ref TASK_MANAGER_ADDRESS: Address = env::var("TASK_MANAGER_ADDRESS")
        .map(|addr| addr.parse().expect("Invalid TASK_MANAGER_ADDRESS"))
        .unwrap_or_else(|_| address!("0000000000000000000000000000000000000000"));
    pub static ref PRIVATE_KEY: String = env::var("PRIVATE_KEY").unwrap_or_else(|_| {
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });
    pub static ref AGGREGATOR_PRIVATE_KEY: String = env::var("PRIVATE_KEY").unwrap_or_else(|_| {
        "2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string()
    });
}

sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    ERC20,
    "../contracts/out/ERC20.sol/ERC20.json"
);

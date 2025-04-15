use crate::error::PhalaAvsError;
use crate::tee::TeeHandler;
use blueprint_sdk::{info, macros::context::KeystoreContext, runner::config::BlueprintEnvironment};

/// The context for the Phala Cloud AVS blueprint jobs.
///
/// This struct holds shared state accessible by all job handlers.
/// It must derive `Clone` because it's cloned for each job execution.
/// It derives `KeystoreContext` to access the keystore provided by the environment.
#[derive(Clone, KeystoreContext)]
pub struct PhalaAvsContext {
    /// The blueprint environment, providing access to configuration, keystore, etc.
    #[config]
    pub env: BlueprintEnvironment,

    /// Handler for interacting with the TEE component.
    pub tee_handler: TeeHandler,
    // Add other shared resources here, e.g.:
    // - EVM Provider/Client (if needed directly in jobs, though often passed via args)
    // - Database connection pool
    // - Metrics registry
}

impl PhalaAvsContext {
    /// Creates a new instance of the AVS context.
    pub async fn new(env: BlueprintEnvironment) -> Result<Self, PhalaAvsError> {
        info!("Creating PhalaAvsContext...");
        let tee_handler = TeeHandler::new().await?;
        Ok(Self {
            env,
            tee_handler,
            // Initialize other fields here
        })
    }
}

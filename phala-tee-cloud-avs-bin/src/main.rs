use blueprint_sdk::Router;
use blueprint_sdk::alloy::primitives::Address;
use blueprint_sdk::evm::producer::{PollingConfig, PollingProducer};
use blueprint_sdk::evm::util::get_provider_http;
use blueprint_sdk::producers::CronJob;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::eigenlayer::bls::EigenlayerBLSConfig;
use phala_tee_cloud_avs_blueprint_lib::{
    HEARTBEAT_JOB_ID, PhalaAvsContext, RESPOND_TO_CHALLENGE_JOB_ID, heartbeat_job,
    respond_to_challenge_job,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();
    info!("Starting Phala Cloud AVS Operator...");

    let env = BlueprintEnvironment::load()?;
    info!("Environment loaded.");

    // --- EVM Setup ---
    let http_rpc_url = env.http_rpc_endpoint.clone();
    let provider = get_provider_http(&http_rpc_url);
    info!("EVM Provider initialized.");

    // --- Polling Producer ---
    let polling_config = PollingConfig::default().poll_interval(Duration::from_secs(5)); // Adjust interval as needed
    let producer = PollingProducer::new(Arc::new(provider), polling_config).await?;
    info!("PollingProducer initialized.");

    // --- Eigenlayer Config ---
    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    info!("EigenlayerBLSConfig initialized.");

    // --- Context ---
    let context = PhalaAvsContext::new(env.clone()).await?;
    info!("PhalaAvsContext initialized.");

    // --- Cron Job for Heartbeat ---
    let heartbeat_cron = CronJob::new(HEARTBEAT_JOB_ID, "* * * * *").await?;
    info!("Heartbeat cron job scheduled.");

    // --- Router ---
    let router = Router::new()
        // TODO: Define job ID and handler for responding to on-chain challenges/events
        .route(RESPOND_TO_CHALLENGE_JOB_ID, respond_to_challenge_job)
        .with_context(context.clone());
    info!("Router configured.");

    // --- Aggregator Client (Optional Background Service) ---
    // Example: If you need to interact with an external aggregator service
    // let aggregator_client_config = EigenDaConfig { /* Load from env */ };
    // let aggregator_client = AggregatorClient::new(aggregator_client_config)?;
    // let aggregator_service = ServiceBuilder::new().service(aggregator_client);

    // --- Runner ---
    let runner_result = BlueprintRunner::builder(eigen_config, env)
        .router(router)
        .producer(producer)
        .producer(heartbeat_cron) // Add cron job as a producer
        // .background_service(aggregator_service) // Example: Add background service if needed
        .with_shutdown_handler(async { info!("Shutting down Phala Cloud AVS Operator...") })
        .run()
        .await;

    if let Err(e) = runner_result {
        error!("Runner failed: {:?}", e);
        return Err(e.into());
    }

    info!("Phala Cloud AVS Operator finished.");
    Ok(())
}

pub fn setup_log() {
    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(LevelFilter::INFO) // Set default level
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE) // Log span events
        .with_target(true) // Show module targets
        .try_init();
}

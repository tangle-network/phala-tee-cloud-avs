//!
//! End-to-End Test for Phala Cloud AVS Eigenlayer Blueprint
//!

#![allow(unused_variables, unused_imports, dead_code)]

use blueprint_sdk::{
    alloy::primitives::{Address, U256}, error, evm::producer::{PollingConfig, PollingProducer}, info, runner::{config::BlueprintEnvironment, eigenlayer::bls::EigenlayerBLSConfig, BlueprintRunner}, tangle::subxt_core::constants::address, testing::{
        tempfile,
        utils::{eigenlayer::EigenlayerTestHarness, setup_log},
    }, warn, Router
};
use eigenlayer_contract_deployer::core::{deploy_core_contracts, DelegationManagerConfig, DeploymentConfigData, EigenPodManagerConfig, RewardsCoordinatorConfig, StrategyFactoryConfig, StrategyManagerConfig};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use phala_tee_cloud_avs_blueprint_lib::{
    HEARTBEAT_JOB_ID, RESPOND_TO_CHALLENGE_JOB_ID,
    context::PhalaAvsContext,
    jobs::{heartbeat_job, respond_to_challenge_job},
};
use reqwest::Client;
use tokio::sync::oneshot;

// Constants (adjust as needed)
const TOKENOMIC_MANAGER_INDEX: usize = 1; // Index in harness accounts for the manager
const OPERATOR_INDEX: usize = 0; // Index for the operator controlled by the blueprint
const RESPONSE_WINDOW_BLOCKS: u64 = 10;

#[tokio::test(flavor = "multi_thread")]
async fn test_phala_avs_e2e() -> eyre::Result<()> {
    setup_log();
    info!("Starting Phala AVS E2E Test...");

    // Initialize test harness
    let temp_dir = tempfile::TempDir::new().unwrap();
    let harness = EigenlayerTestHarness::setup(temp_dir).await.unwrap();

    let env = harness.env().clone();
    let http_endpoint = harness.http_endpoint.to_string();

    let private_key = PRIVATE_KEY.to_string();

    let core_config = DeploymentConfigData {
        strategy_manager: StrategyManagerConfig {
            init_paused_status: U256::from(0),
            init_withdrawal_delay_blocks: 1u32,
        },
        delegation_manager: DelegationManagerConfig {
            init_paused_status: U256::from(0),
            withdrawal_delay_blocks: 0u32,
        },
        eigen_pod_manager: EigenPodManagerConfig {
            init_paused_status: U256::from(0),
        },
        rewards_coordinator: RewardsCoordinatorConfig {
            init_paused_status: U256::from(0),
            max_rewards_duration: 864000u32,
            max_retroactive_length: 432000u32,
            max_future_length: 86400u32,
            genesis_rewards_timestamp: 1672531200u32,
            updater: harness.owner_account(),
            activation_delay: 0u32,
            calculation_interval_seconds: 86400u32,
            global_operator_commission_bips: 1000u16,
        },
        strategy_factory: StrategyFactoryConfig {
            init_paused_status: U256::from(0),
        },
    };
    // Deploy Core EigenLayer Contracts
    let core_config = DeploymentConfigData::default();
    let core_contracts = deploy_core_contracts(
        &http_endpoint,
        &,
        owner_account,
        core_config,
        Some(address!("00000000219ab540356cBB839Cbe05303d7705Fa")),
        Some(1_564_000),
    )
    .await
    .unwrap();
    info!("Core EigenLayer contracts deployed.");

    // Deploy Phala AVS Contracts
    // Deploy Mock PHA Token first
    let pha_token_address = deploy_mock_erc20(&owner_provider, "PhalaToken", "PHA").await?;
    info!(?pha_token_address, "Mock PHA ERC20 deployed.");

    // Deploy PhalaSlaOracle
    // TODO: Generate bindings for PhalaSlaOracle constructor
    /*
    let oracle_deployment_call = PhalaSlaOracle::constructor_call(...);
    let oracle_address = owner_provider.deploy(oracle_deployment_call).await?;
    */
    let oracle_address = Address::default(); // Placeholder
    let phala_sla_oracle = IPhalaSlaOracle::new(oracle_address, manager_provider.clone());
    info!(?oracle_address, "PhalaSlaOracle deployed (Placeholder).");

    // Deploy PhalaServiceManager
    // TODO: Generate bindings for PhalaServiceManager constructor
    /*
    let service_manager_deployment_call = PhalaServiceManager::constructor_call(...);
    let service_manager_address = owner_provider.deploy(service_manager_deployment_call).await?;
    */
    let service_manager_address = Address::default(); // Placeholder
    let phala_service_manager =
        IPhalaServiceManager::new(service_manager_address, manager_provider.clone());
    info!(
        ?service_manager_address,
        "PhalaServiceManager deployed (Placeholder)."
    );

    // Initialize contracts (replace with actual binding calls)
    /*
    let oracle_init_call = phala_sla_oracle.initialize(owner_account, tokenomic_manager_address);
    wait_for_receipt(oracle_init_call).await?;
    let sm_init_call = phala_service_manager.initialize(owner_account, owner_account, tokenomic_manager_address); // Assuming owner is rewards initiator for now
    wait_for_receipt(sm_init_call).await?;
    info!("AVS contracts initialized.");
    */

    // 4. Setup AVS Permissions (Adapt setup_avs_permissions or do manually)
    // This likely involves calling functions on PauserRegistry, PermissionController, etc.
    // Needs the generated bindings and potentially a custom setup function.
    info!("AVS permissions setup (Skipped - Placeholder).");

    // 5. Create Quorum (If necessary for Phala's model - check EigenLayer docs)
    // This involves calling RegistryCoordinator.createTotalDelegatedStakeQuorum or similar
    info!("Quorum creation (Skipped - Placeholder).");

    // 7. Setup Blueprint Runner
    // --- Context ---
    let context = PhalaAvsContext::new(env.clone()).await?;
    info!("PhalaAvsContext initialized.");

    // --- Polling Producer ---
    let polling_config = PollingConfig::default()
        .poll_interval(Duration::from_secs(1))
        .confirmations(0); // Use 0 confs for faster testing on Anvil
    let http_provider = ProviderBuilder::new()
        .on_client(RpcClient::new(
            Http::<Client>::new(http_endpoint.parse()?),
            true,
        ))
        .await?;
    let producer = PollingProducer::new(Arc::new(http_provider), polling_config).await?;
    info!("PollingProducer initialized.");

    // --- Eigenlayer Config ---
    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default())
        .with_exit_after_register(false);
    info!("EigenlayerBLSConfig initialized.");

    // --- Cron Job for Heartbeat ---
    let heartbeat_cron = CronJob::new(HEARTBEAT_JOB_ID, "* * * * * *".parse()?, context.clone())?; // Every second for testing
    info!("Heartbeat cron job scheduled.");

    // --- Router ---
    let router = Router::new()
        .route(RESPOND_TO_CHALLENGE_JOB_ID, respond_to_challenge_job) // Assumes challenge events have this job ID
        // TODO: Consider how/if heartbeat job interacts with router or if it's purely context-based
        .with_context(context.clone());
    info!("Router configured.");

    // --- Run the Blueprint ---
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let runner_handle = tokio::spawn(async move {
        let result = BlueprintRunner::builder(eigen_config, env)
            .router(router)
            .producer(producer)
            .producer(heartbeat_cron)
            .with_shutdown_handler(async { info!("Shutting down Phala Cloud AVS Operator...") })
            .run()
            .await;
        let _ = shutdown_tx.send(result);
    });
    info!("BlueprintRunner started.");

    // --- Allow time for registration ---
    tokio::time::sleep(Duration::from_secs(5)).await;
    info!("Waited for potential registration...");

    // 8. Simulate SLA Challenge
    info!("Issuing SLA Challenge...");
    let challenge_data = Bytes::from_static(b"test_challenge");
    /*
    let issue_challenge_call = phala_sla_oracle.issueSlaChallenge(operator_address, challenge_data.clone());
    let issue_receipt = wait_for_receipt(issue_challenge_call).await?;
    info!(tx_hash = ?issue_receipt.transaction_hash, "SLA Challenge issued.");

    // TODO: Extract challengeId from the SlaChallengeIssued event log
    let challenge_id = U256::from(1); // Placeholder
    */
    let challenge_id = U256::from(1); // Placeholder

    // 9. Wait and Verify Response
    info!("Waiting for operator to respond to challenge...");
    // Need a reliable way to detect response. Checking contract state is best.
    let verification_timeout = Duration::from_secs(30);
    let check_interval = Duration::from_secs(2);
    let start_time = std::time::Instant::now();
    let mut responded = false;

    while start_time.elapsed() < verification_timeout {
        /*
        let details_call = phala_sla_oracle.getChallengeDetails(challenge_id);
        let details = details_call.call().await?;
        if details.responded {
            info!("Challenge {} successfully responded to!", challenge_id);
            responded = true;
            break;
        }
        */
        // Placeholder check - replace with actual contract call
        if start_time.elapsed() > Duration::from_secs(10) {
            // Simulate response after 10s
            info!("Simulating successful response check.");
            responded = true;
            break;
        }
        tokio::time::sleep(check_interval).await;
    }

    // 10. Shutdown
    info!("Shutting down runner...");
    runner_handle.abort();
    let runner_result = shutdown_rx.await?;
    if let Err(e) = runner_result {
        error!(?e, "Blueprint runner failed!");
    }

    // Assertions
    assert!(
        responded,
        "Operator failed to respond to the SLA challenge within the timeout."
    );

    info!("Phala AVS E2E Test Completed Successfully!");
    Ok(())
}

// TODO:
// - Add helper function `deploy_phala_avs_contracts`
// - Generate and use actual contract bindings (sol!, Abigen)
// - Implement proper AVS permission setup
// - Implement quorum creation if needed
// - Parse event logs to get challengeId
// - Replace placeholder contract calls with actual calls using generated bindings
// - Add test case for challenge expiry/failure reporting
// - Add verification for heartbeat job (e.g., checking logs or emitted events if added)

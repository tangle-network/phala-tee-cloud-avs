use crate::PhalaAvsError;
use crate::context::PhalaAvsContext;
use blueprint_sdk::evm::extract::BlockEvents;
use blueprint_sdk::extract::Context;
use blueprint_sdk::macros::debug_job;
use blueprint_sdk::{info, warn};

// --- Job IDs ---

/// Job ID for the periodic heartbeat/liveness check (Cron Job).
pub const HEARTBEAT_JOB_ID: u32 = 0;

/// Job ID for handling potential on-chain challenges or other EVM events.
pub const RESPOND_TO_CHALLENGE_JOB_ID: u32 = 1; // Example ID

// --- Job Handlers ---

/// Cron job handler for periodic heartbeat/SLA check.
///
/// This function is triggered periodically by the `CronJob` producer.
/// It should perform necessary checks (like TEE liveness) and potentially
/// report status or take action if issues are detected.
#[debug_job]
pub async fn heartbeat_job(Context(ctx): Context<PhalaAvsContext>) -> Result<(), PhalaAvsError> {
    info!("Running heartbeat job...");

    match ctx.tee_handler.check_liveness().await {
        Ok(is_live) => {
            if is_live {
                info!("Heartbeat check: TEE/Node is live.");
                // TODO: Potentially report liveness status if required by the AVS design.
            } else {
                warn!("Heartbeat check: TEE/Node is NOT live!");
                // TODO: Implement alerting or recovery logic.
            }
        }
        Err(e) => {
            warn!("Heartbeat check failed: {:?}", e);
            // TODO: Handle error appropriately.
        }
    }

    // Cron jobs typically don't return data for Eigenlayer tasks,
    // but might interact with context or external systems.
    Ok(())
}

/// Job handler for responding to specific EVM events (e.g., challenges).
///
/// This function is triggered by the `PollingProducer` when relevant
/// logs matching configured filters are detected on the EVM chain.
#[debug_job]
pub async fn respond_to_challenge_job(
    Context(_ctx): Context<PhalaAvsContext>,
    BlockEvents(events): BlockEvents,
) -> Result<(), PhalaAvsError> {
    info!("Received {} potential challenge events.", events.len());

    // TODO: Implement logic to:
    // 1. Decode relevant logs using Alloy (e.g., `MyChallengeEvent::decode_log`).
    // 2. Filter for actual challenge events relevant to this operator.
    // 3. Perform the required action based on the challenge (e.g., interact with TEE, query state).
    // 4. Potentially submit a response transaction or sign data for the aggregator.

    for event in events {
        // Example: Log raw event data (use specific decoding in practice)
        info!(
            "Processing event from block: {:?}, tx: {:?}, log index: {:?}",
            event.block_number, event.transaction_hash, event.log_index
        );
        // Add decoding and handling logic here
    }

    // This job might need to return data or interact with the Eigenlayer task manager,
    // depending on the specific challenge mechanism.
    // For now, returning Ok indicates successful processing of the received batch.
    Ok(())
}

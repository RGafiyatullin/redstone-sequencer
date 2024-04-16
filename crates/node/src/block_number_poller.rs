use std::{
    future::Future,
    sync::{Arc, RwLock},
    time::Duration,
};

use reth_primitives::U256;
use reth_rpc_api::EthApiClient;

use crate::{upstream::Upstream, AnyError};

pub const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub async fn start(
    upstream: Upstream,
) -> Result<
    (
        Arc<RwLock<U256>>,
        impl Future<Output = Result<(), AnyError>>,
    ),
    AnyError,
> {
    let current_block_number = Arc::new(RwLock::new(Default::default()));
    let running = {
        let current_block_number = Arc::clone(&current_block_number);
        async move {
            let mut ticks = tokio::time::interval(POLL_INTERVAL);
            loop {
                let _ = ticks.tick().await;

                let block_number = match upstream.anonymous_client().block_number().await {
                    Ok(block_number) => block_number,
                    Err(reason) => {
                        tracing::warn!("failed to fetch block-number: {}", reason);
                        continue;
                    }
                };

                *current_block_number.write().expect("rw-write poisoned") = block_number;
            }
        }
    };

    Ok((current_block_number, running))
}

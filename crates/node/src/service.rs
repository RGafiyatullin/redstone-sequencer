use std::{
    future::Future,
    sync::{Arc, RwLock},
};

use futures::{future, TryFutureExt};
use reth_primitives::U256;

use crate::{
    block_number_poller,
    sequencer::{self, Sequencer},
    upstream::Upstream,
    AnyError,
};

pub async fn start(
    upstream: Upstream,
) -> Result<(Service, impl Future<Output = Result<(), AnyError>>), AnyError> {
    let (sequencer, sequencer_running) = sequencer::start(upstream.clone()).await?;
    let (current_block_number, block_number_poller_running) =
        block_number_poller::start(upstream.clone()).await?;
    let service = Service {
        upstream,
        sequencer,
        current_block_number,
    };
    let running = future::try_join(block_number_poller_running, sequencer_running).map_ok(|_| ());
    Ok((service, running))
}

#[derive(Debug, Clone)]
pub struct Service {
    upstream: Upstream,
    sequencer: Sequencer,
    current_block_number: Arc<RwLock<U256>>,
}

impl Service {
    pub fn upstream(&self) -> &Upstream {
        &self.upstream
    }
    pub fn sequencer(&self) -> &Sequencer {
        &self.sequencer
    }
    pub fn current_block_number(&self) -> U256 {
        *self.current_block_number.read().expect("rw-read poisoned")
    }
}

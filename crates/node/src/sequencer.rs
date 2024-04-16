use std::future::Future;

use tokio::sync::mpsc;

use crate::{upstream::Upstream, AnyError};

pub mod query;
pub use query::Query;

mod pool;
use pool::Pool;

const QUERY_CHANNEL_BUFFER_SIZE: usize = 1024;

pub async fn start(
    upstream: Upstream,
) -> Result<(Sequencer, impl Future<Output = Result<(), AnyError>>), AnyError> {
    let (query_tx, query_rx) = mpsc::channel(QUERY_CHANNEL_BUFFER_SIZE);
    let sequencer = Sequencer { query_tx };
    Ok((sequencer, run(query_rx, upstream)))
}

#[derive(Debug, Clone)]
pub struct Sequencer {
    query_tx: mpsc::Sender<Query>,
}

async fn run(query_rx: mpsc::Receiver<Query>, upstream: Upstream) -> Result<(), AnyError> {
    tracing::info!("Starting sequencer...");

    std::future::pending().await
}

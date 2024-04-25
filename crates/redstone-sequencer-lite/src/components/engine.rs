use std::future::Future;
use std::sync::Arc;

use futures::Stream;
use futures::StreamExt;
use reth_node_api::ConfigureEvm;
use reth_node_api::ConfigureEvmEnv;
use reth_primitives::ChainSpec;
use reth_provider::StateProviderFactory;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;

use crate::AnyError;

mod impl_engine_api;
mod impl_eth_api;

const CHANNEL_BUFFER_SIZE: usize = 64;

#[derive(Debug, Clone)]
pub struct Api {
    pub query_tx: mpsc::Sender<Query>,
    pub chain_spec: Arc<ChainSpec>,
}

#[derive(Debug)]
pub struct Args<B, V> {
    pub chain_spec: Arc<ChainSpec>,
    pub blockchain: B,
    pub evm_config: V,
}

pub fn start<B, V>(args: Args<B, V>) -> (Api, impl Future<Output = Result<(), AnyError>>)
where
    B: StateProviderFactory,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    let (query_tx, query_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    let api = Api {
        query_tx,
        chain_spec: Arc::clone(&args.chain_spec),
    };
    let queries = ReceiverStream::new(query_rx);
    let running = run(queries, args);

    (api, running)
}

#[derive(Debug)]
pub enum Query {}

pub async fn run<Q, B, V>(queries: Q, args: Args<B, V>) -> Result<(), AnyError>
where
    Q: Stream<Item = Query>,
    B: StateProviderFactory,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    let mut queries = std::pin::pin!(queries);

    while let Some(query) = queries.next().await {
        error!("UNHANDLED QUERY: {:?}", query);
    }
    Ok(())
}

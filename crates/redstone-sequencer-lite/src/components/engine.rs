use std::future::Future;
use std::sync::Arc;

use futures::Stream;
use futures::StreamExt;
use reth_node_api::ConfigureEvm;
use reth_node_api::ConfigureEvmEnv;
use reth_primitives::Address;
use reth_primitives::ChainSpec;
use reth_primitives::U256;
use reth_provider::StateProviderFactory;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use tracing::warn;

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

#[derive(Debug)]
pub struct State<B, V> {
    args: Args<B, V>,
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
pub enum Query {
    GetTransactionCount {
        address: Address,
        reply_tx: oneshot::Sender<u64>,
    },
    GetBalance {
        address: Address,
        reply_tx: oneshot::Sender<U256>,
    },
}

pub async fn run<Q, B, V>(queries: Q, args: Args<B, V>) -> Result<(), AnyError>
where
    Q: Stream<Item = Query>,
    B: StateProviderFactory,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    let mut queries = std::pin::pin!(queries);

    let mut state = State { args };

    while let Some(query) = queries.next().await {
        state.handle_query(query).await?;
    }
    Ok(())
}

impl<B, V> State<B, V>
where
    B: StateProviderFactory,
{
    pub async fn handle_query(&mut self, query: Query) -> Result<(), AnyError> {
        match query {
            Query::GetTransactionCount { address, reply_tx } => {
                self.handle_get_transaction_count(address, reply_tx).await
            }
            Query::GetBalance { address, reply_tx } => {
                self.handle_get_balance(address, reply_tx).await
            }
        }
    }

    async fn handle_get_transaction_count(
        &self,
        address: Address,
        reply_tx: oneshot::Sender<u64>,
    ) -> Result<(), AnyError> {
        let nonce = self
            .blockchain()
            .latest()?
            .account_nonce(address)?
            .unwrap_or_default();
        let _ = reply_tx
            .send(nonce)
            .inspect_err(|_| warn!("oneshot-tx: closed"));

        Ok(())
    }

    async fn handle_get_balance(
        &self,
        address: Address,
        reply_tx: oneshot::Sender<U256>,
    ) -> Result<(), AnyError> {
        let balance = self
            .blockchain()
            .latest()?
            .account_balance(address)?
            .unwrap_or_default();
        let _ = reply_tx
            .send(balance)
            .inspect_err(|_| warn!("oneshot-tx: closed"));

        Ok(())
    }
}

impl<B, V> State<B, V>
where
    B: StateProviderFactory,
{
    fn blockchain(&self) -> &dyn StateProviderFactory {
        &self.args.blockchain
    }
}

use std::future::Future;
use std::sync::Arc;

use alloy_eips::BlockNumberOrTag;
use futures::Stream;
use futures::StreamExt;
use reth_node_api::ConfigureEvm;
use reth_node_api::ConfigureEvmEnv;
use reth_primitives::Address;
use reth_primitives::ChainSpec;
use reth_primitives::PooledTransactionsElement;
use reth_primitives::B256;
use reth_primitives::U256;
use reth_provider::BlockReader;
use reth_provider::StateProviderFactory;
use reth_rpc::eth::error::EthApiError;
use reth_rpc_types::AnyTransactionReceipt;
use reth_rpc_types::RichBlock;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use tracing::warn;

use crate::AnyError;

mod impl_engine_api;
mod impl_eth_api;
mod tx_pool;

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

pub trait Blockchain: StateProviderFactory + BlockReader {}
impl<B> Blockchain for B where B: StateProviderFactory + BlockReader {}

#[derive(Debug)]
pub struct State<B, V> {
    args: Args<B, V>,
    nonces: tx_pool::Nonces,
    tx_pool: tx_pool::TxPool,
}

pub fn start<B, V>(args: Args<B, V>) -> (Api, impl Future<Output = Result<(), AnyError>>)
where
    B: Blockchain,
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
    GetBlockByNumber {
        number: BlockNumberOrTag,
        full: bool,
        reply_tx: oneshot::Sender<Result<Option<RichBlock>, EthApiError>>,
    },
    GetBlockByHash {
        hash: B256,
        full: bool,
        reply_tx: oneshot::Sender<Result<Option<RichBlock>, EthApiError>>,
    },
    TransactionAdd {
        tx: PooledTransactionsElement,
    },
    GetTransactionReceipt {
        hash: B256,
        reply_tx: oneshot::Sender<Result<Option<AnyTransactionReceipt>, EthApiError>>,
    },
}

pub async fn run<Q, B, V>(queries: Q, args: Args<B, V>) -> Result<(), AnyError>
where
    Q: Stream<Item = Query>,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    let mut queries = std::pin::pin!(queries);

    let mut state = State {
        args,
        nonces: Default::default(),
        tx_pool: Default::default(),
    };

    while let Some(query) = queries.next().await {
        state.handle_query(query).await?;
    }
    Ok(())
}

impl<B, V> State<B, V>
where
    B: Blockchain,
{
    pub async fn handle_query(&mut self, query: Query) -> Result<(), AnyError> {
        match query {
            Query::GetTransactionCount { address, reply_tx } => {
                self.handle_get_transaction_count(address, reply_tx).await
            }
            Query::GetBalance { address, reply_tx } => {
                self.handle_get_balance(address, reply_tx).await
            }
            Query::GetBlockByNumber {
                number,
                full,
                reply_tx,
            } => {
                self.handle_get_block_by_number(number, full, reply_tx)
                    .await
            }
            Query::GetBlockByHash {
                hash,
                full,
                reply_tx,
            } => self.handle_get_block_by_hash(hash, full, reply_tx).await,

            Query::TransactionAdd { tx } => self.handle_transaction_add(tx).await.or_else(|err| {
                tracing::error!(error = %err, "Error adding transaction.");
                Ok(())
            }),

            Query::GetTransactionReceipt { hash, reply_tx } => {
                self.handle_get_transaction_receipt(hash, reply_tx).await
            }
        }
    }

    fn blockchain(&self) -> &B {
        &self.args.blockchain
    }
}

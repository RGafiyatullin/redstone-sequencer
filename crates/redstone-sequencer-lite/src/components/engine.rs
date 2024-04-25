use std::sync::Arc;

use alloy_eips::BlockNumberOrTag;
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
use tokio::sync::RwLock;

use crate::AnyError;

mod built_payload;
mod impl_engine_api;
mod impl_eth_api;
mod preview;
mod tx_pool;

pub use built_payload::RedstoneBuiltPayload;

#[derive(Debug)]
pub struct Args<B, V> {
    pub chain_spec: Arc<ChainSpec>,
    pub blockchain: B,
    pub evm_config: V,
}

#[derive(Debug)]
pub struct Engine<B, V>(Arc<RwLock<State<B, V>>>);

pub trait Blockchain: StateProviderFactory + BlockReader {}
impl<B> Blockchain for B where B: StateProviderFactory + BlockReader {}

#[derive(Debug)]
struct State<B, V> {
    args: Args<B, V>,
    nonces: tx_pool::Nonces,
    tx_pool: tx_pool::TxPool,
}

pub fn create<B, V>(args: Args<B, V>) -> Engine<B, V> {
    let state = State {
        args,
        nonces: Default::default(),
        tx_pool: Default::default(),
    };
    Engine(Arc::new(RwLock::new(state)))
}

impl<B, V> State<B, V>
where
    B: Blockchain,
{
    fn blockchain(&self) -> &B {
        &self.args.blockchain
    }
}

impl<B, V> Clone for Engine<B, V> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

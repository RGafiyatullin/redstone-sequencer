use std::collections::HashMap;
use std::sync::Arc;

use reth_interfaces::blockchain_tree::BlockchainTreeEngine;
use reth_optimism_payload_builder::OptimismPayloadBuilderAttributes;
use reth_payload_builder::PayloadId;
use reth_primitives::Bytes;
use reth_primitives::ChainSpec;
use reth_provider::BlockIdReader;
use reth_provider::BlockReader;
use reth_provider::CanonChainTracker;
use reth_provider::ChainSpecProvider;
use reth_provider::StageCheckpointReader;
use reth_provider::StateProviderFactory;
use tokio::sync::RwLock;

mod built_payload;
mod impl_engine_api;
mod impl_eth_api;
mod preview;
mod tx_pool;

pub use built_payload::RedstoneBuiltPayload;

use self::preview::Preview;

#[derive(Debug)]
pub struct Args<B, V> {
    pub chain_spec: Arc<ChainSpec>,
    pub blockchain: B,
    pub evm_config: V,
    pub payload_extradata: Bytes,
}

#[derive(Debug)]
pub struct Engine<B, V>(Arc<RwLock<State<B, V>>>);

pub trait Blockchain:
    Clone
    + StateProviderFactory
    + BlockchainTreeEngine
    + BlockReader
    + BlockIdReader
    + CanonChainTracker
    + StageCheckpointReader
    + ChainSpecProvider
{
}
impl<B> Blockchain for B where
    B: Clone
        + StateProviderFactory
        + BlockchainTreeEngine
        + BlockReader
        + BlockIdReader
        + CanonChainTracker
        + StageCheckpointReader
        + ChainSpecProvider
{
}

#[derive(Debug)]
struct State<B, V> {
    args: Args<B, V>,
    nonces: tx_pool::Nonces,
    tx_pool: tx_pool::TxPool,
    payloads: HashMap<PayloadId, Preview<OptimismPayloadBuilderAttributes, B, V>>,
}

pub fn create<B, V>(args: Args<B, V>) -> Engine<B, V> {
    let state = State {
        args,
        nonces: Default::default(),
        tx_pool: Default::default(),
        payloads: Default::default(),
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

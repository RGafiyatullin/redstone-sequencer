use std::sync::Arc;

use reth_node_api::BuiltPayload;
use reth_payload_builder::PayloadId;
use reth_primitives::{BlobTransactionSidecar, ChainSpec, SealedBlock, U256};

#[derive(Debug, Clone)]
pub struct RedstoneBuiltPayload<A> {
    /// Identifier of the payload
    pub id: PayloadId,
    /// The built block
    pub block: SealedBlock,
    /// The fees of the block
    pub fees: U256,
    /// The blobs, proofs, and commitments in the block. If the block is pre-cancun, this will be
    /// empty.
    pub sidecars: Vec<BlobTransactionSidecar>,
    /// The rollup's chainspec.
    pub chain_spec: Arc<ChainSpec>,
    /// The payload attributes.
    pub attributes: A,
}

impl<A> BuiltPayload for RedstoneBuiltPayload<A>
where
    A: Send + Sync + std::fmt::Debug,
{
    fn block(&self) -> &SealedBlock {
        &self.block
    }

    fn fees(&self) -> U256 {
        self.fees
    }
}

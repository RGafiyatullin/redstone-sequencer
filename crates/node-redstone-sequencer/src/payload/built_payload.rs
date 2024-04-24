use std::sync::Arc;

use alloy_rpc_types_engine::{ExecutionPayloadEnvelopeV2, OptimismExecutionPayloadEnvelopeV3};
use reth_node_api::BuiltPayload;
use reth_payload_builder::PayloadId;
use reth_primitives::{BlobTransactionSidecar, ChainSpec, SealedBlock, U256};
use reth_rpc_types::ExecutionPayloadV1;

use super::RedstonePayloadBuilderAttributes;

#[derive(Debug, Clone)]
pub struct RedstoneBuiltPayload {
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
    pub attributes: RedstonePayloadBuilderAttributes,
}

impl BuiltPayload for RedstoneBuiltPayload {
    /// Returns the built block (sealed)
    fn block(&self) -> &SealedBlock {
        unimplemented!()
    }

    /// Returns the fees collected for the built block
    fn fees(&self) -> U256 {
        unimplemented!()
    }
}

impl TryFrom<RedstoneBuiltPayload> for OptimismExecutionPayloadEnvelopeV3 {
    type Error = std::convert::Infallible;
    fn try_from(_value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<RedstoneBuiltPayload> for ExecutionPayloadEnvelopeV2 {
    type Error = std::convert::Infallible;
    fn try_from(_value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<RedstoneBuiltPayload> for ExecutionPayloadV1 {
    type Error = std::convert::Infallible;
    fn try_from(_value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

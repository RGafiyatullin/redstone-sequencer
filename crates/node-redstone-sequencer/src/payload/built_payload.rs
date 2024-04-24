use std::sync::Arc;

use alloy_rpc_types_engine::{
    ExecutionPayloadEnvelopeV2, ExecutionPayloadFieldV2, OptimismExecutionPayloadEnvelopeV3,
};
use reth_node_api::{BuiltPayload, PayloadBuilderAttributes};
use reth_payload_builder::PayloadId;
use reth_primitives::{BlobTransactionSidecar, ChainSpec, SealedBlock, B256, U256};
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
        &self.block
    }

    /// Returns the fees collected for the built block
    fn fees(&self) -> U256 {
        self.fees
    }
}

impl TryFrom<RedstoneBuiltPayload> for OptimismExecutionPayloadEnvelopeV3 {
    type Error = std::convert::Infallible;
    fn try_from(value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        let RedstoneBuiltPayload {
            block,
            fees,
            sidecars,
            chain_spec,
            attributes,
            ..
        } = value;

        let parent_beacon_block_root = attributes
            .parent_beacon_block_root()
            .filter(|_| chain_spec.is_cancun_active_at_timestamp(attributes.timestamp()))
            .unwrap_or(B256::ZERO);

        let execution_payload = reth_rpc_types_compat::engine::payload::block_to_payload_v3(block);

        let out = OptimismExecutionPayloadEnvelopeV3 {
            execution_payload,
            block_value: fees,
            should_override_builder: false,
            blobs_bundle: sidecars
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            parent_beacon_block_root,
        };

        Ok(out)
    }
}

impl TryFrom<RedstoneBuiltPayload> for ExecutionPayloadEnvelopeV2 {
    type Error = std::convert::Infallible;
    fn try_from(value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        let RedstoneBuiltPayload { block, fees, .. } = value;
        let execution_payload = if block.withdrawals.is_some() {
            ExecutionPayloadFieldV2::V2(
                reth_rpc_types_compat::engine::payload::try_block_to_payload_v2(block),
            )
        } else {
            ExecutionPayloadFieldV2::V1(
                reth_rpc_types_compat::engine::payload::try_block_to_payload_v1(block),
            )
        };
        let out = Self {
            block_value: fees,
            execution_payload,
        };

        Ok(out)
    }
}

impl TryFrom<RedstoneBuiltPayload> for ExecutionPayloadV1 {
    type Error = std::convert::Infallible;
    fn try_from(value: RedstoneBuiltPayload) -> Result<Self, Self::Error> {
        let RedstoneBuiltPayload { block, .. } = value;
        Ok(reth_rpc_types_compat::engine::try_block_to_payload_v1(
            block,
        ))
    }
}

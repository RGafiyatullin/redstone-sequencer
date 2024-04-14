use alloy_rpc_types_engine::PayloadId;
use reth_node_api::BuiltPayload;
use reth_node_api::EngineTypes;
use reth_node_api::PayloadAttributes;
use reth_node_api::PayloadBuilderAttributes;
use reth_primitives::ChainSpec;
use reth_primitives::B256;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedstoneSequencerEngine;
impl EngineTypes for RedstoneSequencerEngine {
    type BuiltPayload = RedstoneSequencerBuiltPayload;
    type ExecutionPayloadV1 = RedstoneSequencerPayloadV1;
    type ExecutionPayloadV2 = RedstoneSequencerPayloadV2;
    type ExecutionPayloadV3 = RedstoneSequencerPayloadV3;
    type PayloadAttributes = RedstoneSequencerPayloadAttributes;
    type PayloadBuilderAttributes = RedstoneSequencerPayloadBuilderAttributes;

    fn validate_version_specific_fields(
        _chain_spec: &ChainSpec,
        _version: reth_node_api::EngineApiMessageVersion,
        _payload_or_attrs: reth_node_api::PayloadOrAttributes<'_, Self::PayloadAttributes>,
    ) -> Result<(), reth_node_api::EngineObjectValidationError> {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct RedstoneSequencerBuiltPayload {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedstoneSequencerPayloadV1 {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedstoneSequencerPayloadV2 {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedstoneSequencerPayloadV3 {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedstoneSequencerPayloadAttributes {}

#[derive(Debug, Clone)]
pub struct RedstoneSequencerPayloadBuilderAttributes {}

impl BuiltPayload for RedstoneSequencerBuiltPayload {
    fn block(&self) -> &reth_primitives::SealedBlock {
        unimplemented!()
    }
    fn fees(&self) -> reth_primitives::U256 {
        unimplemented!()
    }
}

impl PayloadAttributes for RedstoneSequencerPayloadAttributes {
    fn timestamp(&self) -> u64 {
        unimplemented!()
    }
    fn withdrawals(&self) -> Option<&Vec<alloy_rpc_types::Withdrawal>> {
        unimplemented!()
    }
    fn ensure_well_formed_attributes(
        &self,
        _chain_spec: &ChainSpec,
        _version: reth_node_api::EngineApiMessageVersion,
    ) -> Result<(), reth_node_api::EngineObjectValidationError> {
        unimplemented!()
    }
    fn parent_beacon_block_root(&self) -> Option<reth_primitives::B256> {
        unimplemented!()
    }
}

impl PayloadBuilderAttributes for RedstoneSequencerPayloadBuilderAttributes {
    type Error = std::convert::Infallible;
    type RpcPayloadAttributes = RedstoneSequencerPayloadAttributes;

    fn try_new(
        _parent: B256,
        _rpc_payload_attributes: Self::RpcPayloadAttributes,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn payload_id(&self) -> PayloadId {
        unimplemented!()
    }

    fn parent(&self) -> B256 {
        unimplemented!()
    }

    fn timestamp(&self) -> u64 {
        unimplemented!()
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        unimplemented!()
    }

    fn suggested_fee_recipient(&self) -> reth_primitives::Address {
        unimplemented!()
    }

    fn prev_randao(&self) -> B256 {
        unimplemented!()
    }

    fn withdrawals(&self) -> &reth_primitives::Withdrawals {
        unimplemented!()
    }

    fn cfg_and_block_env(
        &self,
        _chain_spec: &ChainSpec,
        _parent: &reth_primitives::Header,
    ) -> (
        reth_primitives::revm_primitives::CfgEnvWithHandlerCfg,
        reth_primitives::revm_primitives::BlockEnv,
    ) {
        unimplemented!()
    }
}

impl From<RedstoneSequencerBuiltPayload> for RedstoneSequencerPayloadV1 {
    fn from(_value: RedstoneSequencerBuiltPayload) -> Self {
        unimplemented!()
    }
}

impl From<RedstoneSequencerBuiltPayload> for RedstoneSequencerPayloadV2 {
    fn from(_value: RedstoneSequencerBuiltPayload) -> Self {
        unimplemented!()
    }
}

impl From<RedstoneSequencerBuiltPayload> for RedstoneSequencerPayloadV3 {
    fn from(_value: RedstoneSequencerBuiltPayload) -> Self {
        unimplemented!()
    }
}

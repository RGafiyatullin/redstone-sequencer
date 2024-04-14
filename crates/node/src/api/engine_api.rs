use alloy_primitives::{BlockHash, B256, U64};
use alloy_rpc_types_engine::{
    ExecutionPayloadBodiesV1, ForkchoiceState, ForkchoiceUpdated, PayloadId,
    TransitionConfiguration,
};
use alloy_rpc_types_engine::{
    ExecutionPayloadInputV2, ExecutionPayloadV1, ExecutionPayloadV3, PayloadStatus,
};
use jsonrpsee::core::RpcResult;
use reth_node_api::EngineTypes;
use reth_rpc_api::EngineApiServer;

use super::types::RedstoneSequencerEngine;
use super::Api;

#[async_trait::async_trait]
impl EngineApiServer<RedstoneSequencerEngine> for Api {
    async fn new_payload_v1(&self, _payload: ExecutionPayloadV1) -> RpcResult<PayloadStatus> {
        unimplemented!()
    }

    async fn new_payload_v2(&self, _payload: ExecutionPayloadInputV2) -> RpcResult<PayloadStatus> {
        unimplemented!()
    }

    async fn new_payload_v3(
        &self,
        _payload: ExecutionPayloadV3,
        _versioned_hashes: Vec<B256>,
        _parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus> {
        unimplemented!()
    }

    async fn fork_choice_updated_v1(
        &self,
        _fork_choice_state: ForkchoiceState,
        _payload_attributes: Option<<RedstoneSequencerEngine as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        unimplemented!()
    }

    async fn fork_choice_updated_v2(
        &self,
        _fork_choice_state: ForkchoiceState,
        _payload_attributes: Option<<RedstoneSequencerEngine as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        unimplemented!()
    }

    async fn fork_choice_updated_v3(
        &self,
        _fork_choice_state: ForkchoiceState,
        _payload_attributes: Option<<RedstoneSequencerEngine as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        unimplemented!()
    }

    async fn get_payload_v1(
        &self,
        _payload_id: PayloadId,
    ) -> RpcResult<<RedstoneSequencerEngine as EngineTypes>::ExecutionPayloadV1> {
        unimplemented!()
    }

    async fn get_payload_v2(
        &self,
        _payload_id: PayloadId,
    ) -> RpcResult<<RedstoneSequencerEngine as EngineTypes>::ExecutionPayloadV2> {
        unimplemented!()
    }

    async fn get_payload_v3(
        &self,
        _payload_id: PayloadId,
    ) -> RpcResult<<RedstoneSequencerEngine as EngineTypes>::ExecutionPayloadV3> {
        unimplemented!()
    }

    async fn get_payload_bodies_by_hash_v1(
        &self,
        _block_hashes: Vec<BlockHash>,
    ) -> RpcResult<ExecutionPayloadBodiesV1> {
        unimplemented!()
    }

    async fn get_payload_bodies_by_range_v1(
        &self,
        _start: U64,
        _count: U64,
    ) -> RpcResult<ExecutionPayloadBodiesV1> {
        unimplemented!()
    }

    async fn exchange_transition_configuration(
        &self,
        _transition_configuration: TransitionConfiguration,
    ) -> RpcResult<TransitionConfiguration> {
        unimplemented!()
    }

    async fn exchange_capabilities(&self, _capabilities: Vec<String>) -> RpcResult<Vec<String>> {
        unimplemented!()
    }
}

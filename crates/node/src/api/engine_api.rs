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
use reth_rpc_api::{EngineApiClient, EngineApiServer};

use reth_node_optimism::OptimismEngineTypes;

use super::to_error_object;
use super::Api;

impl Api {
    pub fn backend_engine_api(&self) -> &impl EngineApiClient<OptimismEngineTypes> {
        self.upstream().authenticated_client()
    }
}

#[async_trait::async_trait]
impl EngineApiServer<OptimismEngineTypes> for Api {
    async fn new_payload_v1(&self, payload: ExecutionPayloadV1) -> RpcResult<PayloadStatus> {
        tracing::error!("EngineAPI v1 invoked (new_payload_v1)");
        self.backend_engine_api()
            .new_payload_v1(payload)
            .await
            .map_err(to_error_object)
    }

    async fn new_payload_v2(&self, payload: ExecutionPayloadInputV2) -> RpcResult<PayloadStatus> {
        tracing::error!("EngineAPI v2 invoked (new_payload_v2)");
        self.backend_engine_api()
            .new_payload_v2(payload)
            .await
            .map_err(to_error_object)
    }

    async fn new_payload_v3(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<B256>,
        parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus> {
        tracing::trace!(
        target: "node::api::engine_api::new_payload_v3",
        "CALL [payload: {:#?}, versioned_hashes: {:#?}, parent_beacon_block_root: {}]",
        payload,
        versioned_hashes,
        parent_beacon_block_root,
        );

        self.backend_engine_api()
            .new_payload_v3(payload, versioned_hashes, parent_beacon_block_root)
            .await
            .inspect(|payload_status| {
                tracing::trace!(
                target: "node::api::engine_api::new_payload_v3", 
                "RET.OK: {:#?}", payload_status)
            })
            .inspect_err(|reason| {
                tracing::warn!(
                target: "node::api::engine_api::new_payload_v3",
                "RET.ERR: {:#?}", reason)
            })
            .map_err(to_error_object)
    }

    async fn fork_choice_updated_v1(
        &self,
        fork_choice_state: ForkchoiceState,
        payload_attributes: Option<<OptimismEngineTypes as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        tracing::error!("EngineAPI v1 invoked (fork_choice_updated_v1)");
        self.backend_engine_api()
            .fork_choice_updated_v1(fork_choice_state, payload_attributes)
            .await
            .map_err(to_error_object)
    }

    async fn fork_choice_updated_v2(
        &self,
        fork_choice_state: ForkchoiceState,
        payload_attributes: Option<<OptimismEngineTypes as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        tracing::error!("EngineAPI v2 invoked (fork_choice_updated_v2)");
        self.backend_engine_api()
            .fork_choice_updated_v2(fork_choice_state, payload_attributes)
            .await
            .map_err(to_error_object)
    }

    async fn fork_choice_updated_v3(
        &self,
        fork_choice_state: ForkchoiceState,
        payload_attributes: Option<<OptimismEngineTypes as EngineTypes>::PayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        tracing::trace!(
            target: "node::api::engine_api::fork_choice_updated_v3",
            "CALL [fork_choice_state: {:#?}; payload_attributes: {:#?}]",
            fork_choice_state, payload_attributes
        );
        self.backend_engine_api()
            .fork_choice_updated_v3(fork_choice_state, payload_attributes)
            .await
            .inspect(|forkchoice_updated| {
                tracing::trace!(
                target: "node::api::engine_api::fork_choice_updated_v3",
                "RET.OK: {:#?}", forkchoice_updated)
            })
            .inspect_err(|reason| {
                tracing::warn!(
                target: "node::api::engine_api::fork_choice_updated_v3",
                "RET.ERR: {:#?}", reason)
            })
            .map_err(to_error_object)
    }

    async fn get_payload_v1(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<<OptimismEngineTypes as EngineTypes>::ExecutionPayloadV1> {
        tracing::error!("EngineAPI v1 invoked (get_payload_v1)");
        self.backend_engine_api()
            .get_payload_v1(payload_id)
            .await
            .map_err(to_error_object)
    }

    async fn get_payload_v2(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<<OptimismEngineTypes as EngineTypes>::ExecutionPayloadV2> {
        tracing::error!("EngineAPI v2 invoked (get_payload_v2)");
        self.backend_engine_api()
            .get_payload_v2(payload_id)
            .await
            .map_err(to_error_object)
    }

    async fn get_payload_v3(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<<OptimismEngineTypes as EngineTypes>::ExecutionPayloadV3> {
        tracing::trace!(
            target: "node::api::engine_api::get_payload_v3",
            "CALL [payload_id: {}]", payload_id);
        self.backend_engine_api()
            .get_payload_v3(payload_id)
            .await
            .inspect(|payload| {
                tracing::trace!(
                target: "node::api::engine_api::get_payload_v3",
                "RET.OK: {:#?}", payload)
            })
            .inspect_err(|reason| {
                tracing::warn!(
                target: "node::api::engine_api::get_payload_v3",
                "RET.ERR: {:#?}", reason)
            })
            .map_err(to_error_object)
    }

    async fn get_payload_bodies_by_hash_v1(
        &self,
        block_hashes: Vec<BlockHash>,
    ) -> RpcResult<ExecutionPayloadBodiesV1> {
        tracing::error!(
            target: "node::api::engine_api::get_payload_bodies_by_hash_v1",
            "CALL [block_hashes: {:#?}]", block_hashes);
        self.backend_engine_api()
            .get_payload_bodies_by_hash_v1(block_hashes)
            .await
            .map_err(to_error_object)
    }

    async fn get_payload_bodies_by_range_v1(
        &self,
        start: U64,
        count: U64,
    ) -> RpcResult<ExecutionPayloadBodiesV1> {
        tracing::error!(
            target: "node::api::engine_api::get_payload_bodies_by_hash_v1",
            "CALL [start: {}; count: {}]", start, count);
        self.backend_engine_api()
            .get_payload_bodies_by_range_v1(start, count)
            .await
            .map_err(to_error_object)
    }

    async fn exchange_transition_configuration(
        &self,
        transition_configuration: TransitionConfiguration,
    ) -> RpcResult<TransitionConfiguration> {
        tracing::error!(
            target: "node::api::engine_api::exchange_transition_configuration",
            "CALL [transition_configuration: {:#?}]", transition_configuration);
        self.backend_engine_api()
            .exchange_transition_configuration(transition_configuration)
            .await
            .map_err(to_error_object)
    }

    async fn exchange_capabilities(&self, capabilities: Vec<String>) -> RpcResult<Vec<String>> {
        tracing::error!(
            target: "node::api::engine_api::exchange_capabilities",
            "CALL [capabilities: {:#?}]", capabilities);
        self.backend_engine_api()
            .exchange_capabilities(capabilities)
            .await
            .map_err(to_error_object)
    }
}

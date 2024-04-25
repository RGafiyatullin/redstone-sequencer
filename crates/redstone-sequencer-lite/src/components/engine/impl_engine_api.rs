use alloy_rpc_types_engine::{
    ForkchoiceState, ForkchoiceUpdated, OptimismExecutionPayloadEnvelopeV3,
    OptimismPayloadAttributes, PayloadStatus,
};
use jsonrpsee::core::RpcResult;
use reth_payload_builder::PayloadId;
use reth_primitives::B256;
use reth_rpc_types::ExecutionPayloadV3;

use crate::api::EngineApiV3Server;

use super::Api;

#[async_trait::async_trait]
impl EngineApiV3Server for Api {
    async fn new_payload(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<B256>,
        parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus> {
        unimplemented!()
    }

    async fn forkchoice_updated(
        &self,
        forkchoice_state: ForkchoiceState,
        payload_attributes: Option<OptimismPayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        unimplemented!()
    }

    async fn get_payload(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<OptimismExecutionPayloadEnvelopeV3> {
        unimplemented!()
    }
}

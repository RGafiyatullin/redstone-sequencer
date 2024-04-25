use alloy_rpc_types_engine::{
    ForkchoiceState, ForkchoiceUpdated, OptimismExecutionPayloadEnvelopeV3,
    OptimismPayloadAttributes,
};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_payload_builder::PayloadId;
use reth_primitives::B256;
use reth_rpc_types::engine::{ExecutionPayloadV3, PayloadStatus};

#[rpc(server, namespace = "engine")]
pub trait EngineApiV3 {
    #[method(name = "newPayloadV3")]
    async fn new_payload(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<B256>,
        parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus>;

    #[method(name = "forkchoiceUpdatedV3")]
    async fn forkchoice_updated(
        &self,
        forkchoice_state: ForkchoiceState,
        payload_attributes: Option<OptimismPayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated>;

    #[method(name = "getPayloadV3")]
    async fn get_payload(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<OptimismExecutionPayloadEnvelopeV3>;
}

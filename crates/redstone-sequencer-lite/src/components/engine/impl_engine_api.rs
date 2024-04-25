use alloy_rpc_types_engine::{
    ForkchoiceState, ForkchoiceUpdated, OptimismExecutionPayloadEnvelopeV3,
    OptimismPayloadAttributes, PayloadStatus, PayloadStatusEnum,
};
use jsonrpsee::core::RpcResult;
use reth_node_api::{ConfigureEvm, ConfigureEvmEnv};
use reth_payload_builder::PayloadId;
use reth_primitives::B256;
use reth_rpc_types::ExecutionPayloadV3;

use crate::api::EngineApiV3Server;

use super::{Blockchain, Engine};

#[async_trait::async_trait]
impl<B, V> EngineApiV3Server for Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    async fn new_payload(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<B256>,
        parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus> {
        Ok(PayloadStatus::new(
            alloy_rpc_types_engine::PayloadStatusEnum::Valid,
            Default::default(),
        ))
    }

    async fn forkchoice_updated(
        &self,
        forkchoice_state: ForkchoiceState,
        payload_attributes: Option<OptimismPayloadAttributes>,
    ) -> RpcResult<ForkchoiceUpdated> {
        if let Some(payload_attributes) = payload_attributes {
            unimplemented!()
        } else {
            Ok(ForkchoiceUpdated {
                payload_status: PayloadStatus {
                    status: PayloadStatusEnum::Valid,
                    latest_valid_hash: Some(forkchoice_state.head_block_hash),
                },
                payload_id: None,
            })
        }
    }

    async fn get_payload(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<OptimismExecutionPayloadEnvelopeV3> {
        unimplemented!()
    }
}

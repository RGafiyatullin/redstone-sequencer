use alloy_eips::eip4895::Withdrawal;
use reth_node_api::{EngineApiMessageVersion, EngineObjectValidationError, PayloadAttributes};
use reth_primitives::{Bytes, ChainSpec, B256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedstonePayloadAttributes {
    #[serde(flatten)]
    pub payload_attributes: alloy_rpc_types_engine::PayloadAttributes,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Bytes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tx_pool: Option<bool>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::u64_hex_opt"
    )]
    pub gas_limit: Option<u64>,
}

impl PayloadAttributes for RedstonePayloadAttributes {
    fn timestamp(&self) -> u64 {
        unimplemented!()
    }
    fn withdrawals(&self) -> Option<&Vec<Withdrawal>> {
        unimplemented!()
    }
    fn parent_beacon_block_root(&self) -> Option<B256> {
        unimplemented!()
    }

    fn ensure_well_formed_attributes(
        &self,
        _chain_spec: &ChainSpec,
        _version: EngineApiMessageVersion,
    ) -> Result<(), EngineObjectValidationError> {
        unimplemented!()
    }
}

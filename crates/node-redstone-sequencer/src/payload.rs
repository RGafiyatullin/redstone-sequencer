use std::sync::Arc;

use reth_primitives::Bytes;
use serde::Deserialize;
use serde::Serialize;

use alloy_rpc_types_engine::PayloadAttributes;
use reth_payload_builder::{EthPayloadBuilderAttributes, PayloadId};
use reth_primitives::{BlobTransactionSidecar, ChainSpec, SealedBlock, TransactionSigned, U256};

mod builder;
mod builder_attributes;
mod built_payload;
mod payload_attributes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedstonePayloadBuilder<EvmConfig> {
    chain_spec: Arc<ChainSpec>,
    evm_config: EvmConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedstonePayloadBuilderAttributes {
    pub payload_attributes: EthPayloadBuilderAttributes,
    pub no_tx_pool: bool,
    pub transactions: Vec<TransactionSigned>,
    pub gas_limit: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedstonePayloadAttributes {
    #[serde(flatten)]
    pub payload_attributes: PayloadAttributes,

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

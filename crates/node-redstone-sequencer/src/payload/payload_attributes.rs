use alloy_eips::eip4895::Withdrawal;
use reth_node_api::{EngineApiMessageVersion, EngineObjectValidationError, PayloadAttributes};
use reth_primitives::{ChainSpec, B256};

use super::RedstonePayloadAttributes;

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
        chain_spec: &ChainSpec,
        version: EngineApiMessageVersion,
    ) -> Result<(), EngineObjectValidationError> {
        unimplemented!()
    }
}

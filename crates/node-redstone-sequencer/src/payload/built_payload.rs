use reth_node_api::BuiltPayload;
use reth_primitives::{SealedBlock, U256};

use super::RedstoneBuiltPayload;

impl BuiltPayload for RedstoneBuiltPayload {
    /// Returns the built block (sealed)
    fn block(&self) -> &SealedBlock {
        unimplemented!()
    }

    /// Returns the fees collected for the built block
    fn fees(&self) -> U256 {
        unimplemented!()
    }
}

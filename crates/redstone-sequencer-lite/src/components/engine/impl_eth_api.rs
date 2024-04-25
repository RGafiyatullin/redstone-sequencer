use jsonrpsee::core::RpcResult;
use reth_primitives::Address;
use reth_primitives::BlockId;
use reth_primitives::BlockNumberOrTag;
use reth_primitives::Bytes;
use reth_primitives::B256;
use reth_primitives::U256;
use reth_primitives::U64;
use reth_rpc_types::AnyTransactionReceipt;
use reth_rpc_types::RichBlock;

use crate::api::EthApiServer;

use super::Api;

#[async_trait::async_trait]
impl EthApiServer for Api {
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        unimplemented!()
    }

    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }

    async fn chain_id(&self) -> RpcResult<Option<U64>> {
        Ok(Some(self.chain_spec.chain().id()).map(U64::from))
    }

    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256> {
        unimplemented!()
    }

    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256> {
        unimplemented!()
    }

    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        unimplemented!()
    }
}

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
use tokio::sync::oneshot;

use crate::api::EthApiServer;

use super::Api;
use super::Query;

#[async_trait::async_trait]
impl EthApiServer for Api {
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let query = Query::GetBalance { address, reply_tx };
        self.query_tx.try_send(query).expect("ew. tx");
        let balance = reply_rx.await.expect("ew. rx");
        Ok(balance)
    }

    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let query = Query::GetBlockByNumber {
            number,
            full,
            reply_tx,
        };
        self.query_tx.try_send(query).expect("ew. tx");
        let block_opt = reply_rx.await.expect("ew. rx")?;
        Ok(block_opt)
    }

    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let query = Query::GetBlockByHash {
            hash,
            full,
            reply_tx,
        };
        self.query_tx.try_send(query).expect("ew. tx");
        let block_opt = reply_rx.await.expect("ew. rx")?;
        Ok(block_opt)
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
        let (reply_tx, reply_rx) = oneshot::channel();
        let query = Query::GetTransactionCount { address, reply_tx };
        self.query_tx.try_send(query).expect("ew. tx");
        let nonce = reply_rx.await.expect("ew. rx");
        Ok(U256::from(nonce))
    }

    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        unimplemented!()
    }
}

use alloy_rpc_types::{BlockId, BlockNumberOrTag};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_primitives::{Address, Bytes, B256, U256, U64};
use reth_rpc_types::{AnyTransactionReceipt, RichBlock};
// use alloy_serde::JsonStorageKey;
// use reth_rpc_types::IP1186AccountProofResponse;

#[rpc(server, namespace = "eth")]
pub trait EthApi {
    #[method(name = "getBalance")]
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "getBlockByNumber")]
    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>>;

    #[method(name = "chainId")]
    async fn chain_id(&self) -> RpcResult<Option<U64>>;

    // #[method(name = "getProof")]
    // async fn get_proof(
    //     &self,
    //     address: Address,
    //     keys: Vec<JsonStorageKey>,
    //     block_number: Option<BlockId>,
    // ) -> RpcResult<EIP1186AccountProofResponse>;

    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256>;

    #[method(name = "getTransactionCount")]
    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256>;

    #[method(name = "getTransactionReceipt")]
    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>>;
}

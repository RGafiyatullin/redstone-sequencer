use alloy_primitives::Address;
use alloy_primitives::Bytes;
use alloy_primitives::B256;
use alloy_primitives::B64;
use alloy_primitives::U256;
use alloy_primitives::U64;
use alloy_rpc_types::state::StateOverride;
use alloy_rpc_types::AccessListWithGasUsed;
use alloy_rpc_types::BlockId;
use alloy_rpc_types::BlockNumberOrTag;
use alloy_rpc_types::BlockOverrides;
use alloy_rpc_types::Bundle;
use alloy_rpc_types::EIP1186AccountProofResponse;
use alloy_rpc_types::EthCallResponse;
use alloy_rpc_types::FeeHistory;
use alloy_rpc_types::Header;
use alloy_rpc_types::Index;
use alloy_rpc_types::RichBlock;
use alloy_rpc_types::StateContext;
use alloy_rpc_types::SyncStatus;
use alloy_rpc_types::Transaction;
use alloy_rpc_types::TransactionRequest;
use alloy_rpc_types::Work;
use jsonrpsee::core::RpcResult;
use reth_primitives::serde_helper::JsonStorageKey;
use reth_primitives::serde_helper::U64HexOrNumber;
use reth_rpc_api::EthApiServer;
use reth_rpc_types::AnyTransactionReceipt;

use super::Api;

#[async_trait::async_trait]
impl EthApiServer for Api {
    async fn protocol_version(&self) -> RpcResult<U64> {
        unimplemented!()
    }
    fn syncing(&self) -> RpcResult<SyncStatus> {
        unimplemented!()
    }
    async fn author(&self) -> RpcResult<Address> {
        unimplemented!()
    }
    fn accounts(&self) -> RpcResult<Vec<Address>> {
        unimplemented!()
    }
    fn block_number(&self) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn chain_id(&self) -> RpcResult<Option<U64>> {
        unimplemented!()
    }
    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }
    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }
    async fn block_transaction_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        unimplemented!()
    }
    async fn block_transaction_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        unimplemented!()
    }
    async fn block_uncles_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        unimplemented!()
    }
    async fn block_uncles_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        unimplemented!()
    }
    async fn block_receipts(
        &self,
        block_id: BlockId,
    ) -> RpcResult<Option<Vec<AnyTransactionReceipt>>> {
        unimplemented!()
    }
    async fn uncle_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }
    async fn uncle_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }
    async fn raw_transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Bytes>> {
        unimplemented!()
    }
    async fn transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Transaction>> {
        unimplemented!()
    }
    async fn raw_transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        unimplemented!()
    }
    async fn transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        unimplemented!()
    }
    async fn raw_transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        unimplemented!()
    }
    async fn transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        unimplemented!()
    }
    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        unimplemented!()
    }
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn storage_at(
        &self,
        address: Address,
        index: JsonStorageKey,
        block_number: Option<BlockId>,
    ) -> RpcResult<B256> {
        unimplemented!()
    }
    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn get_code(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<Bytes> {
        unimplemented!()
    }
    async fn header_by_number(&self, hash: BlockNumberOrTag) -> RpcResult<Option<Header>> {
        unimplemented!()
    }
    async fn header_by_hash(&self, hash: B256) -> RpcResult<Option<Header>> {
        unimplemented!()
    }
    async fn call(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_overrides: Option<StateOverride>,
        block_overrides: Option<Box<BlockOverrides>>,
    ) -> RpcResult<Bytes> {
        unimplemented!()
    }
    async fn call_many(
        &self,
        bundle: Bundle,
        state_context: Option<StateContext>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<Vec<EthCallResponse>> {
        unimplemented!()
    }
    async fn create_access_list(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
    ) -> RpcResult<AccessListWithGasUsed> {
        unimplemented!()
    }
    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn gas_price(&self) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn max_priority_fee_per_gas(&self) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn blob_base_fee(&self) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn fee_history(
        &self,
        block_count: U64HexOrNumber,
        newest_block: BlockNumberOrTag,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<FeeHistory> {
        unimplemented!()
    }
    async fn is_mining(&self) -> RpcResult<bool> {
        unimplemented!()
    }
    async fn hashrate(&self) -> RpcResult<U256> {
        unimplemented!()
    }
    async fn get_work(&self) -> RpcResult<Work> {
        unimplemented!()
    }
    async fn submit_hashrate(&self, hashrate: U256, id: B256) -> RpcResult<bool> {
        unimplemented!()
    }
    async fn submit_work(&self, nonce: B64, pow_hash: B256, mix_digest: B256) -> RpcResult<bool> {
        unimplemented!()
    }
    async fn send_transaction(&self, request: TransactionRequest) -> RpcResult<B256> {
        unimplemented!()
    }
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256> {
        unimplemented!()
    }
    async fn sign(&self, address: Address, message: Bytes) -> RpcResult<Bytes> {
        unimplemented!()
    }
    async fn sign_transaction(&self, transaction: TransactionRequest) -> RpcResult<Bytes> {
        unimplemented!()
    }
    async fn sign_typed_data(&self, address: Address, data: serde_json::Value) -> RpcResult<Bytes> {
        unimplemented!()
    }

    async fn get_proof(
        &self,
        address: Address,
        keys: Vec<JsonStorageKey>,
        block_number: Option<BlockId>,
    ) -> RpcResult<EIP1186AccountProofResponse> {
        unimplemented!()
    }
}

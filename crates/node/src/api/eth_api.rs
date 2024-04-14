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
use reth_rpc_api::EthApiClient;
use reth_rpc_api::EthApiServer;
use reth_rpc_types::AnyTransactionReceipt;

use super::Api;

#[async_trait::async_trait]
impl EthApiServer for Api {
    async fn protocol_version(&self) -> RpcResult<U64> {
        self.0
            .engine_api_client
            .protocol_version()
            .await
            .map_err(to_error_object)
    }
    fn syncing(&self) -> RpcResult<SyncStatus> {
        Ok(SyncStatus::None)
    }
    async fn author(&self) -> RpcResult<Address> {
        self.0
            .engine_api_client
            .author()
            .await
            .map_err(to_error_object)
    }
    fn accounts(&self) -> RpcResult<Vec<Address>> {
        Ok(Default::default())
    }
    fn block_number(&self) -> RpcResult<U256> {
        Ok(Default::default())
    }
    async fn chain_id(&self) -> RpcResult<Option<U64>> {
        self.0
            .engine_api_client
            .chain_id()
            .await
            .map_err(to_error_object)
    }
    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        self.0
            .engine_api_client
            .block_by_hash(hash, full)
            .await
            .map_err(to_error_object)
    }
    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        self.0
            .engine_api_client
            .block_by_number(number, full)
            .await
            .map_err(to_error_object)
    }
    async fn block_transaction_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        self.0
            .engine_api_client
            .block_transaction_count_by_hash(hash)
            .await
            .map_err(to_error_object)
    }
    async fn block_transaction_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        self.0
            .engine_api_client
            .block_transaction_count_by_number(number)
            .await
            .map_err(to_error_object)
    }
    async fn block_uncles_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        self.0
            .engine_api_client
            .block_uncles_count_by_hash(hash)
            .await
            .map_err(to_error_object)
    }
    async fn block_uncles_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        self.0
            .engine_api_client
            .block_uncles_count_by_number(number)
            .await
            .map_err(to_error_object)
    }
    async fn block_receipts(
        &self,
        block_id: BlockId,
    ) -> RpcResult<Option<Vec<AnyTransactionReceipt>>> {
        self.0
            .engine_api_client
            .block_receipts(block_id)
            .await
            .map_err(to_error_object)
    }
    async fn uncle_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        self.0
            .engine_api_client
            .uncle_by_block_hash_and_index(hash, index)
            .await
            .map_err(to_error_object)
    }
    async fn uncle_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        self.0
            .engine_api_client
            .uncle_by_block_number_and_index(number, index)
            .await
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Bytes>> {
        self.0
            .engine_api_client
            .raw_transaction_by_hash(hash)
            .await
            .map_err(to_error_object)
    }
    async fn transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Transaction>> {
        self.0
            .engine_api_client
            .transaction_by_hash(hash)
            .await
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        self.0
            .engine_api_client
            .raw_transaction_by_block_hash_and_index(hash, index)
            .await
            .map_err(to_error_object)
    }
    async fn transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        self.0
            .engine_api_client
            .transaction_by_block_hash_and_index(hash, index)
            .await
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        self.0
            .engine_api_client
            .raw_transaction_by_block_number_and_index(number, index)
            .await
            .map_err(to_error_object)
    }
    async fn transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        self.0
            .engine_api_client
            .transaction_by_block_number_and_index(number, index)
            .await
            .map_err(to_error_object)
    }
    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        self.0
            .engine_api_client
            .transaction_receipt(hash)
            .await
            .map_err(to_error_object)
    }
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .balance(address, block_number)
            .await
            .map_err(to_error_object)
    }
    async fn storage_at(
        &self,
        address: Address,
        index: JsonStorageKey,
        block_number: Option<BlockId>,
    ) -> RpcResult<B256> {
        self.0
            .engine_api_client
            .storage_at(address, index, block_number)
            .await
            .map_err(to_error_object)
    }
    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .transaction_count(address, block_number)
            .await
            .map_err(to_error_object)
    }
    async fn get_code(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<Bytes> {
        self.0
            .engine_api_client
            .get_code(address, block_number)
            .await
            .map_err(to_error_object)
    }
    async fn header_by_number(&self, hash: BlockNumberOrTag) -> RpcResult<Option<Header>> {
        self.0
            .engine_api_client
            .header_by_number(hash)
            .await
            .map_err(to_error_object)
    }
    async fn header_by_hash(&self, hash: B256) -> RpcResult<Option<Header>> {
        self.0
            .engine_api_client
            .header_by_hash(hash)
            .await
            .map_err(to_error_object)
    }
    async fn call(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_overrides: Option<StateOverride>,
        block_overrides: Option<Box<BlockOverrides>>,
    ) -> RpcResult<Bytes> {
        self.0
            .engine_api_client
            .call(request, block_number, state_overrides, block_overrides)
            .await
            .map_err(to_error_object)
    }
    async fn call_many(
        &self,
        bundle: Bundle,
        state_context: Option<StateContext>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<Vec<EthCallResponse>> {
        self.0
            .engine_api_client
            .call_many(bundle, state_context, state_override)
            .await
            .map_err(to_error_object)
    }
    async fn create_access_list(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
    ) -> RpcResult<AccessListWithGasUsed> {
        self.0
            .engine_api_client
            .create_access_list(request, block_number)
            .await
            .map_err(to_error_object)
    }
    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .estimate_gas(request, block_number, state_override)
            .await
            .map_err(to_error_object)
    }
    async fn gas_price(&self) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .gas_price()
            .await
            .map_err(to_error_object)
    }
    async fn max_priority_fee_per_gas(&self) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .max_priority_fee_per_gas()
            .await
            .map_err(to_error_object)
    }
    async fn blob_base_fee(&self) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .blob_base_fee()
            .await
            .map_err(to_error_object)
    }
    async fn fee_history(
        &self,
        block_count: U64HexOrNumber,
        newest_block: BlockNumberOrTag,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<FeeHistory> {
        self.0
            .engine_api_client
            .fee_history(block_count, newest_block, reward_percentiles)
            .await
            .map_err(to_error_object)
    }
    async fn is_mining(&self) -> RpcResult<bool> {
        self.0
            .engine_api_client
            .is_mining()
            .await
            .map_err(to_error_object)
    }
    async fn hashrate(&self) -> RpcResult<U256> {
        self.0
            .engine_api_client
            .hashrate()
            .await
            .map_err(to_error_object)
    }
    async fn get_work(&self) -> RpcResult<Work> {
        self.0
            .engine_api_client
            .get_work()
            .await
            .map_err(to_error_object)
    }
    async fn submit_hashrate(&self, hashrate: U256, id: B256) -> RpcResult<bool> {
        self.0
            .engine_api_client
            .submit_hashrate(hashrate, id)
            .await
            .map_err(to_error_object)
    }
    async fn submit_work(&self, nonce: B64, pow_hash: B256, mix_digest: B256) -> RpcResult<bool> {
        self.0
            .engine_api_client
            .submit_work(nonce, pow_hash, mix_digest)
            .await
            .map_err(to_error_object)
    }
    async fn send_transaction(&self, request: TransactionRequest) -> RpcResult<B256> {
        self.0
            .engine_api_client
            .send_transaction(request)
            .await
            .map_err(to_error_object)
    }
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256> {
        self.0
            .engine_api_client
            .send_raw_transaction(bytes)
            .await
            .map_err(to_error_object)
    }
    async fn sign(&self, address: Address, message: Bytes) -> RpcResult<Bytes> {
        self.0
            .engine_api_client
            .sign(address, message)
            .await
            .map_err(to_error_object)
    }
    async fn sign_transaction(&self, transaction: TransactionRequest) -> RpcResult<Bytes> {
        self.0
            .engine_api_client
            .sign_transaction(transaction)
            .await
            .map_err(to_error_object)
    }
    async fn sign_typed_data(&self, address: Address, data: serde_json::Value) -> RpcResult<Bytes> {
        self.0
            .engine_api_client
            .sign_typed_data(address, data)
            .await
            .map_err(to_error_object)
    }

    async fn get_proof(
        &self,
        address: Address,
        keys: Vec<JsonStorageKey>,
        block_number: Option<BlockId>,
    ) -> RpcResult<EIP1186AccountProofResponse> {
        self.0
            .engine_api_client
            .get_proof(address, keys, block_number)
            .await
            .map_err(to_error_object)
    }
}

fn to_error_object(error: jsonrpsee::core::Error) -> jsonrpsee::types::ErrorObjectOwned {
    use jsonrpsee::core::Error;
    use jsonrpsee::types::ErrorObject;

    if let Error::Call(error_object) = error {
        error_object
    } else {
        ErrorObject::owned(-32000, error.to_string(), Some(""))
    }
}

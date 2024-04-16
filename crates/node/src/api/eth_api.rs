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

use super::to_error_object;
use super::Api;

impl Api {
    pub fn backend_eth_api(&self) -> &impl EthApiClient {
        &self.0.anonymous_client
    }
}

#[async_trait::async_trait]
impl EthApiServer for Api {
    async fn protocol_version(&self) -> RpcResult<U64> {
        tracing::trace!(target: "node::api::eth_api::protocol_version", "CALL []");
        self.backend_eth_api()
            .protocol_version()
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::protocol_version", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::protocol_version", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    fn syncing(&self) -> RpcResult<SyncStatus> {
        tracing::error!(target: "node::api::eth_api::syncing", "CALL [] (NOT IMPLEMENTED)");
        Ok(SyncStatus::None)
    }
    async fn author(&self) -> RpcResult<Address> {
        tracing::trace!(target: "node::api::eth_api::author", "CALL []");
        self.backend_eth_api()
            .author()
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::author", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::author", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    fn accounts(&self) -> RpcResult<Vec<Address>> {
        tracing::error!(target: "node::api::eth_api::accounts", "CALL [] (NOT IMPLEMENTED)");
        Ok(Default::default())
    }
    fn block_number(&self) -> RpcResult<U256> {
        let block_number = *self
            .0
            .current_block_number
            .read()
            .expect("rw-lock.read -> poisoned");
        tracing::trace!(target: "node::api::eth_api::block_number", "CALL [] -> {}", block_number);
        Ok(block_number)
    }
    async fn chain_id(&self) -> RpcResult<Option<U64>> {
        tracing::trace!(target: "node::api::eth_api::chain_id", "CALL []");
        self.backend_eth_api()
            .chain_id()
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::chain_id", "RET.OK: {:?}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::chain_id", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        tracing::trace!(target: "node::api::eth_api::block_by_hash", "CALL [hash: {}; full: {}]", hash, full);
        self.backend_eth_api()
            .block_by_hash(hash, full)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        tracing::trace!(target: "node::api::eth_api::block_by_number", "CALL [number: {}; full: {}]", number, full);
        self.backend_eth_api()
            .block_by_number(number, full)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_by_number", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_by_number", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_transaction_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_hash", "CALL [hash: {}]", hash);
        self.backend_eth_api()
            .block_transaction_count_by_hash(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_transaction_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_number", "CALL [number: {}]", number);
        self.backend_eth_api()
            .block_transaction_count_by_number(number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_number", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_transaction_count_by_number", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_uncles_count_by_hash(&self, hash: B256) -> RpcResult<Option<U256>> {
        tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_hash", "CALL [hash: {}]", hash);
        self.backend_eth_api()
            .block_uncles_count_by_hash(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_uncles_count_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> RpcResult<Option<U256>> {
        tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_number", "CALL [number: {}]", number);
        self.backend_eth_api()
            .block_uncles_count_by_number(number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_number", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_uncles_count_by_number", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn block_receipts(
        &self,
        block_id: BlockId,
    ) -> RpcResult<Option<Vec<AnyTransactionReceipt>>> {
        tracing::trace!(target: "node::api::eth_api::block_receipts", "CALL [block_id: {:?}]", block_id);
        self.backend_eth_api()
            .block_receipts(block_id)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::block_receipts", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::block_receipts", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn uncle_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        tracing::trace!(target: "node::api::eth_api::uncle_by_block_hash_and_index", "CALL [hash: {}; index: {:?}]", hash, index);
        self.backend_eth_api()
            .uncle_by_block_hash_and_index(hash, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::uncle_by_block_hash_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::uncle_by_block_hash_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn uncle_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<RichBlock>> {
        tracing::trace!(target: "node::api::eth_api::uncle_by_block_number_and_index", "CALL [number: {}; index: {:?}]", number, index);
        self.backend_eth_api()
            .uncle_by_block_number_and_index(number, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::uncle_by_block_number_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::uncle_by_block_number_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Bytes>> {
        tracing::trace!(target: "node::api::eth_api::raw_transaction_by_hash", "CALL []", );
        self.backend_eth_api()
            .raw_transaction_by_hash(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn transaction_by_hash(&self, hash: B256) -> RpcResult<Option<Transaction>> {
        tracing::trace!(target: "node::api::eth_api::transaction_by_hash", "CALL []", );
        self.backend_eth_api()
            .transaction_by_hash(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::transaction_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::transaction_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_hash_and_index", "CALL []", );
        self.backend_eth_api()
            .raw_transaction_by_block_hash_and_index(hash, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_hash_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_hash_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        tracing::trace!(target: "node::api::eth_api::transaction_by_block_hash_and_index", "CALL []", );
        self.backend_eth_api()
            .transaction_by_block_hash_and_index(hash, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::transaction_by_block_hash_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::transaction_by_block_hash_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn raw_transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Bytes>> {
        tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_number_and_index", "CALL []", );
        self.backend_eth_api()
            .raw_transaction_by_block_number_and_index(number, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_number_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::raw_transaction_by_block_number_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> RpcResult<Option<Transaction>> {
        tracing::trace!(target: "node::api::eth_api::transaction_by_block_number_and_index", "CALL []", );
        self.backend_eth_api()
            .transaction_by_block_number_and_index(number, index)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::transaction_by_block_number_and_index", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::transaction_by_block_number_and_index", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        tracing::trace!(target: "node::api::eth_api::transaction_receipt", "CALL []", );
        self.backend_eth_api()
            .transaction_receipt(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::transaction_receipt", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::transaction_receipt", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::balance", "CALL []", );
        self.backend_eth_api()
            .balance(address, block_number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::balance", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::balance", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn storage_at(
        &self,
        address: Address,
        index: JsonStorageKey,
        block_number: Option<BlockId>,
    ) -> RpcResult<B256> {
        tracing::trace!(target: "node::api::eth_api::storage_at", "CALL []", );
        self.backend_eth_api()
            .storage_at(address, index, block_number)
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::storage_at", "RET.OK: {}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::storage_at", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::transaction_count", "CALL []", );
        self.backend_eth_api()
            .transaction_count(address, block_number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::transaction_count", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::transaction_count", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn get_code(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<Bytes> {
        tracing::trace!(target: "node::api::eth_api::get_code", "CALL []", );
        self.backend_eth_api()
            .get_code(address, block_number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::get_code", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::get_code", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn header_by_number(&self, hash: BlockNumberOrTag) -> RpcResult<Option<Header>> {
        tracing::trace!(target: "node::api::eth_api::header_by_number", "CALL []", );
        self.backend_eth_api()
            .header_by_number(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::header_by_number", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::header_by_number", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn header_by_hash(&self, hash: B256) -> RpcResult<Option<Header>> {
        tracing::trace!(target: "node::api::eth_api::header_by_hash", "CALL []", );
        self.backend_eth_api()
            .header_by_hash(hash)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::header_by_hash", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::header_by_hash", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn call(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_overrides: Option<StateOverride>,
        block_overrides: Option<Box<BlockOverrides>>,
    ) -> RpcResult<Bytes> {
        tracing::trace!(target: "node::api::eth_api::call", "CALL []", );
        self.backend_eth_api()
            .call(request, block_number, state_overrides, block_overrides)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::call", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::call", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn call_many(
        &self,
        bundle: Bundle,
        state_context: Option<StateContext>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<Vec<EthCallResponse>> {
        tracing::trace!(target: "node::api::eth_api::call_many", "CALL []", );
        self.backend_eth_api()
            .call_many(bundle, state_context, state_override)
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::call_many", "RET.OK: {:?}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::call_many", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn create_access_list(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
    ) -> RpcResult<AccessListWithGasUsed> {
        tracing::trace!(target: "node::api::eth_api::create_access_list", "CALL []", );
        self.backend_eth_api()
            .create_access_list(request, block_number)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::create_access_list", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::create_access_list", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::estimate_gas", "CALL []", );
        self.backend_eth_api()
            .estimate_gas(request, block_number, state_override)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::estimate_gas", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::estimate_gas", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn gas_price(&self) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::gas_price", "CALL []", );
        self.backend_eth_api()
            .gas_price()
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::REPLACEME", "RET.OK: {}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::REPLACEME", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn max_priority_fee_per_gas(&self) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::gas_price", "CALL []", );
        self.backend_eth_api()
            .max_priority_fee_per_gas()
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::max_priority_fee_per_gas", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::max_priority_fee_per_gas", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn blob_base_fee(&self) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::blob_base_fee", "CALL []", );
        self.backend_eth_api()
            .blob_base_fee()
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::blob_base_fee", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::blob_base_fee", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn fee_history(
        &self,
        block_count: U64HexOrNumber,
        newest_block: BlockNumberOrTag,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<FeeHistory> {
        tracing::trace!(target: "node::api::eth_api::fee_history", "CALL []", );
        self.backend_eth_api()
            .fee_history(block_count, newest_block, reward_percentiles)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::fee_history", "RET.OK: {:?}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::fee_history", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn is_mining(&self) -> RpcResult<bool> {
        tracing::trace!(target: "node::api::eth_api::is_mining", "CALL []", );
        self.backend_eth_api()
            .is_mining()
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::is_mining", "RET.OK: {}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::is_mining", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn hashrate(&self) -> RpcResult<U256> {
        tracing::trace!(target: "node::api::eth_api::hashrate", "CALL []", );
        self.backend_eth_api()
            .hashrate()
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::hashrate", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::hashrate", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn get_work(&self) -> RpcResult<Work> {
        tracing::trace!(target: "node::api::eth_api::get_work", "CALL []", );
        self.backend_eth_api()
            .get_work()
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::get_work", "RET.OK: {:?}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::get_work", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn submit_hashrate(&self, hashrate: U256, id: B256) -> RpcResult<bool> {
        tracing::trace!(target: "node::api::eth_api::submit_hashrate", "CALL []", );
        self.backend_eth_api()
            .submit_hashrate(hashrate, id)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::submit_hashrate", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::submit_hashrate", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn submit_work(&self, nonce: B64, pow_hash: B256, mix_digest: B256) -> RpcResult<bool> {
        tracing::trace!(target: "node::api::eth_api::submit_work", "CALL []", );
        self.backend_eth_api()
            .submit_work(nonce, pow_hash, mix_digest)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::submit_work", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::submit_work", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn send_transaction(&self, request: TransactionRequest) -> RpcResult<B256> {
        tracing::trace!(target: "node::api::eth_api::send_transaction", "CALL [request: {:?}]", request);
        self.backend_eth_api()
            .send_transaction(request)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::send_transaction", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::send_transaction", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256> {
        tracing::trace!(target: "node::api::eth_api::send_raw_transaction", "CALL [bytes: {:?}]", bytes);
        self.backend_eth_api()
            .send_raw_transaction(bytes)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::send_raw_transaction", "RET.OK: {}", ok))
            .inspect_err(|err| tracing::trace!(target: "node::api::eth_api::send_raw_transaction", "RET.ERR: {}", err))
            .map_err(to_error_object)
    }
    async fn sign(&self, address: Address, message: Bytes) -> RpcResult<Bytes> {
        tracing::trace!(target: "node::api::eth_api::sign", "CALL []", );
        self.backend_eth_api()
            .sign(address, message)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::sign", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::sign", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn sign_transaction(&self, transaction: TransactionRequest) -> RpcResult<Bytes> {
        tracing::trace!(target: "node::api::eth_api::sign_transaction", "CALL []", );
        self.backend_eth_api()
            .sign_transaction(transaction)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::sign_transaction", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::sign_transaction", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
    async fn sign_typed_data(&self, address: Address, data: serde_json::Value) -> RpcResult<Bytes> {
        tracing::trace!(target: "node::api::eth_api::sign_typed_data", "CALL []", );
        self.backend_eth_api()
            .sign_typed_data(address, data)
            .await
            .inspect(|ok| tracing::trace!(target: "node::api::eth_api::sign_typed_data", "RET.OK: {}", ok))
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::sign_typed_data", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }

    async fn get_proof(
        &self,
        address: Address,
        keys: Vec<JsonStorageKey>,
        block_number: Option<BlockId>,
    ) -> RpcResult<EIP1186AccountProofResponse> {
        tracing::trace!(target: "node::api::eth_api::get_proof", "CALL []", );
        self.backend_eth_api()
            .get_proof(address, keys, block_number)
            .await
            .inspect(
                |ok| tracing::trace!(target: "node::api::eth_api::get_proof", "RET.OK: {:?}", ok),
            )
            .inspect_err(
                |err| tracing::trace!(target: "node::api::eth_api::get_proof", "RET.ERR: {}", err),
            )
            .map_err(to_error_object)
    }
}

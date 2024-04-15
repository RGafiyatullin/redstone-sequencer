use alloy_rpc_types::Filter;
use alloy_rpc_types::Log;
use jsonrpsee::core::RpcResult;
use reth_rpc_api::{EthFilterApiClient, EthFilterApiServer};
use reth_rpc_types::{FilterChanges, FilterId, PendingTransactionFilterKind};

use super::{to_error_object, Api};

impl Api {
    pub fn backend_eth_filter_api(&self) -> &impl EthFilterApiClient {
        &self.0.anonymous_client
    }
}

#[async_trait::async_trait]
impl EthFilterApiServer for Api {
    async fn new_filter(&self, filter: Filter) -> RpcResult<FilterId> {
        self.backend_eth_filter_api()
            .new_filter(filter)
            .await
            .map_err(to_error_object)
    }
    async fn new_block_filter(&self) -> RpcResult<FilterId> {
        self.backend_eth_filter_api()
            .new_block_filter()
            .await
            .map_err(to_error_object)
    }
    async fn new_pending_transaction_filter(
        &self,
        kind: Option<PendingTransactionFilterKind>,
    ) -> RpcResult<FilterId> {
        self.backend_eth_filter_api()
            .new_pending_transaction_filter(kind)
            .await
            .map_err(to_error_object)
    }
    async fn filter_changes(&self, id: FilterId) -> RpcResult<FilterChanges> {
        self.backend_eth_filter_api()
            .filter_changes(id)
            .await
            .map_err(to_error_object)
    }
    async fn filter_logs(&self, id: FilterId) -> RpcResult<Vec<Log>> {
        self.backend_eth_filter_api()
            .filter_logs(id)
            .await
            .map_err(to_error_object)
    }
    async fn uninstall_filter(&self, id: FilterId) -> RpcResult<bool> {
        self.backend_eth_filter_api()
            .uninstall_filter(id)
            .await
            .map_err(to_error_object)
    }
    async fn logs(&self, filter: Filter) -> RpcResult<Vec<Log>> {
        self.backend_eth_filter_api()
            .logs(filter)
            .await
            .map_err(to_error_object)
    }
}

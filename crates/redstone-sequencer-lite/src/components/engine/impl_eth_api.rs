use alloy_serde::U64HexOrNumber;
use jsonrpsee::core::RpcResult;
use reth_node_api::ConfigureEvm;
use reth_node_api::ConfigureEvmEnv;
use reth_primitives::Address;
use reth_primitives::BlockId;
use reth_primitives::BlockNumberOrTag;
use reth_primitives::Bytes;
use reth_primitives::PooledTransactionsElement;
use reth_primitives::B256;
use reth_primitives::U256;
use reth_primitives::U64;
use reth_provider::StateProviderFactory;
use reth_provider::TransactionVariant;
use reth_rpc::eth::error::EthApiError;
use reth_rpc_types::state::StateOverride;
use reth_rpc_types::AnyTransactionReceipt;
use reth_rpc_types::BlockTransactionsKind;
use reth_rpc_types::FeeHistory;
use reth_rpc_types::RichBlock;
use reth_rpc_types::TransactionRequest;
use tokio::sync::oneshot;
use tracing::info;
use tracing::warn;

use crate::api::EthApiServer;
use crate::AnyError;

use super::Blockchain;
use super::Engine;
use super::Query;
use super::State;

#[async_trait::async_trait]
impl<B, V> EthApiServer for Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
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

    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        unimplemented!()
    }

    async fn chain_id(&self) -> RpcResult<Option<U64>> {
        Ok(Some(self.0.read().await.args.chain_spec.chain().id()).map(U64::from))
    }

    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_number: Option<BlockId>,
        state_override: Option<StateOverride>,
    ) -> RpcResult<U256> {
        Ok(Default::default())
    }

    async fn fee_history(
        &self,
        block_count: U64HexOrNumber,
        newest_block: BlockNumberOrTag,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<FeeHistory> {
        Ok(Default::default())
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

impl<B, V> State<B, V>
where
    B: Blockchain,
{
    pub(super) async fn handle_get_transaction_count(
        &self,
        address: Address,
        reply_tx: oneshot::Sender<u64>,
    ) -> Result<(), AnyError> {
        let nonce = if let Some(nonce) = self.nonces.get(&address) {
            nonce
        } else {
            self.blockchain()
                .latest()?
                .account_nonce(address)?
                .unwrap_or_default()
        };
        let _ = reply_tx
            .send(nonce)
            .inspect_err(|_| warn!("oneshot-tx: closed"));

        Ok(())
    }

    pub(super) async fn handle_get_balance(
        &self,
        address: Address,
        reply_tx: oneshot::Sender<U256>,
    ) -> Result<(), AnyError> {
        let balance = self
            .blockchain()
            .latest()?
            .account_balance(address)?
            .unwrap_or_default();
        let _ = reply_tx
            .send(balance)
            .inspect_err(|_| warn!("oneshot-tx: closed"));

        Ok(())
    }

    pub(super) async fn handle_get_block_by_number(
        &self,
        number_or_tag: BlockNumberOrTag,
        full: bool,
        reply_tx: oneshot::Sender<Result<Option<RichBlock>, EthApiError>>,
    ) -> Result<(), AnyError> {
        fn handle(
            blockchain: impl Blockchain,
            number_or_tag: BlockNumberOrTag,
            kind: BlockTransactionsKind,
        ) -> Result<Option<RichBlock>, EthApiError> {
            let block_id: BlockId = number_or_tag.into();
            let Some(block_hash) = blockchain.block_hash_for_id(block_id)? else {
                return Ok(None);
            };
            let Some(block_with_senders) =
                blockchain.block_with_senders(block_hash.into(), TransactionVariant::WithHash)?
            else {
                return Ok(None);
            };

            let total_difficulty = blockchain
                .header_td_by_number(block_with_senders.number)?
                .expect(&format!(
                    "could not get total-difficulty for {}",
                    block_hash
                ));

            let block = reth_rpc_types_compat::block::from_block(
                block_with_senders,
                total_difficulty,
                kind,
                Some(block_hash),
            )?;

            Ok(Some(block.into()))
        }

        let _ = reply_tx.send(handle(self.blockchain(), number_or_tag, full.into()));

        Ok(())
    }

    pub(super) async fn handle_get_block_by_hash(
        &self,
        block_hash: B256,
        full: bool,
        reply_tx: oneshot::Sender<Result<Option<RichBlock>, EthApiError>>,
    ) -> Result<(), AnyError> {
        fn handle(
            blockchain: impl Blockchain,
            block_hash: B256,
            kind: BlockTransactionsKind,
        ) -> Result<Option<RichBlock>, EthApiError> {
            let Some(block_with_senders) =
                blockchain.block_with_senders(block_hash.into(), TransactionVariant::WithHash)?
            else {
                return Ok(None);
            };

            let total_difficulty = blockchain
                .header_td_by_number(block_with_senders.number)?
                .expect(&format!(
                    "could not get total-difficulty for {}",
                    block_hash
                ));

            let block = reth_rpc_types_compat::block::from_block(
                block_with_senders,
                total_difficulty,
                kind,
                Some(block_hash),
            )?;

            Ok(Some(block.into()))
        }

        let _ = reply_tx.send(handle(self.blockchain(), block_hash, full.into()));

        Ok(())
    }

    pub(super) async fn handle_transaction_add(
        &mut self,
        tx: PooledTransactionsElement,
    ) -> Result<(), AnyError> {
        let Some(from_address) = tx.recover_signer() else {
            warn!(tx_hash = ?tx.hash(), "could not recover signer");
            return Ok(());
        };
        fn fetch_nonce<S: StateProviderFactory>(
            blockchain: &S,
            address: Address,
        ) -> Result<u64, AnyError> {
            let state = blockchain.latest()?;
            let nonce = state.account_nonce(address)?;
            Ok(nonce.unwrap_or_default())
        }
        self.nonces.ensure_for_address(from_address, |address| {
            fetch_nonce(&self.args.blockchain, address)
        })?;

        self.tx_pool.add(&mut self.nonces, tx)?;

        info!("nonces: {:#?}", self.nonces);
        info!("tx_pool: {:#?}", self.tx_pool);

        Ok(())
    }

    pub(super) async fn handle_get_transaction_receipt(
        &self,
        tx_hash: B256,
        reply_tx: oneshot::Sender<Result<Option<AnyTransactionReceipt>, EthApiError>>,
    ) -> Result<(), AnyError> {
        let _ = reply_tx.send(Ok(None));
        Ok(())
    }
}

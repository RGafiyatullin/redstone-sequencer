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
use reth_provider::TransactionVariant;
use reth_rpc::eth::error::EthApiError;
use reth_rpc::eth::error::RpcInvalidTransactionError;
use reth_rpc_types::state::StateOverride;
use reth_rpc_types::AnyTransactionReceipt;
use reth_rpc_types::FeeHistory;
use reth_rpc_types::RichBlock;
use reth_rpc_types::TransactionRequest;
use tracing::debug;
use tracing::warn;

use crate::api::EthApiServer;

use super::Blockchain;
use super::Engine;
use super::State;

#[async_trait::async_trait]
impl<B, V> EthApiServer for Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    async fn balance(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<U256> {
        let r = self.0.read().await;
        let state = if let Some(block_id) = block_number {
            r.blockchain().state_by_block_id(block_id)
        } else {
            r.blockchain().latest()
        }
        .map_err(Into::into)
        .map_err(EthApiError::Internal)?;
        let balance = state
            .account_balance(address)
            .map_err(Into::into)
            .map_err(EthApiError::Internal)?
            .unwrap_or_default();

        Ok(balance)
    }

    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RichBlock>> {
        let r = self.0.read().await;
        let block_id: BlockId = number.into();
        let Some(block_hash) = r
            .blockchain()
            .block_hash_for_id(block_id)
            .map_err(Into::into)
            .map_err(EthApiError::Internal)?
        else {
            return Ok(None);
        };

        self.block_by_hash(block_hash, full).await
    }

    async fn block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<RichBlock>> {
        let r = self.0.read().await;

        let Some(block_with_senders) = r
            .blockchain()
            .block_with_senders(hash.into(), TransactionVariant::WithHash)
            .map_err(Into::into)
            .map_err(EthApiError::Internal)?
        else {
            return Ok(None);
        };

        let total_difficulty = r
            .blockchain()
            .header_td_by_number(block_with_senders.number)
            .map_err(Into::into)
            .map_err(EthApiError::Internal)?
            .expect(&format!("could not get total-difficulty for {}", hash));

        let block = reth_rpc_types_compat::block::from_block(
            block_with_senders,
            total_difficulty,
            full.into(),
            Some(hash),
        )
        .map_err(EthApiError::InvalidBlockData)?;

        Ok(Some(block.into()))
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
        let tx = PooledTransactionsElement::decode_enveloped(&mut bytes.as_ref())
            .map_err(|e| e.to_string())
            .map_err(EthApiError::InvalidParams)?;

        let tx_hash = *tx.hash();

        let from_address = tx
            .recover_signer()
            .ok_or_else(|| RpcInvalidTransactionError::TxTypeNotSupported)
            .map_err(EthApiError::InvalidTransaction)?;

        let mut w = self.0.write().await;
        let State {
            args,
            nonces,
            tx_pool,
            payloads,
        } = &mut *w;

        nonces
            .ensure_for_address(from_address, |address| {
                let state = args.blockchain.latest()?;
                let nonce = state.account_nonce(from_address)?;
                Ok(nonce.unwrap_or_default())
            })
            .map_err(EthApiError::Internal)?;

        tx_pool
            .add(nonces, tx)
            .map_err(|e| e.to_string())
            .map_err(EthApiError::InvalidParams)?;

        debug!(
            ?tx_hash,
            scheduled_count = tx_pool.scheduled_count(),
            pending_count = tx_pool.pending_count(),
            "Added transaction into pool."
        );

        if payloads.len() == 1 {
            let (payload_id, builder) = payloads
                .iter_mut()
                .next()
                .expect("len checked right before");
            debug!(?payload_id, "Selected payload builder");
            for tx in tx_pool.scheduled_drain() {
                let tx_hash = *tx.hash();
                if let Err(err) = builder.process_transaction(tx.into_transaction(), false) {
                    warn!(?payload_id, ?tx_hash, %err, "error processing transaction: ")
                } else {
                    debug!(?payload_id, ?tx_hash, "transaction processed");
                }
            }
        } else {
            warn!(
                payloads_len = payloads.len(),
                "Could not select a payload builder"
            )
        }

        Ok(tx_hash)
    }

    async fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> RpcResult<U256> {
        let r = self.0.read().await;
        let nonce = if let Some(nonce) = r.nonces.get(&address) {
            nonce
        } else {
            r.blockchain()
                .latest()
                .map_err(Into::into)
                .map_err(EthApiError::Internal)?
                .account_nonce(address)
                .map_err(Into::into)
                .map_err(EthApiError::Internal)?
                .unwrap_or_default()
        };
        Ok(U256::from(nonce))
    }

    async fn transaction_receipt(&self, hash: B256) -> RpcResult<Option<AnyTransactionReceipt>> {
        Ok(None)
    }
}

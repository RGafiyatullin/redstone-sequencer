use std::collections::HashMap;

use alloy_serde::U64HexOrNumber;
use jsonrpsee::core::RpcResult;
use reth_node_api::ConfigureEvm;
use reth_node_api::ConfigureEvmEnv;
use reth_node_api::PayloadBuilderAttributes;
use reth_payload_builder::PayloadId;
use reth_primitives::Address;
use reth_primitives::BlockId;
use reth_primitives::BlockNumberOrTag;
use reth_primitives::Bytes;
use reth_primitives::PooledTransactionsElement;
use reth_primitives::Receipt;
use reth_primitives::TransactionKind;
use reth_primitives::TransactionMeta;
use reth_primitives::TransactionSigned;
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

use super::payload_builder::RedstonePayloadBuilder;
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
    async fn block_number(&self) -> RpcResult<U256> {
        let block_number = self
            .0
            .read()
            .await
            .blockchain()
            .block_number_for_id(BlockId::latest())
            .map_err(Into::into)
            .map_err(EthApiError::Internal)?;

        Ok(block_number.map(U256::from).unwrap_or(U256::ZERO))
    }

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
        _request: TransactionRequest,
        _block_number: Option<BlockId>,
        _state_override: Option<StateOverride>,
    ) -> RpcResult<U256> {
        Ok(Default::default())
    }

    async fn fee_history(
        &self,
        _block_count: U64HexOrNumber,
        _newest_block: BlockNumberOrTag,
        _reward_percentiles: Option<Vec<f64>>,
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
            .ensure_for_address(from_address, |_address| {
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
                match builder.process_transaction(tx.into_transaction(), false) {
                    Ok(receipt) => debug!(?payload_id, ?tx_hash, ?receipt, "transaction processed"),
                    Err(err) => {
                        warn!(?payload_id, ?tx_hash, %err, "error processing transaction: ")
                    }
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
        _block_number: Option<BlockId>,
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
        let r = self.0.read().await;

        fn from_db(
            blockchain: &impl Blockchain,
            hash: B256,
        ) -> Result<Option<(TransactionSigned, TransactionMeta, Receipt)>, EthApiError> {
            let opt = blockchain
                .transaction_by_hash_with_meta(hash)?
                .zip(blockchain.receipt_by_hash(hash)?)
                .map(|((tx, meta), receipt)| (tx, meta, receipt));
            Ok(opt)
        }

        fn from_builder<A, B, V>(
            builders: &HashMap<PayloadId, RedstonePayloadBuilder<A, B, V>>,
            hash: B256,
        ) -> Option<(TransactionSigned, Receipt)>
        where
            A: PayloadBuilderAttributes,
            B: Blockchain,
            V: ConfigureEvm + ConfigureEvmEnv,
        {
            builders
                .values()
                .map(|builder| builder.transactions_with_receipts())
                .flatten()
                .find(|(tx, _)| tx.hash() == hash)
                .map(|(tx, receipt)| (tx.to_owned(), receipt.to_owned()))
        }

        fn make_any_transaction_receipt(
            hash: B256,
            transaction: TransactionSigned,
            meta: TransactionMeta,
            receipt: Receipt,
        ) -> Result<AnyTransactionReceipt, EthApiError> {
            let all_receipts: &[Receipt] = &[];

            // Note: we assume this transaction is valid, because it's mined (or part of pending block) and
            // we don't need to check for pre EIP-2
            let from = transaction
                .recover_signer_unchecked()
                .ok_or(EthApiError::InvalidTransactionSignature)?;
            // get the previous transaction cumulative gas used
            let gas_used = if meta.index == 0 {
                receipt.cumulative_gas_used
            } else {
                let prev_tx_idx = (meta.index - 1) as usize;
                all_receipts
                    .get(prev_tx_idx)
                    .map(|prev_receipt| {
                        receipt.cumulative_gas_used - prev_receipt.cumulative_gas_used
                    })
                    .unwrap_or_default()
            };
            let blob_gas_used = transaction.transaction.blob_gas_used();
            // Blob gas price should only be present if the transaction is a blob transaction
            let blob_gas_price = blob_gas_used.and_then(|_| {
                meta.excess_blob_gas
                    .map(revm::primitives::calc_blob_gasprice)
            });
            let logs_bloom = receipt.bloom_slow();

            // get number of logs in the block
            let mut num_logs = 0;
            for prev_receipt in all_receipts.iter().take(meta.index as usize) {
                num_logs += prev_receipt.logs.len();
            }

            let mut logs = Vec::with_capacity(receipt.logs.len());
            for (tx_log_idx, log) in receipt.logs.into_iter().enumerate() {
                let rpclog = alloy_rpc_types::Log {
                    inner: log,
                    block_hash: Some(meta.block_hash),
                    block_number: Some(meta.block_number),
                    block_timestamp: Some(meta.timestamp),
                    transaction_hash: Some(meta.tx_hash),
                    transaction_index: Some(meta.index),
                    log_index: Some((num_logs + tx_log_idx) as u64),
                    removed: false,
                };
                logs.push(rpclog);
            }

            let rpc_receipt = reth_rpc_types::Receipt {
                status: receipt.success,
                cumulative_gas_used: receipt.cumulative_gas_used as u128,
                logs,
            };

            #[allow(clippy::needless_update)]
            let res_receipt = alloy_rpc_types::TransactionReceipt {
                inner: alloy_rpc_types::AnyReceiptEnvelope {
                    inner: alloy_rpc_types::ReceiptWithBloom {
                        receipt: rpc_receipt,
                        logs_bloom,
                    },
                    r#type: transaction.transaction.tx_type().into(),
                },
                transaction_hash: meta.tx_hash,
                transaction_index: Some(meta.index),
                block_hash: Some(meta.block_hash),
                block_number: Some(meta.block_number),
                from,
                to: None,
                gas_used: gas_used as u128,
                contract_address: None,
                effective_gas_price: transaction.effective_gas_price(meta.base_fee),
                // TODO pre-byzantium receipts have a post-transaction state root
                state_root: None,
                // EIP-4844 fields
                blob_gas_price,
                blob_gas_used: blob_gas_used.map(u128::from),
            };
            let mut res_receipt = alloy_rpc_types::WithOtherFields::new(res_receipt);

            match transaction.transaction.kind() {
                TransactionKind::Create => {
                    res_receipt.contract_address =
                        Some(from.create(transaction.transaction.nonce()));
                }
                TransactionKind::Call(addr) => {
                    res_receipt.to = Some(*addr);
                }
            }

            res_receipt.transaction_hash = hash;
            Ok(res_receipt)
        }

        if let Some((tx, meta, receipt)) = from_db(r.blockchain(), hash)? {
            Some(make_any_transaction_receipt(hash, tx, meta, receipt))
                .transpose()
                .map_err(Into::into)
        } else if let Some((tx, receipt)) = from_builder(&r.payloads, hash) {
            Some(make_any_transaction_receipt(
                hash,
                tx,
                Default::default(),
                receipt,
            ))
            .transpose()
            .map_err(Into::into)
        } else {
            Ok(None)
        }
    }
}

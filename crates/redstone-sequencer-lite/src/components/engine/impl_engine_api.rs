use std::sync::Arc;

use alloy_rpc_types_engine::{
    ForkchoiceState, ForkchoiceUpdated, OptimismExecutionPayloadEnvelopeV3,
    OptimismPayloadAttributes, PayloadStatus, PayloadStatusEnum,
};
use jsonrpsee::core::RpcResult;
use reth_interfaces::{
    blockchain_tree::{BlockValidationKind, CanonicalOutcome},
    provider::ProviderError,
};
use reth_node_api::{ConfigureEvm, ConfigureEvmEnv, PayloadBuilderAttributes};
use reth_optimism_payload_builder::OptimismPayloadBuilderAttributes;
use reth_payload_builder::PayloadId;
use reth_primitives::{TransactionSigned, B256, U256};
use reth_provider::BlockSource;
use reth_rpc::eth::error::EthApiError;
use reth_rpc_types::{ExecutionPayloadV1, ExecutionPayloadV2, ExecutionPayloadV3};
use tracing::{debug, info};

use crate::{api::EngineApiV3Server, components::engine::RedstoneBuiltPayload};

use super::{payload_builder::RedstonePayloadBuilder, Blockchain, Engine};

type PayloadAttrs = OptimismPayloadAttributes;

#[async_trait::async_trait]
impl<B, V> EngineApiV3Server for Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    async fn forkchoice_updated(
        &self,
        state: ForkchoiceState,
        attrs: Option<PayloadAttrs>,
    ) -> RpcResult<ForkchoiceUpdated> {
        self.process_forkchoice_updated(state, attrs)
            .await
            .map_err(Into::into)
    }

    async fn new_payload(
        &self,
        payload: ExecutionPayloadV3,
        versioned_hashes: Vec<B256>,
        parent_beacon_block_root: B256,
    ) -> RpcResult<PayloadStatus> {
        let block = reth_rpc_types_compat::engine::try_into_sealed_block(
            payload.into(),
            Some(parent_beacon_block_root),
        )
        .map_err(|e| e.to_string())
        .map_err(EthApiError::InvalidParams)?;

        let this_hash = *block.hash();
        let parent_hash = block.parent_hash;
        let state_root = block.state_root;

        info!(?this_hash, ?parent_hash, ?state_root, "new-payload");

        let r = self.0.read().await;

        let block_hash = block.hash();
        let insert_payload_ok = r
            .blockchain()
            .insert_block_without_senders(block, BlockValidationKind::Exhaustive)
            .map_err(|e| e.to_string())
            .map_err(EthApiError::InvalidParams)?;

        info!(
            ?this_hash,
            ?parent_hash,
            ?state_root,
            ?insert_payload_ok,
            "new-payload [INSERT-OK]"
        );

        Ok(PayloadStatus::new(
            alloy_rpc_types_engine::PayloadStatusEnum::Valid,
            Some(block_hash),
        ))
    }

    async fn get_payload(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<OptimismExecutionPayloadEnvelopeV3> {
        self.process_get_payload(payload_id)
            .await
            .inspect(|envelope| {
                let parent_hash = envelope
                    .execution_payload
                    .payload_inner
                    .payload_inner
                    .parent_hash;
                let block_hash = envelope
                    .execution_payload
                    .payload_inner
                    .payload_inner
                    .block_hash;
                let state_root = envelope
                .execution_payload
                .payload_inner
                .payload_inner
                .state_root;
                info!(this = ?block_hash, parent = ?parent_hash, state_root = ?state_root, "get-payload.OK: ");
            })
            .map_err(Into::into)
    }
}

impl<B, V> Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
    async fn process_get_payload(
        &self,
        payload_id: PayloadId,
    ) -> Result<OptimismExecutionPayloadEnvelopeV3, EthApiError> {
        let Some(builder) = self.0.write().await.payloads.remove(&payload_id) else {
            return Err(EthApiError::InvalidParams(format!(
                "Unknown payload-id: {}",
                payload_id
            )));
        };
        let RedstoneBuiltPayload {
            block,
            fees,
            sidecars,
            chain_spec,
            attributes,
            ..
        } = builder
            .into_payload()
            .map_err(|e| e.to_string())
            .map_err(EthApiError::InvalidParams)?;

        let parent_beacon_block_root = chain_spec
            .is_cancun_active_at_timestamp(attributes.timestamp())
            .then(|| attributes.parent_beacon_block_root())
            .flatten()
            .unwrap_or(B256::ZERO);

        let transactions = block.raw_transactions();
        let withdrawals: Vec<_> = block
            .withdrawals
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|w| reth_rpc_types::Withdrawal {
                index: w.index,
                validator_index: w.validator_index,
                address: w.address,
                amount: w.amount,
            })
            .collect();

        let payload_inner = ExecutionPayloadV1 {
            parent_hash: block.parent_hash,
            fee_recipient: block.beneficiary,
            state_root: block.state_root,
            receipts_root: block.receipts_root,
            logs_bloom: block.logs_bloom,
            prev_randao: block.mix_hash,
            block_number: block.number,
            gas_limit: block.gas_limit,
            gas_used: block.gas_used,
            timestamp: block.timestamp,
            extra_data: block.extra_data.clone(),
            base_fee_per_gas: U256::from(block.base_fee_per_gas.unwrap_or_default()),
            block_hash: block.hash(),
            transactions,
        };
        let payload_inner = ExecutionPayloadV2 {
            payload_inner,
            withdrawals,
        };
        let execution_payload = ExecutionPayloadV3 {
            payload_inner,
            blob_gas_used: block.blob_gas_used(),
            excess_blob_gas: block.excess_blob_gas.unwrap_or_default(),
        };

        let envelope = OptimismExecutionPayloadEnvelopeV3 {
            execution_payload,
            block_value: fees,
            should_override_builder: false,
            blobs_bundle: sidecars
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            parent_beacon_block_root,
        };

        Ok(envelope)
    }

    async fn process_forkchoice_updated(
        &self,
        state: ForkchoiceState,
        attrs: Option<PayloadAttrs>,
    ) -> Result<ForkchoiceUpdated, EthApiError> {
        if state.head_block_hash.is_zero() {
            return Ok(forkchoice::new_updated(
                forkchoice::state_invalid("head-block-hash is zero"),
                None,
                None,
            ));
        }
        {
            let r = self.0.read().await;

            let make_canonical_outcome = match r.blockchain().make_canonical(state.head_block_hash)
            {
                Ok(outcome) => outcome,
                Err(_err) => return Err(EthApiError::Unsupported("make_canonical failed")),
            };

            match make_canonical_outcome {
                CanonicalOutcome::Committed { head } => {
                    debug!("make_canonical {} -> Committed", head.hash());
                    r.blockchain().set_canonical_head(head);
                }
                CanonicalOutcome::AlreadyCanonical { header } => {
                    debug!("make_canonical {} -> AlreadyCanonical", header.hash());
                    r.blockchain().set_canonical_head(header);
                }
            }

            let finalized = r
                .blockchain()
                .find_block_by_hash(state.finalized_block_hash, BlockSource::Any)?
                .ok_or_else(|| ProviderError::UnknownBlockHash(state.finalized_block_hash))?;
            r.blockchain().finalize_block(finalized.number);
            r.blockchain()
                .set_finalized(finalized.header.seal(state.finalized_block_hash));

            let safe = r
                .blockchain()
                .find_block_by_hash(state.safe_block_hash, BlockSource::Any)?
                .ok_or_else(|| ProviderError::UnknownBlockHash(state.finalized_block_hash))?;
            r.blockchain()
                .set_safe(safe.header.seal(state.safe_block_hash));
        };
        let payload_id = if let Some(attrs) = attrs {
            let payload_id = self.process_payload_attributes(state, attrs).await?;
            Some(payload_id)
        } else {
            None
        };

        Ok(forkchoice::new_updated(
            PayloadStatusEnum::Valid,
            Some(state.head_block_hash),
            payload_id,
        ))
    }

    async fn process_payload_attributes(
        &self,
        state: ForkchoiceState,
        attrs: PayloadAttrs,
    ) -> Result<PayloadId, EthApiError> {
        let mut txs = vec![];
        for tx_encoded in attrs.transactions.as_ref().into_iter().flatten() {
            let tx = TransactionSigned::decode_enveloped(&mut &tx_encoded[..])
                .map_err(|e| e.to_string())
                .map_err(EthApiError::InvalidParams)?;
            txs.push(tx);
        }
        let (payload_id, builder) = {
            let r = self.0.read().await;

            let parent_block = r
                .blockchain()
                .find_block_by_hash(state.head_block_hash, BlockSource::Any)?
                .ok_or_else(|| ProviderError::UnknownBlockHash(state.head_block_hash))?
                .seal(state.head_block_hash);

            let builder_attributes =
                <OptimismPayloadBuilderAttributes as PayloadBuilderAttributes>::try_new(
                    state.head_block_hash,
                    attrs,
                )
                .map_err(|e| e.to_string())
                .map_err(EthApiError::InvalidParams)?;

            let payload_id = builder_attributes.payload_id();

            let mut builder =
                RedstonePayloadBuilder::<OptimismPayloadBuilderAttributes, B, V>::init(
                    builder_attributes,
                    r.blockchain().clone(),
                    Arc::clone(&r.args.chain_spec),
                    Arc::new(parent_block),
                    r.args.evm_config.clone(),
                    r.args.payload_extradata.clone(),
                )
                .map_err(|e| e.to_string())
                .map_err(EthApiError::InvalidParams)?;

            for tx in txs {
                builder
                    .process_transaction(tx, /* skip_fees: */ true)
                    .map_err(|e| e.to_string())
                    .map_err(EthApiError::InvalidParams)?;
            }

            (payload_id, builder)
        };
        {
            let mut w = self.0.write().await;
            w.payloads.insert(payload_id, builder);
        }

        Ok(payload_id)
    }
}

mod forkchoice {
    use super::*;

    pub(crate) fn state_invalid(validation_error: impl Into<String>) -> PayloadStatusEnum {
        let validation_error = validation_error.into();
        PayloadStatusEnum::Invalid { validation_error }
    }

    pub(crate) fn new_updated(
        status: PayloadStatusEnum,
        latest_valid_hash: Option<B256>,
        payload_id: Option<PayloadId>,
    ) -> ForkchoiceUpdated {
        ForkchoiceUpdated {
            payload_status: PayloadStatus {
                status,
                latest_valid_hash,
            },
            payload_id,
        }
    }

    // pub(crate) fn payload_id_optimism(
    //     parent: &B256,
    //     attributes: &OptimismPayloadAttributes,
    //     txs: &[TransactionSigned],
    // ) -> PayloadId {
    //     use alloy_rlp::Encodable;
    //     use sha2::Digest;

    //     let mut hasher = sha2::Sha256::new();
    //     hasher.update(parent.as_slice());
    //     hasher.update(&attributes.payload_attributes.timestamp.to_be_bytes()[..]);
    //     hasher.update(attributes.payload_attributes.prev_randao.as_slice());
    //     hasher.update(
    //         attributes
    //             .payload_attributes
    //             .suggested_fee_recipient
    //             .as_slice(),
    //     );
    //     if let Some(withdrawals) = &attributes.payload_attributes.withdrawals {
    //         let mut buf = Vec::new();
    //         withdrawals.encode(&mut buf);
    //         hasher.update(buf);
    //     }

    //     if let Some(parent_beacon_block) = attributes.payload_attributes.parent_beacon_block_root {
    //         hasher.update(parent_beacon_block);
    //     }

    //     let no_tx_pool = attributes.no_tx_pool.unwrap_or_default();
    //     if no_tx_pool || !txs.is_empty() {
    //         hasher.update([no_tx_pool as u8]);
    //         hasher.update(txs.len().to_be_bytes());
    //         txs.iter().for_each(|tx| hasher.update(tx.hash()));
    //     }

    //     if let Some(gas_limit) = attributes.gas_limit {
    //         hasher.update(gas_limit.to_be_bytes());
    //     }

    //     let out = hasher.finalize();
    //     PayloadId::new(out.as_slice()[..8].try_into().expect("sufficient length"))
    // }
}

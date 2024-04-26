use std::sync::Arc;

use alloy_rpc_types_engine::{
    ForkchoiceState, ForkchoiceUpdated, OptimismExecutionPayloadEnvelopeV3,
    OptimismPayloadAttributes, PayloadStatus, PayloadStatusEnum,
};
use futures::TryFutureExt;
use jsonrpsee::core::RpcResult;
use reth_interfaces::{blockchain_tree::CanonicalOutcome, provider::ProviderError};
use reth_node_api::{ConfigureEvm, ConfigureEvmEnv, PayloadAttributes, PayloadBuilderAttributes};
use reth_optimism_payload_builder::OptimismPayloadBuilderAttributes;
use reth_payload_builder::PayloadId;
use reth_primitives::{TransactionSigned, B256};
use reth_provider::BlockSource;
use reth_rpc::eth::error::{EthApiError, RpcInvalidTransactionError};
use reth_rpc_types::ExecutionPayloadV3;
use tracing::debug;

use crate::{api::EngineApiV3Server, components::evm::RedstoneEvmConfig};

use super::{preview::Preview, Blockchain, Engine};

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
        Ok(PayloadStatus::new(
            alloy_rpc_types_engine::PayloadStatusEnum::Valid,
            Default::default(),
        ))
    }

    async fn get_payload(
        &self,
        payload_id: PayloadId,
    ) -> RpcResult<OptimismExecutionPayloadEnvelopeV3> {
        unimplemented!()
    }
}

impl<B, V> Engine<B, V>
where
    Engine<B, V>: 'static,
    B: Blockchain,
    V: ConfigureEvm + ConfigureEvmEnv,
{
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
                Err(err) => return Err(EthApiError::Unsupported("make_canonical failed")),
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

            let builder = Preview::<OptimismPayloadBuilderAttributes, B, V>::init(
                builder_attributes,
                r.blockchain().clone(),
                Arc::clone(&r.args.chain_spec),
                Arc::new(parent_block),
                r.args.evm_config.clone(),
                r.args.payload_extradata.clone(),
            )
            .map_err(|e| e.to_string())
            .map_err(EthApiError::InvalidParams)?;

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

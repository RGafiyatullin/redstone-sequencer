use std::sync::Arc;

use reth_basic_payload_builder::WithdrawalsOutcome;
use reth_node_api::{ConfigureEvm, PayloadBuilderAttributes};
use reth_optimism_payload_builder::error::OptimismPayloadBuilderError;
use reth_payload_builder::error::PayloadBuilderError;
use reth_primitives::{
    constants::BEACON_NONCE, proofs, Block, Bytes, ChainSpec, Hardfork, Header, Receipt, Receipts,
    SealedBlock, TransactionSigned, TxType, EMPTY_OMMER_ROOT_HASH, U256,
};
use reth_provider::{BundleStateWithReceipts, StateProvider};
use reth_revm::database::StateProviderDatabase;
use revm::{
    db::{states::bundle_state::BundleRetention, BundleState, State},
    primitives::{BlockEnv, CfgEnvWithHandlerCfg, EVMError, EnvWithHandlerCfg, ResultAndState},
    DatabaseCommit,
};
use tracing::{trace, warn};

use super::{Blockchain, RedstoneBuiltPayload};

pub(crate) struct RedstonePayloadBuilder<A, B, E> {
    attributes: A,
    blockchain: B,
    chain_spec: Arc<ChainSpec>,
    cfg: CfgEnvWithHandlerCfg,
    block_env: BlockEnv,
    state_provider: Box<dyn StateProvider>,
    bundle_state: BundleState,
    evm_config: E,
    cumulative_gas_used: u64,
    total_fees: U256,
    parent_block: Arc<SealedBlock>,
    transactions: Vec<TransactionSigned>,
    receipts: Vec<Receipt>,
    extra_data: Bytes,
}

impl<A, B, E> RedstonePayloadBuilder<A, B, E>
where
    A: PayloadBuilderAttributes,
    B: Blockchain,
    E: ConfigureEvm,
{
    pub(crate) fn init(
        attributes: A,
        blockchain: B,
        chain_spec: Arc<ChainSpec>,
        parent_block: Arc<SealedBlock>,
        evm_config: E,
        extra_data: Bytes,
    ) -> Result<Self, PayloadBuilderError> {
        let state_provider = blockchain
            .state_by_block_hash(parent_block.hash())
            .inspect_err(|err| {
                warn!(
                    target: "payload_builder",
                    parent_hash=%parent_block.hash(),
                    %err,
                    "failed to get state for empty payload"
                )
            })?;
        let state = StateProviderDatabase::new(&state_provider);

        let (cfg, block_env) =
            attributes.cfg_and_block_env(chain_spec.as_ref(), parent_block.header());

        let mut db = State::builder()
            .with_database(state)
            .with_bundle_update()
            .build();

        let block_number: u64 = block_env.number.to();
        // let base_fee: u64 = block_env.basefee.to();
        // let block_gas_limit: u64 = block_env.gas_limit.try_into().unwrap_or(u64::MAX);
        // let total_fees = U256::ZERO;
        // let cumulative_gas_used = 0;

        // let is_regolith =
        //     chain_spec.is_fork_active_at_timestamp(Hardfork::Regolith, attributes.timestamp());

        reth_basic_payload_builder::pre_block_beacon_root_contract_call(
            &mut db,
            chain_spec.as_ref(),
            block_number,
            &cfg,
            &block_env,
            &attributes,
        )
        .inspect_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash=%parent_block.hash(),
                %err,
                "failed to apply beacon root contract call for empty payload"
            )
        })?;

        reth_revm::optimism::ensure_create2_deployer(
            Arc::clone(&chain_spec),
            attributes.timestamp(),
            &mut db,
        )
        .map_err(|_| OptimismPayloadBuilderError::ForceCreate2DeployerFail)
        .map_err(PayloadBuilderError::other)
        .inspect_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash=%parent_block.hash(),
                %err,
                "failed to force-deploy create2deployer"
            )
        })?;

        db.merge_transitions(BundleRetention::PlainState);
        let bundle_state = db.take_bundle();

        let out = Self {
            attributes,
            blockchain,
            chain_spec,
            cfg,
            block_env,
            state_provider,
            bundle_state,
            evm_config,
            cumulative_gas_used: 0,
            total_fees: U256::ZERO,
            parent_block,
            transactions: Default::default(),
            receipts: Default::default(),
            extra_data,
        };

        Ok(out)
    }

    pub(crate) fn process_transaction(
        &mut self,
        tx: TransactionSigned,
        skip_fees: bool,
    ) -> Result<Option<&Receipt>, PayloadBuilderError> {
        if matches!(tx.tx_type(), TxType::Eip4844) {
            return Err(OptimismPayloadBuilderError::BlobTransactionRejected)
                .map_err(PayloadBuilderError::other);
        }

        let tx_secr = tx
            .clone()
            .try_into_ecrecovered()
            .map_err(|_| OptimismPayloadBuilderError::TransactionEcRecoverFailed)
            .map_err(PayloadBuilderError::other)?;

        let is_regolith = self
            .chain_spec
            .is_fork_active_at_timestamp(Hardfork::Regolith, self.attributes.timestamp());

        let mut db = State::builder()
            .with_database(StateProviderDatabase::new(&self.state_provider))
            .with_bundle_update()
            .with_bundle_prestate(self.bundle_state.clone())
            .build();

        let depositor = (is_regolith && tx_secr.is_deposit())
            .then(|| {
                db.load_cache_account(tx_secr.signer())
                    .map(|acc| acc.account_info().unwrap_or_default())
            })
            .transpose()
            .map_err(|_| OptimismPayloadBuilderError::AccountLoadFailed(tx_secr.signer()))
            .map_err(PayloadBuilderError::other)?;

        let env = EnvWithHandlerCfg::new_with_cfg_env(
            self.cfg.clone(),
            self.block_env.clone(),
            reth_primitives::revm::env::tx_env_with_recovered(&tx_secr),
        );

        let mut evm = self.evm_config.evm_with_env(&mut db, env);

        let Some(ResultAndState { result, state }) = evm
            .transact()
            .map(Some)
            .or_else(|err| match err {
                EVMError::Transaction(err) => {
                    trace!(target: "payload_builder", %err, ?tx, "Error in transaction, skipping");
                    Ok(None)
                }
                other => Err(other),
            })
            .map_err(PayloadBuilderError::EvmExecutionError)?
        else {
            return Ok(None);
        };

        std::mem::drop(evm);
        db.commit(state);

        let gas_used = result.gas_used();
        self.cumulative_gas_used += gas_used;

        let receipt = Receipt {
            tx_type: tx_secr.tx_type(),
            success: result.is_success(),
            cumulative_gas_used: self.cumulative_gas_used,
            logs: result.into_logs().into_iter().map(Into::into).collect(),
            deposit_nonce: depositor.map(|account| account.nonce),
            deposit_receipt_version: self
                .chain_spec
                .is_fork_active_at_timestamp(Hardfork::Canyon, self.attributes.timestamp())
                .then_some(1),
        };

        if !skip_fees {
            let miner_fee = tx_secr
                .effective_tip_per_gas(Some(self.block_env.basefee.to()))
                .expect("fee is always valid; execution succeeded");
            self.total_fees += U256::from(miner_fee) * U256::from(gas_used);
        }

        self.transactions.push(tx);
        self.receipts.push(receipt);

        db.merge_transitions(BundleRetention::PlainState);
        self.bundle_state = db.take_bundle();

        Ok(self.receipts.last())
    }

    pub(crate) fn into_payload(self) -> Result<RedstoneBuiltPayload<A>, PayloadBuilderError> {
        let Self {
            state_provider,
            attributes,
            chain_spec,
            transactions,
            receipts,
            parent_block,
            block_env,
            extra_data,
            cumulative_gas_used,
            bundle_state,
            ..
        } = self;

        let fees = self.total_fees;
        let sidecars = Default::default();
        let ommers = Default::default();
        let block_gas_limit: u64 = block_env.gas_limit.try_into().unwrap_or(u64::MAX);
        let base_fee: u64 = block_env.basefee.to();

        let mut db = State::builder()
            .with_database(StateProviderDatabase::new(&state_provider))
            .with_bundle_update()
            .with_bundle_prestate(bundle_state)
            .build();

        let WithdrawalsOutcome {
            withdrawals_root,
            withdrawals,
        } = reth_basic_payload_builder::commit_withdrawals(
            &mut db,
            chain_spec.as_ref(),
            attributes.timestamp(),
            attributes.withdrawals().clone(),
        )
        .inspect_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash=%parent_block.hash(),
                %err,
                "failed to commit withdrawals for empty payload"
            )
        })?;

        db.merge_transitions(BundleRetention::PlainState);
        let bundle_state = db.take_bundle();

        let bundle = BundleStateWithReceipts::new(
            bundle_state,
            Receipts::from_vec(vec![receipts.into_iter().map(Some).collect()]),
            block_env.number.to(),
        );
        let receipts_root = bundle
            .optimism_receipts_root_slow(
                block_env.number.to(),
                chain_spec.as_ref(),
                attributes.timestamp(),
            )
            .expect("Number is in range");
        let logs_bloom = bundle
            .block_logs_bloom(block_env.number.to())
            .expect("Number is in range");
        let state_root = state_provider.state_root(bundle.state())?;
        let transactions_root = proofs::calculate_transaction_root(&transactions);

        let is_cancun_active_now = chain_spec.is_cancun_active_at_timestamp(attributes.timestamp());
        let is_cancun_active_at_parent =
            chain_spec.is_cancun_active_at_timestamp(parent_block.timestamp);

        let (excess_blob_gas, blob_gas_used) =
            match (is_cancun_active_now, is_cancun_active_at_parent) {
                (true, true) => Some((
                    reth_primitives::eip4844::calculate_excess_blob_gas(
                        parent_block.excess_blob_gas.unwrap_or_default(),
                        parent_block.blob_gas_used.unwrap_or_default(),
                    ),
                    0,
                )),
                (true, false) => Some((0, 0)),
                (false, _probably_false_too) => None,
            }
            .unzip();

        let header = Header {
            parent_hash: parent_block.hash(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: block_env.coinbase,
            state_root,
            transactions_root,
            receipts_root,
            withdrawals_root,
            logs_bloom,
            timestamp: attributes.timestamp(),
            mix_hash: attributes.prev_randao(),
            nonce: BEACON_NONCE,
            base_fee_per_gas: Some(base_fee),
            number: parent_block.number + 1,
            gas_limit: block_gas_limit,
            difficulty: U256::ZERO,
            extra_data,
            parent_beacon_block_root: attributes.parent_beacon_block_root(),
            blob_gas_used,
            excess_blob_gas,
            gas_used: cumulative_gas_used,
        };

        let block = Block {
            header,
            body: transactions,
            ommers,
            withdrawals,
        }
        .seal_slow();

        let payload_id = attributes.payload_id();

        let payload = RedstoneBuiltPayload {
            id: payload_id,
            block,
            fees,
            sidecars,
            chain_spec,
            attributes,
        };
        Ok(payload)
    }
}

impl<A, B, E> std::fmt::Debug for RedstonePayloadBuilder<A, B, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Preview").finish()
    }
}

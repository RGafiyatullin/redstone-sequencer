use std::sync::Arc;

use reth_basic_payload_builder::WithdrawalsOutcome;
use reth_node_api::PayloadBuilderAttributes;
use reth_optimism_payload_builder::error::OptimismPayloadBuilderError;
use reth_primitives::constants::BEACON_NONCE;
use reth_primitives::constants::EMPTY_RECEIPTS;
use reth_primitives::constants::EMPTY_TRANSACTIONS;
use reth_primitives::Block;
use reth_primitives::Hardfork;
use reth_primitives::Receipt;
use reth_primitives::TransactionSigned;
use reth_primitives::TxType;
use reth_primitives::EMPTY_OMMER_ROOT_HASH;
use reth_primitives::U256;
use reth_revm::database::StateProviderDatabase;
use revm::db::states::bundle_state::BundleRetention;
use revm::db::State;
use revm::primitives::EVMError;
use revm::primitives::EnvWithHandlerCfg;
use revm::primitives::ResultAndState;
use revm::DatabaseCommit;
use tracing::trace;

use reth_basic_payload_builder::BuildArguments;
use reth_basic_payload_builder::BuildOutcome;
use reth_basic_payload_builder::PayloadBuilder;
use reth_basic_payload_builder::PayloadConfig;
use reth_node_api::ConfigureEvm;
use reth_payload_builder::error::PayloadBuilderError;
use reth_primitives::ChainSpec;
use reth_provider::StateProviderFactory;
use reth_transaction_pool::TransactionPool;
use tracing::warn;

use super::RedstoneBuiltPayload;
use super::RedstonePayloadBuilderAttributes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedstonePayloadBuilder<EvmConfig> {
    chain_spec: Arc<ChainSpec>,
    evm_config: EvmConfig,
}

impl<EvmConfig> RedstonePayloadBuilder<EvmConfig> {
    /// OptimismPayloadBuilder constructor.
    pub fn new(chain_spec: Arc<ChainSpec>, evm_config: EvmConfig) -> Self {
        Self {
            chain_spec,
            evm_config,
        }
    }
}

impl<Pool, Client, EvmConfig> PayloadBuilder<Pool, Client> for RedstonePayloadBuilder<EvmConfig>
where
    Client: StateProviderFactory,
    Pool: TransactionPool,
    EvmConfig: ConfigureEvm,
{
    type Attributes = RedstonePayloadBuilderAttributes;
    type BuiltPayload = RedstoneBuiltPayload;

    fn try_build(
        &self,
        args: BuildArguments<Pool, Client, Self::Attributes, Self::BuiltPayload>,
    ) -> Result<BuildOutcome<Self::BuiltPayload>, PayloadBuilderError> {
        // FIXME
        let BuildArguments {
            client,
            pool,
            cached_reads,
            config,
            cancel,
            best_payload,
        } = args;

        build_payload(&client, config, &self.evm_config, []).map(move |payload| {
            BuildOutcome::Better {
                payload,
                cached_reads,
            }
        })
    }

    fn on_missing_payload(
        &self,
        args: BuildArguments<Pool, Client, Self::Attributes, Self::BuiltPayload>,
    ) -> Option<Self::BuiltPayload> {
        if args.config.attributes.no_tx_pool {
            if let Ok(BuildOutcome::Better { payload, .. }) = self.try_build(args) {
                trace!(target: "payload_builder", "[OPTIMISM] Forced best payload");
                return Some(payload);
            }
        }

        None
    }

    fn build_empty_payload(
        client: &Client,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<Self::BuiltPayload, PayloadBuilderError> {
        unimplemented!()
    }
}

pub fn build_payload<Client, EvmConfig>(
    client: &Client,
    config: PayloadConfig<RedstonePayloadBuilderAttributes>,
    evm_config: &EvmConfig,
    transactions: impl IntoIterator<Item = TransactionSigned>,
) -> Result<RedstoneBuiltPayload, PayloadBuilderError>
where
    Client: StateProviderFactory,
    EvmConfig: ConfigureEvm,
{
    let PayloadConfig {
        initialized_block_env,
        parent_block,
        attributes,
        chain_spec,
        initialized_cfg,
        extra_data,
    } = config;

    let state = client.state_by_block_hash(parent_block.hash())
        .inspect_err(|err| warn!(target: "payload_builder", parent_hash=%parent_block.hash(), %err, "failed to get state for empty payload"))?;

    let mut db = State::builder()
        .with_database(StateProviderDatabase::new(&state))
        .with_bundle_update()
        .build();

    let base_fee: u64 = initialized_block_env.basefee.to();
    let block_number: u64 = initialized_block_env.number.to();
    let block_gas_limit: u64 = initialized_block_env
        .gas_limit
        .try_into()
        .unwrap_or(u64::MAX);

    let mut total_fees = U256::ZERO;
    let mut cumulative_gas_used = 0;
    let mut executed_txs = vec![];
    let mut receipts = vec![];

    let is_regolith =
        chain_spec.is_fork_active_at_timestamp(Hardfork::Regolith, attributes.timestamp());

    reth_basic_payload_builder::pre_block_beacon_root_contract_call(
        &mut db,
        chain_spec.as_ref(),
        block_number,
        &initialized_cfg,
        &initialized_block_env,
        &attributes
    ).inspect_err(|err| warn!(target: "payload_builder", parent_hash=%parent_block.hash(), %err, "failed to apply beacon root contract call for empty payload"))?;

    for tx in attributes.transactions.iter().cloned().chain(transactions) {
        if matches!(tx.tx_type(), TxType::Eip4844) {
            return Err(OptimismPayloadBuilderError::BlobTransactionRejected)
                .map_err(PayloadBuilderError::other);
        }

        let tx = tx
            .try_into_ecrecovered()
            .map_err(|_| OptimismPayloadBuilderError::TransactionEcRecoverFailed)
            .map_err(PayloadBuilderError::other)?;

        let depositor = (is_regolith && tx.is_deposit())
            .then(|| {
                db.load_cache_account(tx.signer())
                    .map(|acc| acc.account_info().unwrap_or_default())
            })
            .transpose()
            .map_err(|_| OptimismPayloadBuilderError::AccountLoadFailed(tx.signer()))
            .map_err(PayloadBuilderError::other)?;

        let env = EnvWithHandlerCfg::new_with_cfg_env(
            initialized_cfg.clone(),
            initialized_block_env.clone(),
            reth_primitives::revm::env::tx_env_with_recovered(&tx),
        );

        let mut evm = evm_config.evm_with_env(&mut db, env);

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
            continue;
        };

        std::mem::drop(evm);

        db.commit(state);

        cumulative_gas_used += result.gas_used();

        receipts.push(Some(Receipt {
            tx_type: tx.tx_type(),
            success: result.is_success(),
            cumulative_gas_used,
            logs: result.into_logs().into_iter().map(Into::into).collect(),
            deposit_nonce: depositor.map(|account| account.nonce),
            deposit_receipt_version: chain_spec
                .is_fork_active_at_timestamp(Hardfork::Canyon, attributes.timestamp())
                .then_some(1),
        }));

        executed_txs.push(tx);
    }

    let WithdrawalsOutcome { withdrawals_root, withdrawals } = reth_basic_payload_builder::commit_withdrawals(
        &mut db, chain_spec.as_ref(),
        attributes.timestamp(),
        attributes.withdrawals().clone()
    ).inspect_err(|err| warn!(target: "payload_builder", parent_hash=%parent_block.hash(), %err, "failed to commit withdrawals for empty payload"))?;

    db.merge_transitions(BundleRetention::PlainState);

    // calculate the state root
    let bundle_state = db.take_bundle();
    let state_root = state.state_root(&bundle_state)
        .inspect_err(|err| warn!(target: "payload_builder", parent_hash=%parent_block.hash(), %err, "failed to calculate state root for empty payload"))?;

    let is_cancun_active_now = chain_spec.is_cancun_active_at_timestamp(attributes.timestamp());
    let is_cancun_active_at_parent =
        chain_spec.is_cancun_active_at_timestamp(parent_block.timestamp);

    let (excess_blob_gas, blob_gas_used) = match (is_cancun_active_now, is_cancun_active_at_parent)
    {
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

    let header = reth_primitives::Header {
        parent_hash: parent_block.hash(),
        ommers_hash: EMPTY_OMMER_ROOT_HASH,
        beneficiary: initialized_block_env.coinbase,
        state_root,
        transactions_root: EMPTY_TRANSACTIONS,
        withdrawals_root,
        receipts_root: EMPTY_RECEIPTS,
        logs_bloom: Default::default(),
        timestamp: attributes.timestamp(),
        mix_hash: attributes.prev_randao(),
        nonce: BEACON_NONCE,
        base_fee_per_gas: Some(base_fee),
        number: parent_block.number + 1,
        gas_limit: block_gas_limit,
        difficulty: U256::ZERO,
        gas_used: 0,
        extra_data,
        blob_gas_used,
        excess_blob_gas,
        parent_beacon_block_root: attributes.parent_beacon_block_root(),
    };

    let block = Block {
        header,
        body: executed_txs,
        ommers: Default::default(),
        withdrawals,
    }
    .seal_slow();

    Ok(RedstoneBuiltPayload {
        id: attributes.payload_id(),
        block,
        fees: U256::ZERO,
        sidecars: Default::default(),
        chain_spec,
        attributes,
    })
}

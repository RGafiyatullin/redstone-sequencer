use std::sync::Arc;

use reth_node_api::{ConfigureEvm, PayloadBuilderAttributes};
use reth_optimism_payload_builder::error::OptimismPayloadBuilderError;
use reth_payload_builder::error::PayloadBuilderError;
use reth_primitives::{
    Bytes, ChainSpec, Hardfork, Receipt, SealedBlock, TransactionSigned, TxType, U256,
};
use reth_provider::StateProvider;
use reth_revm::database::StateProviderDatabase;
use revm::{
    db::State,
    primitives::{BlockEnv, CfgEnvWithHandlerCfg, EnvWithHandlerCfg},
};
use tracing::warn;

use crate::AnyError;

use super::Blockchain;

pub struct Preview<A, B, E> {
    attributes: A,
    blockchain: B,
    chain_spec: Arc<ChainSpec>,
    cfg: CfgEnvWithHandlerCfg,
    block_env: BlockEnv,
    db: State<StateProviderDatabase<Box<dyn StateProvider>>>,
    evm_config: E,
}

impl<A, B, E> Preview<A, B, E>
where
    A: PayloadBuilderAttributes,
    B: Blockchain,
    E: ConfigureEvm,
{
    pub fn init(
        attributes: A,
        blockchain: B,
        chain_spec: Arc<ChainSpec>,
        parent_block: Arc<SealedBlock>,
        cfg: CfgEnvWithHandlerCfg,
        block_env: BlockEnv,
        evm_config: E,
        extra_data: Bytes,
    ) -> Result<Self, PayloadBuilderError> {
        let state = blockchain
            .state_by_block_hash(parent_block.hash())
            .inspect_err(|err| {
                warn!(
                    target: "payload_builder",
                    parent_hash=%parent_block.hash(),
                    %err,
                    "failed to get state for empty payload"
                )
            })?;
        let mut db = State::builder()
            .with_database(StateProviderDatabase::new(state))
            .with_bundle_update()
            .build();

        let base_fee: u64 = block_env.basefee.to();
        let block_number: u64 = block_env.number.to();
        let block_gas_limit: u64 = block_env.gas_limit.try_into().unwrap_or(u64::MAX);

        let mut total_fees = U256::ZERO;
        let mut cumulative_gas_used = 0;
        // let mut executed_txs = vec![];
        // let mut receipts = vec![];

        let is_regolith =
            chain_spec.is_fork_active_at_timestamp(Hardfork::Regolith, attributes.timestamp());

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

        let out = Self {
            attributes,
            blockchain,
            chain_spec,
            cfg,
            block_env,
            db,
            evm_config,
        };

        Ok(out)
    }

    pub fn process_transaction(
        &mut self,
        tx: TransactionSigned,
    ) -> Result<Option<Receipt>, PayloadBuilderError> {
        if matches!(tx.tx_type(), TxType::Eip4844) {
            return Err(OptimismPayloadBuilderError::BlobTransactionRejected)
                .map_err(PayloadBuilderError::other);
        }

        let tx = tx
            .try_into_ecrecovered()
            .map_err(|_| OptimismPayloadBuilderError::TransactionEcRecoverFailed)
            .map_err(PayloadBuilderError::other)?;

        let is_regolith = self
            .chain_spec
            .is_fork_active_at_timestamp(Hardfork::Regolith, self.attributes.timestamp());

        let depositor = (is_regolith && tx.is_deposit())
            .then(|| {
                self.db
                    .load_cache_account(tx.signer())
                    .map(|acc| acc.account_info().unwrap_or_default())
            })
            .transpose()
            .map_err(|_| OptimismPayloadBuilderError::AccountLoadFailed(tx.signer()))
            .map_err(PayloadBuilderError::other)?;

        let env = EnvWithHandlerCfg::new_with_cfg_env(
            self.cfg.clone(),
            self.block_env.clone(),
            reth_primitives::revm::env::tx_env_with_recovered(&tx),
        );

        let mut evm = self.evm_config.evm_with_env(&mut self.db, env);

        unimplemented!()
    }
}

impl<A, B, E> std::fmt::Debug for Preview<A, B, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Preview").finish()
    }
}

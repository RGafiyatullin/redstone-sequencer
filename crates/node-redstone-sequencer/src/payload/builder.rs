use std::sync::Arc;

use tracing::trace;

use reth_basic_payload_builder::BuildArguments;
use reth_basic_payload_builder::BuildOutcome;
use reth_basic_payload_builder::PayloadBuilder;
use reth_node_api::ConfigureEvm;
use reth_payload_builder::error::PayloadBuilderError;
use reth_primitives::ChainSpec;
use reth_provider::StateProviderFactory;
use reth_transaction_pool::TransactionPool;

use super::RedstoneBuiltPayload;
use super::RedstonePayloadBuilder;
use super::RedstonePayloadBuilderAttributes;

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
        unimplemented!()
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
        config: reth_basic_payload_builder::PayloadConfig<Self::Attributes>,
    ) -> Result<Self::BuiltPayload, PayloadBuilderError> {
        unimplemented!()
    }
}

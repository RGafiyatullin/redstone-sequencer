use reth_node_api::PayloadBuilderAttributes;
use reth_payload_builder::{EthPayloadBuilderAttributes, PayloadId};
use reth_primitives::{Address, ChainSpec, Header, TransactionSigned, Withdrawals, B256};
use revm::primitives::{BlockEnv, CfgEnvWithHandlerCfg};

use super::RedstonePayloadAttributes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedstonePayloadBuilderAttributes {
    pub payload_attributes: EthPayloadBuilderAttributes,
    pub no_tx_pool: bool,
    pub transactions: Vec<TransactionSigned>,
    pub gas_limit: Option<u64>,
}

impl PayloadBuilderAttributes for RedstonePayloadBuilderAttributes {
    type RpcPayloadAttributes = RedstonePayloadAttributes;
    type Error = alloy_rlp::Error;

    /// Creates a new payload builder for the given parent block and the attributes.
    ///
    /// Derives the unique [PayloadId] for the given parent and attributes
    fn try_new(_parent: B256, _attributes: RedstonePayloadAttributes) -> Result<Self, Self::Error> {
        // let (id, transactions) = {
        //     let transactions: Vec<_> = attributes
        //         .transactions
        //         .as_deref()
        //         .unwrap_or(&[])
        //         .iter()
        //         .map(|tx| TransactionSigned::decode_enveloped(&mut tx.as_ref()))
        //         .collect::<Result<_, _>>()?;
        //     (
        //         payload_id_optimism(&parent, &attributes, &transactions),
        //         transactions,
        //     )
        // };

        // let withdraw = attributes
        //     .payload_attributes
        //     .withdrawals
        //     .map(|withdrawals| {
        //         Withdrawals::new(
        //             withdrawals
        //                 .into_iter()
        //                 .map(convert_standalone_withdraw_to_withdrawal)
        //                 .collect(),
        //         )
        //     });

        // let payload_attributes = EthPayloadBuilderAttributes {
        //     id,
        //     parent,
        //     timestamp: attributes.payload_attributes.timestamp,
        //     suggested_fee_recipient: attributes.payload_attributes.suggested_fee_recipient,
        //     prev_randao: attributes.payload_attributes.prev_randao,
        //     withdrawals: withdraw.unwrap_or_default(),
        //     parent_beacon_block_root: attributes.payload_attributes.parent_beacon_block_root,
        // };

        // Ok(Self {
        //     payload_attributes,
        //     no_tx_pool: attributes.no_tx_pool.unwrap_or_default(),
        //     transactions,
        //     gas_limit: attributes.gas_limit,
        // })

        unimplemented!()
    }

    fn payload_id(&self) -> PayloadId {
        self.payload_attributes.id
    }

    fn parent(&self) -> B256 {
        self.payload_attributes.parent
    }

    fn timestamp(&self) -> u64 {
        self.payload_attributes.timestamp
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.payload_attributes.parent_beacon_block_root
    }

    fn suggested_fee_recipient(&self) -> Address {
        self.payload_attributes.suggested_fee_recipient
    }

    fn prev_randao(&self) -> B256 {
        self.payload_attributes.prev_randao
    }

    fn withdrawals(&self) -> &Withdrawals {
        &self.payload_attributes.withdrawals
    }

    fn cfg_and_block_env(
        &self,
        _chain_spec: &ChainSpec,
        _parent: &Header,
    ) -> (CfgEnvWithHandlerCfg, BlockEnv) {
        // // configure evm env based on parent block
        // let mut cfg = CfgEnv::default();
        // cfg.chain_id = chain_spec.chain().id();

        // // ensure we're not missing any timestamp based hardforks
        // let spec_id = revm_spec_by_timestamp_after_merge(chain_spec, self.timestamp());

        // // if the parent block did not have excess blob gas (i.e. it was pre-cancun), but it is
        // // cancun now, we need to set the excess blob gas to the default value
        // let blob_excess_gas_and_price = parent
        //     .next_block_excess_blob_gas()
        //     .or_else(|| {
        //         if spec_id.is_enabled_in(SpecId::CANCUN) {
        //             // default excess blob gas is zero
        //             Some(0)
        //         } else {
        //             None
        //         }
        //     })
        //     .map(BlobExcessGasAndPrice::new);

        // let block_env = BlockEnv {
        //     number: U256::from(parent.number + 1),
        //     coinbase: self.suggested_fee_recipient(),
        //     timestamp: U256::from(self.timestamp()),
        //     difficulty: U256::ZERO,
        //     prevrandao: Some(self.prev_randao()),
        //     gas_limit: U256::from(parent.gas_limit),
        //     // calculate basefee based on parent block's gas usage
        //     basefee: U256::from(
        //         parent
        //             .next_block_base_fee(chain_spec.base_fee_params(self.timestamp()))
        //             .unwrap_or_default(),
        //     ),
        //     // calculate excess gas based on parent block's blob gas usage
        //     blob_excess_gas_and_price,
        // };

        // let cfg_with_handler_cfg;
        // {
        //     cfg_with_handler_cfg = CfgEnvWithHandlerCfg {
        //         cfg_env: cfg,
        //         handler_cfg: HandlerCfg {
        //             spec_id,
        //             is_optimism: chain_spec.is_optimism(),
        //         },
        //     };
        // }

        // (cfg_with_handler_cfg, block_env)

        unimplemented!()
    }
}

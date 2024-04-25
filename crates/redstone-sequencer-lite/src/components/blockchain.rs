use std::sync::Arc;

use reth::{
    beacon_consensus::BeaconConsensus,
    blockchain_tree::{BlockchainTree, ShareableBlockchainTree, TreeExternals},
};
use reth_db::database::Database;
use reth_node_api::ConfigureEvm;
use reth_primitives::ChainSpec;
use reth_provider::{providers::BlockchainProvider, ProviderFactory};
use reth_revm::EvmProcessorFactory;

use crate::AnyError;

pub type Blockchain<Db, Evm> =
    BlockchainProvider<Db, ShareableBlockchainTree<Db, EvmProcessorFactory<Evm>>>;

pub fn open<Db, Evm>(
    chain_spec: Arc<ChainSpec>,
    provider_factory: ProviderFactory<Db>,
    evm_config: Evm,
) -> Result<Blockchain<Db, Evm>, AnyError>
where
    Db: Database + Clone,
    Evm: ConfigureEvm + 'static,
{
    let config = Default::default();
    let prune_modes = None;
    let consensus = Arc::new(BeaconConsensus::new(Arc::clone(&chain_spec)));
    let externals = TreeExternals::new(
        provider_factory.clone(),
        consensus,
        EvmProcessorFactory::new(chain_spec, evm_config),
    );
    let tree = BlockchainTree::new(externals, config, prune_modes)?;
    let tree = ShareableBlockchainTree::new(tree);
    let blockchain = BlockchainProvider::new(provider_factory, tree)?;
    Ok(blockchain)
}

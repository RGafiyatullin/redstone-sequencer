use std::{sync::Arc, time::Instant};

use reth::dirs::{DataDirPath, MaybePlatformPath};
use reth_primitives::ChainSpec;
use tracing::info;

use crate::components::blockchain;
use crate::components::db;
use crate::components::evm;
use crate::AnyError;

use super::Cli;

#[derive(Debug, structopt::StructOpt)]
pub(crate) struct CmdNode {
    #[structopt(long, env = "CHAIN_SPEC", parse(try_from_str = super::args::load_chain_spec))]
    chain_spec: Arc<ChainSpec>,

    #[structopt(long, env = "DB_PATH")]
    db_path: MaybePlatformPath<DataDirPath>,
}

impl CmdNode {
    pub(crate) async fn run(&self, _cli: &Cli) -> Result<(), AnyError> {
        let t0 = Instant::now();
        let db_path = self
            .db_path
            .unwrap_or_chain_default(self.chain_spec.chain());
        let provider_factory = db::open(db_path, Arc::clone(&self.chain_spec))?;
        db::ensure_genesis(&provider_factory, Arc::clone(&self.chain_spec))?;
        info!(elapsed = ?t0.elapsed(), "Database ready.");

        let t0 = Instant::now();
        let evm_config = evm::RedstoneEvmConfig::default();
        let _blockchain_provider =
            blockchain::open(Arc::clone(&self.chain_spec), provider_factory, evm_config);
        info!(elapsed = ?t0.elapsed(), "Blockchain provider ready.");

        Err("not implemented".into())
    }
}

use std::net::SocketAddr;
use std::{sync::Arc, time::Instant};

use reth::dirs::{DataDirPath, MaybePlatformPath};
use reth_primitives::ChainSpec;
use tracing::info;

use crate::components::blockchain;
use crate::components::db;
use crate::components::engine;
use crate::components::evm;
use crate::components::rpc;
use crate::AnyError;

use super::Cli;

#[derive(Debug, structopt::StructOpt)]
pub(crate) struct CmdNode {
    #[structopt(long, env = "RPC_BIND_ADDR_A")]
    rpc_bind_addr_a: SocketAddr,
    #[structopt(long, env = "RPC_BIND_ADDR_B")]
    rpc_bind_addr_b: SocketAddr,

    #[structopt(long, env = "CHAIN_SPEC", parse(try_from_str = super::args::load_chain_spec))]
    chain_spec: Arc<ChainSpec>,

    #[structopt(long, env = "DB_PATH")]
    db_path: MaybePlatformPath<DataDirPath>,
    // #[structopt(long, env = "TICK_INTERVAL")]
    // tick_interval: humantime::Duration,
}

impl CmdNode {
    pub(crate) async fn run(&self, _cli: &Cli) -> Result<(), AnyError> {
        let t0 = Instant::now();
        let db_path = self
            .db_path
            .unwrap_or_chain_default(self.chain_spec.chain());
        let provider_factory = db::open(db_path, Arc::clone(&self.chain_spec))?;
        let genesis_hash = db::ensure_genesis(&provider_factory, Arc::clone(&self.chain_spec))?;
        info!(elapsed = ?t0.elapsed(), genesis_hash = ?genesis_hash, "Database ready.");

        let t0 = Instant::now();
        let evm_config = evm::RedstoneEvmConfig::default();
        let blockchain_provider = blockchain::open(
            Arc::clone(&self.chain_spec),
            provider_factory,
            evm_config.clone(),
        )?;
        info!(elapsed = ?t0.elapsed(), "Blockchain provider ready.");

        let engine_args = engine::Args {
            // tick_interval: self.tick_interval.into(),
            chain_spec: Arc::clone(&self.chain_spec),
            blockchain: blockchain_provider,
            evm_config,
            payload_extradata: Default::default(),
        };
        let engine = engine::create(engine_args);
        let rpc_handle_a =
            rpc::start_rpc_server_a(self.rpc_bind_addr_a, engine.clone(), engine.clone()).await?;
        let rpc_handle_b = rpc::start_server_b(self.rpc_bind_addr_b, engine).await?;

        let () = rpc_handle_a.stopped().await;
        let () = rpc_handle_b.stopped().await;

        Ok(())
    }
}

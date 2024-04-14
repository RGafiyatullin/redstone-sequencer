use std::net::SocketAddr;

use alloy_primitives::ChainId;
use jsonrpsee::RpcModule;
use node::api::{EngineApiServer, EthApiServer};
use structopt::StructOpt;

use crate::{AnyError, Cli};

#[derive(Debug, StructOpt)]
pub struct Node {
    #[structopt(long, short, env = "CHAIN_ID")]
    chain_id: ChainId,

    #[structopt(long, short, env = "RPC_BIND_ADDR", default_value = "0.0.0.0:8551")]
    rpc_bind_addr: SocketAddr,
}

impl Node {
    pub async fn run(&self, _cli: &Cli) -> Result<(), AnyError> {
        let mut rpc_module = RpcModule::new(());
        let api = node::api::Api {};
        rpc_module.merge(EthApiServer::into_rpc(api.clone()))?;
        rpc_module.merge(EngineApiServer::into_rpc(api))?;

        tracing::info!("Binding {} for RPC server", self.rpc_bind_addr);
        let rpc_server = jsonrpsee::server::ServerBuilder::new()
            .build(self.rpc_bind_addr)
            .await?;

        tracing::info!("Starting RPC-server");
        let rpc_running = rpc_server.start(rpc_module);
        rpc_running.stopped().await;

        tracing::info!("RPC-server stopped. Bye!");

        Err("not implemented".into())
    }
}

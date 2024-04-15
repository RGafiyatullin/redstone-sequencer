use std::{net::SocketAddr, path::PathBuf};

use jsonrpsee::RpcModule;
use node::api::EthFilterApiServer;
use node::api::{EngineApiServer, EthApiServer};
use reth_rpc::JwtSecret;
use structopt::StructOpt;

use crate::{AnyError, Cli};

#[derive(Debug, StructOpt)]
pub struct Node {
    // #[structopt(long, env = "CHAIN_ID")]
    // chain_id: ChainId,
    #[structopt(long, env = "RPC_BIND_ADDR_A", default_value = "0.0.0.0:8551")]
    rpc_bind_addr_a: SocketAddr,

    #[structopt(long, env = "RPC_BIND_ADDR_B", default_value = "0.0.0.0:8545")]
    rpc_bind_addr_b: SocketAddr,

    #[structopt(long, env = "BACKEND_ENGINE_API_JWT_SECRET_PATH")]
    engine_api_secret_path: PathBuf,

    #[structopt(long, env = "BACKEND_ENGINE_API_URL")]
    engine_api_url: String,

    #[structopt(long, env = "BACKEND_ETH_API_URL")]
    eth_api_url: String,
}

impl Node {
    pub async fn run(&self, _cli: &Cli) -> Result<(), AnyError> {
        let jwt_secret = JwtSecret::from_file(self.engine_api_secret_path.as_ref())?;
        let api = node::api::Api::new(&self.eth_api_url, &self.engine_api_url, jwt_secret).await?;

        let mut rpc_module_a = RpcModule::new(());
        let mut rpc_module_b = RpcModule::new(());

        rpc_module_a.merge(EthApiServer::into_rpc(api.clone()))?;
        rpc_module_a.merge(EngineApiServer::into_rpc(api.clone()))?;
        rpc_module_a.merge(EthFilterApiServer::into_rpc(api.clone()))?;

        rpc_module_b.merge(EthApiServer::into_rpc(api.clone()))?;
        rpc_module_b.merge(EthFilterApiServer::into_rpc(api))?;

        tracing::info!("Binding {} for RPC server [A]", self.rpc_bind_addr_a);
        let rpc_server_a = jsonrpsee::server::ServerBuilder::new()
            .build(self.rpc_bind_addr_a)
            .await?;

        tracing::info!("Binding {} for RPC server [B]", self.rpc_bind_addr_b);
        let rpc_server_b = jsonrpsee::server::ServerBuilder::new()
            .build(self.rpc_bind_addr_b)
            .await?;

        tracing::info!("Starting RPC-server [A]");
        let rpc_running_a = rpc_server_a.start(rpc_module_a);

        tracing::info!("Starting RPC-server [B]");
        let rpc_running_b = rpc_server_b.start(rpc_module_b);

        let _ = tokio::try_join!(
            async move {
                rpc_running_a.stopped().await;
                tracing::info!("RPC-server [A] stopped.");
                Err::<(), ()>(())
            },
            async move {
                rpc_running_b.stopped().await;
                tracing::info!("RPC-server [B] stopped.");
                Err::<(), ()>(())
            }
        );

        tracing::info!("Bye!");

        Ok(())
    }
}

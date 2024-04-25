use std::net::SocketAddr;

use jsonrpsee::server::ServerHandle;
use jsonrpsee::RpcModule;

use crate::api::EngineApiV3Server;
use crate::api::EthApiServer;

use crate::AnyError;

pub async fn start_rpc_server_a(
    bind_addr: SocketAddr,
    eth_api: impl EthApiServer,
    engine_api: impl EngineApiV3Server,
) -> Result<ServerHandle, AnyError> {
    let mut m = RpcModule::new(());
    m.merge(eth_api.into_rpc())?;
    m.merge(engine_api.into_rpc())?;

    start_rpc_server(bind_addr, m).await
}

pub async fn start_server_b(
    bind_addr: SocketAddr,
    eth_api: impl EthApiServer,
) -> Result<ServerHandle, AnyError> {
    start_rpc_server(bind_addr, eth_api.into_rpc()).await
}

pub async fn start_rpc_server<M>(
    bind_addr: SocketAddr,
    rpc_module: RpcModule<M>,
) -> Result<ServerHandle, AnyError> {
    let server = jsonrpsee::server::Server::builder()
        .build(bind_addr)
        .await?;
    let server_handle = server.start(rpc_module);

    Ok(server_handle)
}

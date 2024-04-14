mod engine_api;
mod eth_api;
pub mod types;

use std::sync::Arc;

use jsonrpsee::http_client::transport::HttpBackend;
use jsonrpsee::http_client::HttpClient;
pub use reth_rpc_api::EngineApiServer;
pub use reth_rpc_api::EthApiServer;

use crate::AnyError;

#[derive(Debug, Clone)]
pub struct Api(Arc<Inner>);

impl Api {
    pub async fn new(engine_api_url: &str) -> Result<Self, AnyError> {
        let engine_api_client =
            jsonrpsee::http_client::HttpClient::<HttpBackend>::builder().build(engine_api_url)?;

        Ok(Self(Arc::new(Inner { engine_api_client })))
    }
}

#[derive(Debug, Clone)]
struct Inner {
    engine_api_client: HttpClient,
}

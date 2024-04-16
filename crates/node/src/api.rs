mod engine_api;
mod eth_api;
mod eth_filter_api;

use std::sync::Arc;
use std::sync::RwLock;

use jsonrpsee::http_client::transport::HttpBackend;
use jsonrpsee::http_client::HttpClient;

use reth_primitives::U256;
use reth_rpc::JwtSecret;
pub use reth_rpc_api::EngineApiServer;
pub use reth_rpc_api::EthApiServer;
pub use reth_rpc_api::EthFilterApiServer;

use crate::auth_layer::AddJwtHeader;
use crate::AnyError;

#[derive(Debug, Clone)]
pub struct Api(Arc<Inner>);

impl Api {
    pub async fn new(
        eth_api_url: &str,
        engine_api_url: &str,
        engine_api_secret: JwtSecret,
    ) -> Result<Self, AnyError> {
        let engine_api_client = jsonrpsee::http_client::HttpClient::<HttpBackend>::builder()
            .set_http_middleware(
                tower::ServiceBuilder::new()
                    .layer(crate::auth_layer::EngineAuthLayer::new(engine_api_secret)),
            )
            .build(engine_api_url)?;

        let eth_api_client =
            jsonrpsee::http_client::HttpClient::<HttpBackend>::builder().build(eth_api_url)?;

        Ok(Self(Arc::new(Inner {
            anonymous_client: eth_api_client,
            authenticated_client: engine_api_client,
            current_block_number: Default::default(),
        })))
    }

    pub fn set_current_block_number(&self, block_number: U256) {
        *self
            .0
            .current_block_number
            .write()
            .expect("rw-lock.write -> poisoned") = block_number;
    }
}

#[derive(Debug)]
struct Inner {
    anonymous_client: HttpClient<HttpBackend>,
    authenticated_client: HttpClient<AddJwtHeader<HttpBackend>>,
    current_block_number: RwLock<U256>,
}

fn to_error_object(error: jsonrpsee::core::ClientError) -> jsonrpsee::types::ErrorObjectOwned {
    use jsonrpsee::core::ClientError;
    use jsonrpsee::types::ErrorObject;

    if let ClientError::Call(error_object) = error {
        error_object
    } else {
        ErrorObject::owned(-32000, error.to_string(), Some(""))
    }
}

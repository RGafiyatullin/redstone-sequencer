mod engine_api;
mod eth_api;

use std::sync::Arc;

use jsonrpsee::http_client::transport::HttpBackend;
use jsonrpsee::http_client::HttpClient;

use reth_rpc::JwtSecret;
pub use reth_rpc_api::EngineApiServer;
pub use reth_rpc_api::EthApiServer;

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
            eth_api_client,
            engine_api_client,
        })))
    }
}

#[derive(Debug, Clone)]
struct Inner {
    eth_api_client: HttpClient<HttpBackend>,
    engine_api_client: HttpClient<AddJwtHeader<HttpBackend>>,
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

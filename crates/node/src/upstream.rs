use std::sync::Arc;

use jsonrpsee::http_client::{transport::HttpBackend, HttpClient};
use reth_rpc::JwtSecret;

use crate::{auth_layer::AddJwtHeader, AnyError};

#[derive(Debug, Clone)]
pub struct Upstream(Arc<Inner>);

impl Upstream {
    pub fn new(
        eth_api_url: &str,
        engine_api_url: &str,
        engine_api_secret: JwtSecret,
    ) -> Result<Self, AnyError> {
        let engine_api_client = HttpClient::<HttpBackend>::builder()
            .set_http_middleware(
                tower::ServiceBuilder::new()
                    .layer(crate::auth_layer::EngineAuthLayer::new(engine_api_secret)),
            )
            .build(engine_api_url)?;

        let eth_api_client = HttpClient::<HttpBackend>::builder().build(eth_api_url)?;

        Ok(Self(Arc::new(Inner {
            anonymous_client: eth_api_client,
            authenticated_client: engine_api_client,
        })))
    }

    pub fn anonymous_client(&self) -> &impl jsonrpsee::core::client::ClientT {
        &self.0.anonymous_client
    }
    pub fn authenticated_client(&self) -> &impl jsonrpsee::core::client::ClientT {
        &self.0.authenticated_client
    }
}

#[derive(Debug)]
struct Inner {
    anonymous_client: HttpClient<HttpBackend>,
    authenticated_client: HttpClient<AddJwtHeader<HttpBackend>>,
}

mod engine_api;
mod eth_api;
mod eth_filter_api;

pub use reth_rpc_api::EngineApiServer;
pub use reth_rpc_api::EthApiServer;
pub use reth_rpc_api::EthFilterApiServer;

use crate::service::Service;
use crate::upstream::Upstream;
use crate::AnyError;

#[derive(Debug, Clone)]
pub struct Api {
    service: Service,
}

impl Api {
    pub fn new(service: Service) -> Result<Self, AnyError> {
        Ok(Self { service })
    }

    fn upstream(&self) -> &Upstream {
        self.service.upstream()
    }
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

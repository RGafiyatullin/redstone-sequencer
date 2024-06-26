pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub mod api;
pub mod auth_layer;

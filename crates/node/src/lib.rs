pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub mod api;
pub mod auth_layer;
pub mod block_number_poller;
pub mod sequencer;
pub mod service;
pub mod upstream;

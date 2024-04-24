#![cfg(feature = "optimism")]

/// CLI argument parsing for the optimism node.
pub mod args;

/// Exports optimism-specific implementations of the [EngineTypes](reth_node_api::EngineTypes)
/// trait.
pub mod engine;
pub use engine::RedstoneEngineTypes;

/// Exports optimism-specific implementations of the
/// [ConfigureEvmEnv](reth_node_api::ConfigureEvmEnv) trait.
pub mod evm;
pub use evm::OptimismEvmConfig;

pub mod node;
pub use node::OptimismNode;

pub mod txpool;

pub mod rpc;

pub mod payload;

pub use reth_optimism_payload_builder::{
    OptimismBuiltPayload, OptimismPayloadBuilder, OptimismPayloadBuilderAttributes,
};

#![allow(missing_docs, rustdoc::missing_crate_level_docs)]

use redstone_sequencer::{
    args::RollupArgs, rpc::SequencerClient, RedstoneEngineTypes, OptimismNode,
};

use clap::Parser;
use reth::cli::Cli;
use reth_node_builder::NodeHandle;
use reth_provider::BlockReaderIdExt;
use std::sync::Arc;

// We use jemalloc for performance reasons
#[cfg(all(feature = "jemalloc", unix))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(feature = "optimism"))]
compile_error!("Cannot build the `op-reth` binary with the `optimism` feature flag disabled. Did you mean to build `reth`?");

#[cfg(feature = "optimism")]
fn main() {
    eprintln!("YES, this is the correct binary");

    reth::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(err) = Cli::<RollupArgs>::parse().run(|builder, rollup_args| async move {
        let NodeHandle {
            node,
            node_exit_future,
        } = builder
            .node(OptimismNode::new(rollup_args.clone()))
            .extend_rpc_modules(move |ctx| {
                // register sequencer tx forwarder
                if let Some(sequencer_http) = rollup_args.sequencer_http.clone() {
                    ctx.registry
                        .set_eth_raw_transaction_forwarder(Arc::new(SequencerClient::new(
                            sequencer_http,
                        )));
                }

                Ok(())
            })
            .launch()
            .await?;

        eprintln!("CHAIN_ID: {}", node.chain_spec().chain().id());

        // If `enable_genesis_walkback` is set to true, the rollup client will need to
        // perform the derivation pipeline from genesis, validating the data dir.
        // When set to false, set the finalized, safe, and unsafe head block hashes
        // on the rollup client using a fork choice update. This prevents the rollup
        // client from performing the derivation pipeline from genesis, and instead
        // starts syncing from the current tip in the DB.
        if node.chain_spec().is_optimism() && !rollup_args.enable_genesis_walkback {
            let client = node.rpc_server_handles.auth.http_client();
            if let Ok(Some(head)) = node.provider.latest_header() {
                reth_rpc_api::EngineApiClient::<RedstoneEngineTypes>::fork_choice_updated_v2(
                    &client,
                    reth_rpc_types::engine::ForkchoiceState {
                        head_block_hash: head.hash(),
                        safe_block_hash: head.hash(),
                        finalized_block_hash: head.hash(),
                    },
                    None,
                )
                .await?;
            }
        }

        node_exit_future.await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}

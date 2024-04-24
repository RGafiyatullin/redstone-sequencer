#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init_timed();

    if let Err(reason) = redstone_sequencer_lite::run(structopt::StructOpt::from_args()).await {
        tracing::error!(error = %reason, "Application failure.");
    }
}

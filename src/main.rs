use structopt::StructOpt;

use redstone_sequencer::AnyError;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let _ = dotenv::dotenv();
    pretty_env_logger::init_timed();

    redstone_sequencer::Cli::from_args().run().await
}

mod cli;
mod db;

pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub async fn run(cli: cli::Cli) -> Result<(), AnyError> {
    cli.run().await
}

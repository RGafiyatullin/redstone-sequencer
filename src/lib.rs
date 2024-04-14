use structopt::StructOpt;

pub mod commands;

pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Init(commands::Init),
    Node(commands::Node),
}

impl Cli {
    pub async fn run(&self) -> Result<(), AnyError> {
        match &self.command {
            Command::Init(inner) => inner.run(self).await,
            Command::Node(inner) => inner.run(self).await,
        }
    }
}

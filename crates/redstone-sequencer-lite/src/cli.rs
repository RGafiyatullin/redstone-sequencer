use crate::AnyError;

mod args;
mod cmd_init;
mod cmd_node;

#[derive(Debug, structopt::StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, structopt::StructOpt)]
enum Command {
    Init(cmd_init::CmdInit),
    Node(cmd_node::CmdNode),
}

impl Cli {
    pub async fn run(&self) -> Result<(), AnyError> {
        match &self.command {
            Command::Init(inner) => inner.run(self).await,
            Command::Node(inner) => inner.run(self).await,
        }
    }
}

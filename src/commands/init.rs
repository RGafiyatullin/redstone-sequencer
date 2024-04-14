use structopt::StructOpt;

use crate::{AnyError, Cli};

#[derive(Debug, StructOpt)]
pub struct Init {}

impl Init {
    pub async fn run(&self, _cli: &Cli) -> Result<(), AnyError> {
        Err("not implemented".into())
    }
}

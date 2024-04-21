use std::{path::{Path, PathBuf}, str::FromStr};

use reth_primitives::ChainSpec;

type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

fn main() -> Result<(), AnyError> {
    for file in std::env::args().skip(1) {
        println!("loading {:?}", file);
        let path = PathBuf::from_str(file.as_str())?;
        let file = std::fs::File::open(path)?;
        let spec: ChainSpec = serde_json::from_reader(file)?;

        println!("{:#?}", spec);
    }
    Ok(())
}
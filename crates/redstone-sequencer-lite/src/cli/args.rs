use std::{fs::File, path::PathBuf, str::FromStr, sync::Arc};

use reth_primitives::ChainSpec;

use crate::AnyError;

pub(crate) fn load_chain_spec(path_str: &str) -> Result<Arc<ChainSpec>, AnyError> {
    let t0 = std::time::Instant::now();

    let path = PathBuf::from_str(path_str)?;
    tracing::debug!("loading chain-spec from {:?}", path_str);

    let file = File::open(path)?;
    let spec: ChainSpec = serde_json::from_reader(file)?;

    tracing::info!(elapsed = ?t0.elapsed(), "loaded chain-spec from {:?}", path_str);

    Ok(Arc::new(spec))
}

use std::{collections::BTreeMap, sync::Arc, time::Instant};

use reth::{
    core::init::InitDatabaseError,
    dirs::{ChainPath, DataDirPath},
};
use reth_db::{
    abstraction::database::Database, models::client_version::ClientVersion, DatabaseEnv,
};
use reth_primitives::{Account, ChainSpec, StaticFileSegment, StorageEntry};
use reth_provider::HashingWriter;
use reth_provider::HistoryWriter;
use reth_provider::{
    BlockHashReader, BlockReader, DatabaseProviderRW, ProviderError, ProviderFactory,
};
use tracing::info;

use crate::AnyError;

pub type ChainDb = ProviderFactory<Arc<DatabaseEnv>>;

pub fn open(
    db_path: ChainPath<DataDirPath>,
    chain_spec: Arc<ChainSpec>,
) -> Result<ChainDb, AnyError> {
    let db = Arc::new(reth_db::init_db(
        &db_path,
        reth_db::mdbx::DatabaseArguments::new(ClientVersion::default()),
    )?);
    let provider_factory = ProviderFactory::new(db, chain_spec, db_path.static_files_path())?;

    Ok(provider_factory)
}

pub fn ensure_genesis(provider: &ChainDb, chain_spec: &ChainSpec) -> Result<(), AnyError> {
    let chainspec_hash = chain_spec.genesis_hash();
    let should_write_genesis = match provider.block_hash(0) {
        Ok(None) | Err(ProviderError::MissingStaticFileBlock(StaticFileSegment::Headers, 0)) => {
            Ok(true)
        }
        Ok(Some(database_hash)) => {
            if database_hash == chainspec_hash {
                Ok(false)
            } else {
                Err(InitDatabaseError::GenesisHashMismatch {
                    chainspec_hash,
                    database_hash,
                }
                .into())
            }
        }
        Err(reason) => Err(InitDatabaseError::Provider(reason)),
    }?;

    if should_write_genesis {
        let provider_rw = provider.provider_rw()?;
    }

    Ok(())
}

fn insert_genesis_hashes<DB: Database>(
    provider: &DatabaseProviderRW<DB>,
    genesis: &reth_primitives::Genesis,
) -> Result<(), ProviderError>
where
    DB: Database,
{
    // insert and hash accounts to hashing table
    let alloc_accounts = genesis
        .alloc
        .clone()
        .into_iter()
        .map(|(addr, account)| (addr, Some(Account::from_genesis_account(account))));
    provider.insert_account_for_hashing(alloc_accounts)?;

    let alloc_storage = genesis
        .alloc
        .clone()
        .into_iter()
        .filter_map(|(addr, account)| {
            // only return Some if there is storage
            account.storage.map(|storage| {
                (
                    addr,
                    storage.into_iter().map(|(key, value)| StorageEntry {
                        key,
                        value: value.into(),
                    }),
                )
            })
        });
    provider.insert_storage_for_hashing(alloc_storage)?;

    Ok(())
}

/// Inserts history indices for genesis accounts and storage.
fn insert_genesis_history<DB: Database>(
    provider: &DatabaseProviderRW<DB>,
    genesis: &reth_primitives::Genesis,
) -> Result<(), ProviderError> {
    let account_transitions = genesis
        .alloc
        .keys()
        .map(|addr| (*addr, vec![0]))
        .collect::<BTreeMap<_, _>>();
    provider.insert_account_history_index(account_transitions)?;

    let storage_transitions = genesis
        .alloc
        .iter()
        .filter_map(|(addr, account)| account.storage.as_ref().map(|storage| (addr, storage)))
        .flat_map(|(addr, storage)| storage.iter().map(|(key, _)| ((*addr, *key), vec![0])))
        .collect::<BTreeMap<_, _>>();
    provider.insert_storage_history_index(storage_transitions)?;

    Ok(())
}

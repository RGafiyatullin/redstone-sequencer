use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use reth::{
    core::init::InitDatabaseError,
    dirs::{ChainPath, DataDirPath},
};
use reth_db::transaction::DbTx;
use reth_db::transaction::DbTxMut;
use reth_db::{
    abstraction::database::Database, models::client_version::ClientVersion, tables, DatabaseEnv,
};
use reth_primitives::{
    stage::StageId, Account, Bytecode, ChainSpec, StaticFileSegment, StorageEntry, B256, U256,
};
use reth_provider::{
    bundle_state::{BundleStateInit, RevertsInit},
    BundleStateWithReceipts, HistoryWriter,
};
use reth_provider::{providers::StaticFileProvider, HashingWriter};
use reth_provider::{providers::StaticFileWriter, OriginalValuesKnown};
use reth_provider::{BlockHashReader, DatabaseProviderRW, ProviderError, ProviderFactory};

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

pub fn ensure_genesis<Db>(
    provider: &ProviderFactory<Db>,
    chain_spec: Arc<ChainSpec>,
) -> Result<B256, AnyError>
where
    Db: Database,
{
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
        let genesis = chain_spec.genesis();
        let provider_rw = provider.provider_rw()?;
        insert_genesis_hashes(&provider_rw, genesis)?;
        insert_genesis_history(&provider_rw, genesis)?;

        let tx = provider_rw.into_tx();
        let static_file_provider = provider.static_file_provider();
        insert_genesis_header::<Db>(&tx, &static_file_provider, Arc::clone(&chain_spec))?;
        insert_genesis_state::<Db>(&tx, genesis)?;

        for stage in StageId::ALL {
            tx.put::<tables::StageCheckpoints>(stage.to_string(), Default::default())?;
        }
        tx.commit()?;
        static_file_provider.commit()?;
    }

    Ok(chainspec_hash)
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

/// Inserts header for the genesis state.
fn insert_genesis_header<DB: Database>(
    tx: &<DB as Database>::TXMut,
    static_file_provider: &StaticFileProvider,
    chain: Arc<ChainSpec>,
) -> Result<(), ProviderError> {
    let (header, block_hash) = chain.sealed_genesis_header().split();

    match static_file_provider.block_hash(0) {
        Ok(None) | Err(ProviderError::MissingStaticFileBlock(StaticFileSegment::Headers, 0)) => {
            let (difficulty, hash) = (header.difficulty, block_hash);
            let mut writer = static_file_provider.latest_writer(StaticFileSegment::Headers)?;
            writer.append_header(header, difficulty, hash)?;
        }
        Ok(Some(_)) => {}
        Err(e) => return Err(e),
    }

    tx.put::<tables::HeaderNumbers>(block_hash, 0)?;
    tx.put::<tables::BlockBodyIndices>(0, Default::default())?;

    Ok(())
}

/// Inserts the genesis state into the database.
pub fn insert_genesis_state<DB: Database>(
    tx: &<DB as Database>::TXMut,
    genesis: &reth_primitives::Genesis,
) -> Result<(), ProviderError> {
    let capacity = genesis.alloc.len();
    let mut state_init: BundleStateInit = HashMap::with_capacity(capacity);
    let mut reverts_init = HashMap::with_capacity(capacity);
    let mut contracts: HashMap<B256, Bytecode> = HashMap::with_capacity(capacity);

    for (address, account) in &genesis.alloc {
        let bytecode_hash = if let Some(code) = &account.code {
            let bytecode = Bytecode::new_raw(code.clone());
            let hash = bytecode.hash_slow();
            contracts.insert(hash, bytecode);
            Some(hash)
        } else {
            None
        };

        // get state
        let storage = account
            .storage
            .as_ref()
            .map(|m| {
                m.iter()
                    .map(|(key, value)| {
                        let value = U256::from_be_bytes(value.0);
                        (*key, (U256::ZERO, value))
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        reverts_init.insert(
            *address,
            (
                Some(None),
                storage
                    .keys()
                    .map(|k| StorageEntry::new(*k, U256::ZERO))
                    .collect(),
            ),
        );

        state_init.insert(
            *address,
            (
                None,
                Some(Account {
                    nonce: account.nonce.unwrap_or_default(),
                    balance: account.balance,
                    bytecode_hash,
                }),
                storage,
            ),
        );
    }
    let all_reverts_init: RevertsInit = HashMap::from([(0, reverts_init)]);

    let bundle = BundleStateWithReceipts::new_init(
        state_init,
        all_reverts_init,
        contracts.into_iter().collect(),
        Default::default(),
        0,
    );

    bundle.write_to_storage(tx, None, OriginalValuesKnown::Yes)?;

    Ok(())
}

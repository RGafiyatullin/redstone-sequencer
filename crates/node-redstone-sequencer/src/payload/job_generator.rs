use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::Semaphore;

use alloy_eips::BlockNumberOrTag;
use reth::tasks::TaskSpawner;
use reth_basic_payload_builder::PayloadBuilder;
use reth_node_api::PayloadBuilderAttributes;
use reth_payload_builder::error::PayloadBuilderError;
use reth_payload_builder::{database::CachedReads, PayloadJob, PayloadJobGenerator};
use reth_primitives::{ChainSpec, B256};
use reth_provider::{BlockReaderIdExt, BlockSource, CanonStateNotification, StateProviderFactory};
use reth_transaction_pool::TransactionPool;

use super::RedstonePayloadJob;

#[derive(Debug, Clone, Default)]
pub struct RedstonePayloadJobGeneratorConfig {
    pub max_payload_tasks: usize,
}

#[derive(Debug)]
pub struct RedstonePayloadJobGenerator<Client, Pool, Tasks, Builder> {
    /// The client that can interact with the chain.
    client: Client,
    /// txpool
    pool: Pool,
    /// How to spawn building tasks
    executor: Tasks,
    /// The configuration for the job generator.
    config: RedstonePayloadJobGeneratorConfig,
    /// Restricts how many generator tasks can be executed at once.
    payload_task_guard: Arc<Semaphore>,
    /// The chain spec.
    chain_spec: Arc<ChainSpec>,
    /// The type responsible for building payloads.
    ///
    /// See [PayloadBuilder]
    builder: Builder,
    /// Stored cached_reads for new payload jobs.
    pre_cached: Option<PrecachedState>,
}

impl<Client, Pool, Tasks, Builder> RedstonePayloadJobGenerator<Client, Pool, Tasks, Builder> {
    /// Creates a new [RedstonePayloadJobGenerator] with the given config and custom [PayloadBuilder]
    pub fn with_builder(
        client: Client,
        pool: Pool,
        executor: Tasks,
        config: RedstonePayloadJobGeneratorConfig,
        chain_spec: Arc<ChainSpec>,
        builder: Builder,
    ) -> Self {
        Self {
            client,
            pool,
            executor,
            payload_task_guard: Arc::new(Semaphore::new(config.max_payload_tasks)),
            config,
            chain_spec,
            builder,
            pre_cached: None,
        }
    }

    // /// Returns the maximum duration a job should be allowed to run.
    // ///
    // /// This adheres to the following specification:
    // // > Client software SHOULD stop the updating process when either a call to engine_getPayload
    // // > with the build process's payloadId is made or SECONDS_PER_SLOT (12s in the Mainnet
    // // > configuration) have passed since the point in time identified by the timestamp parameter.
    // // See also <https://github.com/ethereum/execution-apis/blob/431cf72fd3403d946ca3e3afc36b973fc87e0e89/src/engine/paris.md?plain=1#L137>
    // fn max_job_duration(&self, unix_timestamp: u64) -> Duration {
    //     let duration_until_timestamp = duration_until(unix_timestamp);

    //     // safety in case clocks are bad
    //     let duration_until_timestamp = duration_until_timestamp.min(self.config.deadline * 3);

    //     self.config.deadline + duration_until_timestamp
    // }

    // /// Returns the [Instant](tokio::time::Instant) at which the job should be terminated because it
    // /// is considered timed out.
    // fn job_deadline(&self, unix_timestamp: u64) -> tokio::time::Instant {
    //     tokio::time::Instant::now() + self.max_job_duration(unix_timestamp)
    // }

    // /// Returns a reference to the tasks type
    // pub fn tasks(&self) -> &Tasks {
    //     &self.executor
    // }

    // /// Returns the pre-cached reads for the given parent block if it matches the cached state's
    // /// block.
    // fn maybe_pre_cached(&self, parent: B256) -> Option<CachedReads> {
    //     self.pre_cached.as_ref().filter(|pc| pc.block == parent).map(|pc| pc.cached.clone())
    // }
}

impl<Client, Pool, Tasks, Builder> PayloadJobGenerator
    for RedstonePayloadJobGenerator<Client, Pool, Tasks, Builder>
where
    Client: StateProviderFactory + BlockReaderIdExt + Clone + Unpin + 'static,
    Pool: TransactionPool + Unpin + 'static,
    Tasks: TaskSpawner + Clone + Unpin + 'static,
    Builder: PayloadBuilder<Pool, Client> + Unpin + 'static,
    <Builder as PayloadBuilder<Pool, Client>>::Attributes: Unpin + Clone,
    <Builder as PayloadBuilder<Pool, Client>>::BuiltPayload: Unpin + Clone,
{
    type Job = RedstonePayloadJob<Client, Pool, Tasks, Builder>;

    fn new_payload_job(
        &self,
        attributes: <Self::Job as PayloadJob>::PayloadAttributes,
    ) -> Result<Self::Job, PayloadBuilderError> {
        let parent_block = if attributes.parent().is_zero() {
            // use latest block if parent is zero: genesis block
            self.client
                .block_by_number_or_tag(BlockNumberOrTag::Latest)?
                .ok_or_else(|| PayloadBuilderError::MissingParentBlock(attributes.parent()))?
                .seal_slow()
        } else {
            let block = self
                .client
                .find_block_by_hash(attributes.parent(), BlockSource::Any)?
                .ok_or_else(|| PayloadBuilderError::MissingParentBlock(attributes.parent()))?;

            // we already know the hash, so we can seal it
            block.seal(attributes.parent())
        };

        // let config = PayloadConfig::new(
        //     Arc::new(parent_block),
        //     self.config.extradata.clone(),
        //     attributes,
        //     Arc::clone(&self.chain_spec),
        // );

        // let until = self.job_deadline(config.attributes.timestamp());
        // let deadline = Box::pin(tokio::time::sleep_until(until));

        // let cached_reads = self.maybe_pre_cached(config.parent_block.hash());

        // Ok(RedstonePayloadJob {
        //     config,
        //     client: self.client.clone(),
        //     pool: self.pool.clone(),
        //     executor: self.executor.clone(),
        //     deadline,
        //     interval: tokio::time::interval(self.config.interval),
        //     best_payload: None,
        //     pending_block: None,
        //     cached_reads,
        //     payload_task_guard: self.payload_task_guard.clone(),
        //     metrics: Default::default(),
        //     builder: self.builder.clone(),
        // })
        unimplemented!()
    }

    fn on_new_state(&mut self, new_state: CanonStateNotification) {
        let mut cached = CachedReads::default();

        // extract the state from the notification and put it into the cache
        let committed = new_state.committed();
        let new_state = committed.state();
        for (addr, acc) in new_state.bundle_accounts_iter() {
            if let Some(info) = acc.info.clone() {
                // we want pre cache existing accounts and their storage
                // this only includes changed accounts and storage but is better than nothing
                let storage = acc
                    .storage
                    .iter()
                    .map(|(key, slot)| (*key, slot.present_value))
                    .collect();
                cached.insert_account(addr, info, storage);
            }
        }

        self.pre_cached = Some(PrecachedState {
            block: committed.tip().hash(),
            cached,
        });
    }
}

/// Returns the duration until the given unix timestamp in seconds.
///
/// Returns `Duration::ZERO` if the given timestamp is in the past.
fn duration_until(unix_timestamp_secs: u64) -> Duration {
    let unix_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = Duration::from_secs(unix_timestamp_secs);
    timestamp.saturating_sub(unix_now)
}

/// Pre-filled [CachedReads] for a specific block.
///
/// This is extracted from the [CanonStateNotification] for the tip block.
#[derive(Debug, Clone)]
pub struct PrecachedState {
    /// The block for which the state is pre-cached.
    pub block: B256,
    /// Cached state for the block.
    pub cached: CachedReads,
}

use std::future::Future;

use reth_payload_builder::{error::PayloadBuilderError, KeepPayloadJobAlive, PayloadJob};

use super::{RedstoneBuiltPayload, RedstonePayloadBuilderAttributes};

/// A basic payload job that continuously builds a payload with the best transactions from the pool.
#[derive(Debug)]
pub struct RedstonePayloadJob<Client, Pool, Tasks, Builder>
// where
//     Builder: PayloadBuilder<Pool, Client>,
{
    // /// The configuration for how the payload will be created.
    // config: PayloadConfig<BuilderAttributes>,
    /// The client that can interact with the chain.
    client: Client,
    /// The transaction pool.
    pool: Pool,
    /// How to spawn building tasks
    executor: Tasks,
    // /// The deadline when this job should resolve.
    // deadline: Pin<Box<Sleep>>,
    // /// The interval at which the job should build a new payload after the last.
    // interval: Interval,
    // /// The best payload so far.
    // best_payload: Option<Builder::BuiltPayload>,
    // /// Receiver for the block that is currently being built.
    // pending_block: Option<PendingPayload<Builder::BuiltPayload>>,
    // /// Restricts how many generator tasks can be executed at once.
    // payload_task_guard: PayloadTaskGuard,
    // /// Caches all disk reads for the state the new payloads builds on
    // ///
    // /// This is used to avoid reading the same state over and over again when new attempts are
    // /// triggered, because during the building process we'll repeatedly execute the transactions.
    // cached_reads: Option<CachedReads>,
    // /// metrics for this type
    // metrics: PayloadBuilderMetrics,
    // /// The type responsible for building payloads.
    // ///
    /// See [PayloadBuilder]
    builder: Builder,
}

impl<Client, Pool, Tasks, Builder> PayloadJob for RedstonePayloadJob<Client, Pool, Tasks, Builder>
where
    RedstonePayloadJob<Client, Pool, Tasks, Builder>: Send + Sync,
{
    /// Represents the payload attributes type that is used to spawn this payload job.
    type PayloadAttributes = RedstonePayloadBuilderAttributes;
    /// Represents the future that resolves the block that's returned to the CL.
    type ResolvePayloadFuture =
        std::future::Pending<Result<Self::BuiltPayload, PayloadBuilderError>>;
    // : Future<Output = Result<Self::BuiltPayload, PayloadBuilderError>>
    //     + Send
    //     + Sync
    //     + 'static;
    /// Represents the built payload type that is returned to the CL.
    type BuiltPayload = RedstoneBuiltPayload;

    /// Returns the best payload that has been built so far.
    ///
    /// Note: This is never called by the CL.
    fn best_payload(&self) -> Result<Self::BuiltPayload, PayloadBuilderError> {
        unimplemented!()
    }

    /// Returns the payload attributes for the payload being built.
    fn payload_attributes(&self) -> Result<Self::PayloadAttributes, PayloadBuilderError> {
        unimplemented!()
    }

    /// Called when the payload is requested by the CL.
    ///
    /// This is invoked on [`engine_getPayloadV2`](https://github.com/ethereum/execution-apis/blob/main/src/engine/shanghai.md#engine_getpayloadv2) and [`engine_getPayloadV1`](https://github.com/ethereum/execution-apis/blob/main/src/engine/paris.md#engine_getpayloadv1).
    ///
    /// The timeout for returning the payload to the CL is 1s, thus the future returned should
    /// resolve in under 1 second.
    ///
    /// Ideally this is the best payload built so far, or an empty block without transactions, if
    /// nothing has been built yet.
    ///
    /// According to the spec:
    /// > Client software MAY stop the corresponding build process after serving this call.
    ///
    /// It is at the discretion of the implementer whether the build job should be kept alive or
    /// terminated.
    ///
    /// If this returns [`KeepPayloadJobAlive::Yes`], then the [`PayloadJob`] will be polled
    /// once more. If this returns [`KeepPayloadJobAlive::No`] then the [`PayloadJob`] will be
    /// dropped after this call.
    fn resolve(&mut self) -> (Self::ResolvePayloadFuture, KeepPayloadJobAlive) {
        unimplemented!()
    }
}

impl<Client, Pool, Tasks, Builder> Future for RedstonePayloadJob<Client, Pool, Tasks, Builder> {
    type Output = Result<(), PayloadBuilderError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unimplemented!()
    }
}

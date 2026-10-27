/// Implemented by a worker's state to answer the commands sent through its `FeagiWorker`. The
/// reply variant is chosen by `FeagiWorkerCommands`, so a command can never get the wrong kind
/// of reply
pub trait FeagiWorkerCommandProcessor {
    /// What type of custom request going in is supported
    type CustomRequest;
    /// What type of custom response coming out is supported
    type CustomResponse;

    /// Called when the worker is told to stop. Runs before the stop is acknowledged, so
    /// `FeagiWorker::stop` returning means this has finished. Override this if custom behavior
    /// is needed on the given worker
    async fn on_stop(&mut self) {}

    /// Answers a custom request. Either replies right away (queries) or applies a state change
    /// and returns some form of acknowledgement. Do not block on here as it would impede the
    /// worker
    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse;
}
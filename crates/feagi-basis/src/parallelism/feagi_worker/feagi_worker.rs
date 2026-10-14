//! Things shared amongst FeagiWorker implementation

//region Worker Traits

pub trait FeagiWorkerStatic<> {
    type InitError: core::error::Error;

    fn init_worker_static<F>(&self, worker_function: F) -> Result<(), Self::InitError>
    where
        F: core::future::Future<Output = ()> + Send + 'static;
}


#[cfg(feature = "std")]
pub trait FeagiWorkerDynamic<>: FeagiWorkerStatic<> {
    // TODO allow initing workers in a non-static context
}

//endregion



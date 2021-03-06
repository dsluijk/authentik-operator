use anyhow::anyhow;
use kube::runtime::finalizer;
use thiserror::Error;

#[derive(thiserror::Error, Debug)]
#[error("Reconcile failed: {0}")]
pub struct ReconcileError(#[from] anyhow::Error);

impl From<finalizer::Error<Self>> for ReconcileError {
    fn from(e: finalizer::Error<Self>) -> Self {
        match e {
            finalizer::Error::ApplyFailed(err) => err,
            finalizer::Error::CleanupFailed(err) => err,
            finalizer::Error::AddFinalizer(err) => Self(err.into()),
            finalizer::Error::RemoveFinalizer(err) => Self(err.into()),
            finalizer::Error::UnnamedObject => Self(anyhow!("Finalizer run on unnamed object")),
        }
    }
}

#[derive(Error, Debug)]
pub enum StartError {
    #[error("Kube error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Failed to start webserver.")]
    IOError(#[from] std::io::Error),
    #[error("Failed to initialize tracing logger")]
    TracingError,
}

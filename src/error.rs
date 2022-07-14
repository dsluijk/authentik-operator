use anyhow::anyhow;
use kube::runtime::finalizer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AKApiError {
    #[error("Failed to prepare HTTP request: {0}")]
    StreamError(#[from] hyper::Error),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] hyper::http::Error),
    #[error("Failed to serialize request body: {0}")]
    SerializeError(#[from] serde_json::Error),
}

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

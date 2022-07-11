use kube::runtime::finalizer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReconcileError {
    #[error("Kube error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("JSON Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Object `{0}` has no namespace attached")]
    NoNamespace(String),
    #[error("The provided was invalid: {0}")]
    InvalidObj(String),
    #[error("Failed to commit K8S object: {0}")]
    CommitError(#[from] kube::api::entry::CommitError),
    #[error("An unknown error has occured ({0}). This is likely a bug!")]
    Internal(String),
}

impl From<finalizer::Error<Self>> for ReconcileError {
    fn from(e: finalizer::Error<Self>) -> Self {
        match e {
            finalizer::Error::ApplyFailed(err) => err,
            finalizer::Error::CleanupFailed(err) => err,
            finalizer::Error::AddFinalizer(err) => err.into(),
            finalizer::Error::RemoveFinalizer(err) => err.into(),
            finalizer::Error::UnnamedObject => {
                Self::Internal("Finalizer run on unnamed object".to_string())
            }
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

use thiserror::Error;
use tokio::sync::mpsc::error::SendTimeoutError;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Out of resources")]
    NoWorkersAvailable,
    #[error("Timed out while invoking function")]
    Timeout,
    #[error("encountered error while invoking function")]
    NoReply,
    #[error("encountered internal error while invoking function")]
    InternalNodeHandling,
}

impl<T> From<SendTimeoutError<T>> for BackendError {
    fn from(_: SendTimeoutError<T>) -> BackendError {
        BackendError::Timeout
    }
}

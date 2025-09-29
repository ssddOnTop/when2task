use crate::TaskId;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Task {0} failed: {1}")]
    TaskError(TaskId, String),

    #[error("Join error: {0}")]
    JoinError(#[from] JoinError),
}

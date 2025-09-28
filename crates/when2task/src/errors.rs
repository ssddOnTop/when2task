use crate::TaskId;
use crate::blueprint::BlueprintError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Blueprint error: {0}")]
    BlueprintError(#[from] BlueprintError),

    #[error("Task {0} failed: {1}")]
    TaskError(TaskId, String),

    #[error("Join error: {0}")]
    JoinError(#[from] JoinError),
}

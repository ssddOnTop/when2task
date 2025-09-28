use crate::TaskId;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Blueprint error: {0}")]
    BlueprintError(#[from] BlueprintError),

    #[error("Task {0} failed: {1}")]
    TaskError(TaskId, String),

    #[error("Join error: {0}")]
    JoinError(String),
}

#[derive(Debug, Error)]
pub enum BlueprintError {
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<TaskId>),

    #[error("Task {0} has missing dependency {1}")]
    MissingDependency(TaskId, TaskId),
}

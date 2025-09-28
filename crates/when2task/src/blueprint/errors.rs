use crate::TaskId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlueprintError {
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<TaskId>),

    #[error("Task {0} has missing dependency {1}")]
    MissingDependency(TaskId, TaskId),

    #[error("Internal error: {0}")]
    InternalError(String),
}

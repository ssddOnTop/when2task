use crate::{ExecutionError, Task, TaskId};
use std::collections::HashMap;
use crate::blueprint::Blueprint;

pub enum ExecutionMode {
    /// Everything function is executed truly asynchronously
    /// For example, if a step has tasks A, B and C, we execute
    /// each of them asynchronously.
    TrueAsync,

    /// All the individual tasks in a step are executed in parallel,
    /// but we wait for all the tasks in the same step to complete.
    /// For example, if a step has tasks A, B and C, we execute
    /// the tasks in parallel and wait for all of them.
    PseudoAsync,
    // Parallel,
}

pub struct TaskExecutor<'a, T, E> {
    tasks: HashMap<TaskId, Task<'a, T, E>>,
    mode: ExecutionMode,
}

impl<'a, T, E> TaskExecutor<'a, T, E> {
    pub fn new(execution_mode: ExecutionMode) -> Self {
        Self {
            tasks: Default::default(),
            mode: execution_mode,
        }
    }
    pub fn insert(mut self, task: Task<'a, T, E>) -> Self {
        self.tasks.insert(*task.id(), task);
        self
    }

    pub async fn execute(self) -> Result<(), ExecutionError> {
        let _blueprint = Blueprint::from_tasks(&self.tasks)?;
        todo!()
    }
}

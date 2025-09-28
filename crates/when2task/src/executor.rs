use crate::{Task, TaskId};
use std::collections::HashMap;

pub enum ExecutionMode {
    Async,
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

    pub async fn execute() {
        todo!()
    }
}

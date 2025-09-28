use crate::{Task, TaskId};
use std::collections::HashMap;

pub struct TaskExecutor<'a, T, E> {
    tasks: HashMap<TaskId, Task<'a, T, E>>,
}

impl<'a, T, E> TaskExecutor<'a, T, E> {
    pub fn new() -> Self {
        Self {
            tasks: Default::default(),
        }
    }
    pub fn insert(mut self, task: Task<'a, T, E>) -> Self {
        self.tasks.insert(*task.id(), task);
        self
    }
}

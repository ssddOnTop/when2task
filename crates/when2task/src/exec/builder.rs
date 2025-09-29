use crate::blueprint::Blueprint;
use crate::{BuildError, ExecutionMode, Task, TaskExecutor, TaskId};
use dashmap::DashMap;

pub struct TaskExecutorBuilder<T, E> {
    tasks: DashMap<TaskId, Task<'static, T, E>>,
    mode: ExecutionMode<T, E>,
}

impl<T, E> TaskExecutorBuilder<T, E> {
    pub fn new(execution_mode: ExecutionMode<T, E>) -> Self {
        Self {
            tasks: Default::default(),
            mode: execution_mode,
        }
    }
    pub fn insert(&self, task: Task<'static, T, E>) -> &Self {
        self.tasks.insert(*task.id(), task);
        self
    }

    pub fn build(self) -> Result<TaskExecutor<T, E>, BuildError> {
        let blueprint = Blueprint::from_tasks(&self.tasks)?;

        Ok(TaskExecutor {
            mode: self.mode,
            tasks: self.tasks,
            blueprint,
        })
    }
}

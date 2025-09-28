use crate::blueprint::Blueprint;
use crate::result::{ExecutionResult, TaskResult};
use crate::{ExecutionError, ExecutionMode, Task, TaskId};
use futures::FutureExt;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::task::JoinError;

pub struct TaskExecutor<'a, T, E> {
    tasks: HashMap<TaskId, Task<'a, T, E>>,
    mode: ExecutionMode<T, E>,
}

impl<'a, T, E> TaskExecutor<'a, T, E> {
    pub fn new(execution_mode: ExecutionMode<T, E>) -> Self {
        Self {
            tasks: Default::default(),
            mode: execution_mode,
        }
    }
    pub fn insert(mut self, task: Task<'a, T, E>) -> Self {
        self.tasks.insert(*task.id(), task);
        self
    }

    pub fn task_ids(&self) -> Vec<TaskId> {
        self.tasks.keys().cloned().collect()
    }
}

impl<T: 'static, E: 'static> TaskExecutor<'static, T, E> {
    pub async fn execute(mut self) -> Result<ExecutionResult<T, E>, ExecutionError> {
        let blueprint = Blueprint::from_tasks(&self.tasks)?;

        let mut execution_steps = vec![];
        let total_tasks = self.tasks.len();
        let mut successful_tasks = 0;
        let mut failed_tasks = 0;

        // Execute tasks step by step
        for step_index in 0..blueprint.step_count() {
            let task_ids = blueprint.tasks_at_step(step_index).unwrap();
            let mut step_handles: Vec<
                Pin<Box<dyn Future<Output = Result<TaskResult<T, E>, JoinError>>>>,
            > = vec![];

            // Spawn all tasks in this step concurrently
            for task_id in task_ids {
                let task_id = *task_id;
                if let Some(task) = self.tasks.remove(&task_id) {
                    if let Some(spawn) = self.mode.execution_fn.as_ref() {
                        let handle = spawn(task.into_task())
                            .map(move |r| r.map(|result| TaskResult { task_id, result }));
                        step_handles.push(Box::pin(handle));
                    } else {
                        step_handles.push(Box::pin(
                            task.into_task()
                                .map(move |result| Ok(TaskResult { task_id, result })),
                        ));
                    }
                }
            }

            // Wait for all tasks in this step to complete
            let step_results = futures::future::join_all(step_handles).await;
            let mut current_step_results = vec![];

            for join_result in step_results {
                let task_result = join_result?;
                if task_result.result.is_ok() {
                    successful_tasks += 1;
                } else {
                    failed_tasks += 1;
                }
                current_step_results.push(task_result);
            }

            execution_steps.push(current_step_results);
        }

        Ok(ExecutionResult {
            steps: execution_steps,
            total_tasks,
            successful_tasks,
            failed_tasks,
        })
    }
}

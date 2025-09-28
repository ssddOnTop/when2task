use crate::blueprint::Blueprint;
use crate::result::{ExecutionResult, TaskResult};
use crate::{ExecutionError, ExecutionMode, Task, TaskId};
use futures::FutureExt;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::task::JoinError;

type StepHandle<T, E> = Pin<Box<dyn Future<Output = Result<TaskResult<T, E>, JoinError>>>>;

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
            let mut step_handles: Vec<StepHandle<T, E>> = vec![];

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dependency, Task};
    use std::future;

    #[test]
    fn test_new_executor() {
        let executor = TaskExecutor::<(), ()>::new(ExecutionMode::true_async());
        assert!(executor.tasks.is_empty());
        assert!(executor.task_ids().is_empty());
    }

    #[test]
    fn test_insert_task() {
        let task = Task::new_independent(future::ready(Ok::<(), ()>(())));
        let task_id = *task.id();
        let executor = TaskExecutor::new(ExecutionMode::true_async()).insert(task);

        assert_eq!(executor.tasks.len(), 1);
        let ids = executor.task_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], task_id);
    }

    #[test]
    fn test_multiple_task_insertion() {
        let task1 = Task::new_independent(future::ready(Ok::<(), ()>(())));
        let task2 = Task::new_independent(future::ready(Ok::<(), ()>(())));
        let id1 = *task1.id();
        let id2 = *task2.id();

        let executor = TaskExecutor::new(ExecutionMode::true_async())
            .insert(task1)
            .insert(task2);

        assert_eq!(executor.tasks.len(), 2);
        let mut ids = executor.task_ids();
        ids.sort();
        let mut expected = vec![id1, id2];
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_execute_single_successful_task() {
        let task = Task::new_independent(future::ready(Ok::<i32, ()>(42)));
        let executor = TaskExecutor::new(ExecutionMode::true_async()).insert(task);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 1);
        assert_eq!(result.successful_tasks, 1);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].len(), 1);
        assert!(result.steps[0][0].result.is_ok());
        assert_eq!(result.steps[0][0].result.as_ref().unwrap(), &42);
        assert!(result.all_successful());
    }

    #[tokio::test]
    async fn test_execute_single_failed_task() {
        let task = Task::new_independent(future::ready(Err::<i32, &str>("error")));
        let executor = TaskExecutor::new(ExecutionMode::true_async()).insert(task);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 1);
        assert_eq!(result.successful_tasks, 0);
        assert_eq!(result.failed_tasks, 1);
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].len(), 1);
        assert!(result.steps[0][0].result.is_err());
        assert_eq!(result.steps[0][0].result.as_ref().unwrap_err(), &"error");
        assert!(!result.all_successful());
    }

    #[tokio::test]
    async fn test_execute_multiple_independent_tasks() {
        let task1 = Task::new_independent(future::ready(Ok::<i32, &str>(1)));
        let task2 = Task::new_independent(future::ready(Ok::<i32, &str>(2)));
        let task3 = Task::new_independent(future::ready(Err::<i32, &str>("fail")));

        let executor = TaskExecutor::new(ExecutionMode::true_async())
            .insert(task1)
            .insert(task2)
            .insert(task3);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 3);
        assert_eq!(result.successful_tasks, 2);
        assert_eq!(result.failed_tasks, 1);
        assert_eq!(result.steps.len(), 1); // All independent, so one step
        assert_eq!(result.steps[0].len(), 3);
    }

    #[tokio::test]
    async fn test_execute_dependent_tasks() {
        let task1 = Task::new_independent(future::ready(Ok::<i32, ()>(1)));
        let task1_id = *task1.id();
        let task2 = Task::new(
            future::ready(Ok::<i32, ()>(2)),
            Dependency::from([task1_id]),
        );

        let executor = TaskExecutor::new(ExecutionMode::true_async())
            .insert(task1)
            .insert(task2);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 2);
        assert_eq!(result.successful_tasks, 2);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps.len(), 2); // Two steps due to dependency
        assert_eq!(result.steps[0].len(), 1); // First step has task1
        assert_eq!(result.steps[1].len(), 1); // Second step has task2
    }

    #[tokio::test]
    async fn test_execute_empty_executor() {
        let executor = TaskExecutor::<i32, &str>::new(ExecutionMode::true_async());

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 0);
        assert_eq!(result.successful_tasks, 0);
        assert_eq!(result.failed_tasks, 0);
        assert!(result.steps.is_empty());
        assert!(result.all_successful());
    }

    #[tokio::test]
    async fn test_execute_with_pseudo_async_mode() {
        let task = Task::new_independent(future::ready(Ok::<i32, ()>(100)));
        let executor = TaskExecutor::new(ExecutionMode::pseudo_async(tokio::spawn)).insert(task);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 1);
        assert_eq!(result.successful_tasks, 1);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps[0][0].result.as_ref().unwrap(), &100);
    }
}

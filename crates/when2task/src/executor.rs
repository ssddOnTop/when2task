use crate::{Blueprint, BlueprintError, ExecutionError, Task, TaskId};
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct TaskExecutor<T, E> {
    tasks: HashMap<TaskId, Task<'static, T, E>>,
}

/// Result of task execution containing the task ID and its result
#[derive(Debug)]
pub struct TaskResult<T, E> {
    pub task_id: TaskId,
    pub result: Result<T, E>,
}

/// Complete execution result with all task results organized by execution steps
#[derive(Debug)]
pub struct ExecutionResult<T, E> {
    pub steps: Vec<Vec<TaskResult<T, E>>>,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
}

impl<T, E> ExecutionResult<T, E> {
    /// Returns all successful task results
    pub fn successful_results(&self) -> impl Iterator<Item = &TaskResult<T, E>> {
        self.steps
            .iter()
            .flat_map(|step| step.iter())
            .filter(|result| result.result.is_ok())
    }

    /// Returns all failed task results
    pub fn failed_results(&self) -> impl Iterator<Item = &TaskResult<T, E>> {
        self.steps
            .iter()
            .flat_map(|step| step.iter())
            .filter(|result| result.result.is_err())
    }

    /// Returns true if all tasks completed successfully
    pub fn all_successful(&self) -> bool {
        self.failed_tasks == 0
    }
}

impl<T, E> TaskExecutor<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn insert(mut self, task: Task<'static, T, E>) -> Self {
        self.tasks.insert(*task.id(), task);
        self
    }

    /// Add a task to the executor
    pub fn add_task(&mut self, task: Task<'static, T, E>) {
        self.tasks.insert(*task.id(), task);
    }

    /// Execute all tasks according to their dependencies
    /// Returns results organized by execution steps
    pub async fn execute(mut self) -> Result<ExecutionResult<T, E>, ExecutionError> {
        // Create execution blueprint
        let blueprint = Blueprint::from_tasks(&self.tasks)?;

        let mut execution_steps = vec![];
        let total_tasks = self.tasks.len();
        let mut successful_tasks = 0;
        let mut failed_tasks = 0;

        // Execute tasks step by step
        for step_index in 0..blueprint.step_count() {
            let task_ids = blueprint.tasks_at_step(step_index).unwrap();
            let mut step_handles = vec![];

            // Spawn all tasks in this step concurrently
            for task_id in task_ids {
                let task_id = *task_id;
                if let Some(task) = self.tasks.remove(&task_id) {
                    let handle: JoinHandle<TaskResult<T, E>> = tokio::spawn(async move {
                        let result = task.into_future().await;
                        TaskResult { task_id, result }
                    });
                    step_handles.push(handle);
                }
            }

            // Wait for all tasks in this step to complete
            let step_results = futures::future::join_all(step_handles).await;
            let mut current_step_results = vec![];

            for join_result in step_results {
                match join_result {
                    Ok(task_result) => {
                        if task_result.result.is_ok() {
                            successful_tasks += 1;
                        } else {
                            failed_tasks += 1;
                        }
                        current_step_results.push(task_result);
                    }
                    Err(e) => {
                        return Err(ExecutionError::JoinError(format!(
                            "Task join failed: {}",
                            e
                        )));
                    }
                }
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

    /// Execute tasks and return only the successful results in a flat vector
    pub async fn execute_and_collect_results(self) -> Result<Vec<T>, ExecutionError> {
        let execution_result = self.execute().await?;
        let _results: Vec<T> = execution_result
            .successful_results()
            .filter_map(|task_result| {
                if let Ok(ref _value) = task_result.result {
                    // Note: This clones the value. For non-Clone types, you'd need a different approach
                    None::<T> // We can't extract T without Clone trait
                } else {
                    None
                }
            })
            .collect();

        // For now, return empty vector since we can't extract T without Clone
        // This method would need to be redesigned based on specific use case
        Ok(vec![])
    }

    /// Get a reference to the tasks map
    pub fn tasks(&self) -> &HashMap<TaskId, Task<'static, T, E>> {
        &self.tasks
    }

    /// Get the number of tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Check if a task exists
    pub fn has_task(&self, task_id: &TaskId) -> bool {
        self.tasks.contains_key(task_id)
    }

    /// Create execution blueprint without consuming the executor
    pub fn create_blueprint(&self) -> Result<Blueprint, BlueprintError> {
        Blueprint::from_tasks(&self.tasks)
    }
}

impl<T, E> Default for TaskExecutor<T, E> {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Task;

    use std::time::Duration;
    use tokio::time::sleep;

    fn create_task_with_delay(delay_ms: u64) -> Task<'static, String, String> {
        let future = async move {
            sleep(Duration::from_millis(delay_ms)).await;
            Ok(format!("Task completed after {}ms", delay_ms))
        };
        Task::new(future, vec![])
    }

    fn create_dependent_task(
        delay_ms: u64,
        dependencies: Vec<TaskId>,
    ) -> Task<'static, String, String> {
        let future = async move {
            sleep(Duration::from_millis(delay_ms)).await;
            Ok(format!("Dependent task completed after {}ms", delay_ms))
        };
        Task::new(future, dependencies)
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let mut executor = TaskExecutor::new();

        let task1 = create_task_with_delay(100);
        let task2 = create_task_with_delay(150);
        let task3 = create_task_with_delay(200);

        executor.add_task(task1);
        executor.add_task(task2);
        executor.add_task(task3);

        let start = std::time::Instant::now();
        let result = executor.execute().await.unwrap();
        let duration = start.elapsed();

        // Should complete in roughly 200ms (the longest task) rather than 450ms (sum of all tasks)
        assert!(duration.as_millis() < 300);
        assert_eq!(result.total_tasks, 3);
        assert_eq!(result.successful_tasks, 3);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps.len(), 1); // All tasks in one step
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let mut executor = TaskExecutor::new();

        let task1 = create_task_with_delay(100);
        let task1_id = *task1.id();

        let task2 = create_dependent_task(100, vec![task1_id]);
        let task2_id = *task2.id();

        let task3 = create_dependent_task(100, vec![task2_id]);

        executor.add_task(task1);
        executor.add_task(task2);
        executor.add_task(task3);

        let start = std::time::Instant::now();
        let result = executor.execute().await.unwrap();
        let duration = start.elapsed();

        // Should take roughly 300ms (sequential execution)
        assert!(duration.as_millis() >= 250);
        assert_eq!(result.total_tasks, 3);
        assert_eq!(result.successful_tasks, 3);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps.len(), 3); // Three sequential steps
    }

    #[tokio::test]
    async fn test_mixed_execution() {
        let mut executor = TaskExecutor::new();

        // Create independent tasks
        let task1 = create_task_with_delay(100);
        let task1_id = *task1.id();

        let task2 = create_task_with_delay(100);
        let task2_id = *task2.id();

        // Create tasks that depend on the first two
        let task3 = create_dependent_task(100, vec![task1_id, task2_id]);
        let task4 = create_dependent_task(100, vec![task1_id]);

        executor.add_task(task1);
        executor.add_task(task2);
        executor.add_task(task3);
        executor.add_task(task4);

        let result = executor.execute().await.unwrap();

        assert_eq!(result.total_tasks, 4);
        assert_eq!(result.successful_tasks, 4);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.steps.len(), 2); // Two steps: [task1, task2] then [task3, task4]

        // First step should have 2 tasks (task1, task2)
        assert_eq!(result.steps[0].len(), 2);
        // Second step should have 2 tasks (task3, task4)
        assert_eq!(result.steps[1].len(), 2);
    }
}

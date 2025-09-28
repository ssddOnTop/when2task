use crate::TaskId;

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

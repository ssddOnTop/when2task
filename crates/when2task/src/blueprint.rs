use crate::{BlueprintError, Task, TaskId};
use std::collections::{HashMap, HashSet};

/// Represents a step in the execution plan where all tasks can be executed in parallel
#[derive(Debug, Clone)]
pub struct Step {
    pub tasks: Vec<TaskId>,
}

/// A blueprint that contains the execution plan with ordered steps
#[derive(Debug)]
pub struct Blueprint {
    pub steps: Vec<Step>,
    pub task_to_step: HashMap<TaskId, usize>,
}
impl Blueprint {
    /// Creates an execution blueprint from a collection of tasks
    /// Uses topological sorting to determine execution order
    pub fn from_tasks<T, E>(
        tasks: &HashMap<TaskId, Task<T, E>>,
    ) -> Result<Self, BlueprintError> {
        // Validate that all dependencies exist
        for (task_id, task) in tasks {
            for dep_id in task.dependencies() {
                if !tasks.contains_key(dep_id) {
                    return Err(BlueprintError::MissingDependency(*task_id, *dep_id));
                }
            }
        }

        // Perform topological sorting using Kahn's algorithm
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        let mut adjacency_list: HashMap<TaskId, Vec<TaskId>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for task_id in tasks.keys() {
            in_degree.insert(*task_id, 0);
            adjacency_list.insert(*task_id, vec![]);
        }

        // Calculate in-degrees and build adjacency list
        for (task_id, task) in tasks {
            for dep_id in task.dependencies() {
                adjacency_list.get_mut(dep_id).unwrap().push(*task_id);
                *in_degree.get_mut(task_id).unwrap() += 1;
            }
        }

        let mut steps = vec![];
        let mut task_to_step = HashMap::new();
        let mut processed = HashSet::new();
        let mut step_index = 0;

        // Process tasks level by level
        loop {
            // Find all tasks with no remaining dependencies
            let ready_tasks: Vec<TaskId> = in_degree
                .iter()
                .filter(|(task_id, degree)| **degree == 0 && !processed.contains(*task_id))
                .map(|(task_id, _)| *task_id)
                .collect();

            if ready_tasks.is_empty() {
                break;
            }

            // Create execution step
            let step = Step {
                tasks: ready_tasks.clone(),
            };

            // Record step mapping
            for task_id in &ready_tasks {
                task_to_step.insert(*task_id, step_index);
                processed.insert(*task_id);
            }

            steps.push(step);

            // Update in-degrees for dependent tasks
            for task_id in ready_tasks {
                for dependent_id in &adjacency_list[&task_id] {
                    if let Some(degree) = in_degree.get_mut(dependent_id) {
                        *degree -= 1;
                    }
                }
            }

            step_index += 1;
        }

        // Check for circular dependencies
        if processed.len() != tasks.len() {
            let remaining: Vec<TaskId> = tasks
                .keys()
                .filter(|id| !processed.contains(id))
                .cloned()
                .collect();
            return Err(BlueprintError::CircularDependency(remaining));
        }

        Ok(Blueprint {
            steps,
            task_to_step,
        })
    }

    /// Returns the total number of execution steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns the step index for a given task
    pub fn step_for_task(&self, task_id: &TaskId) -> Option<usize> {
        self.task_to_step.get(task_id).copied()
    }

    /// Returns all tasks that can be executed in parallel at the given step
    pub fn tasks_at_step(&self, step: usize) -> Option<&[TaskId]> {
        self.steps.get(step).map(|s| s.tasks.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Task;

    use std::future;

    fn create_dummy_task() -> Task<'static, (), ()> {
        let future = future::ready(Ok(()));
        Task::new(future, vec![])
    }

    #[test]
    fn test_simple_blueprint() {
        let mut tasks = HashMap::new();
        let task1 = create_dummy_task();
        let task2 = create_dummy_task();

        let id1 = *task1.id();
        let id2 = *task2.id();

        tasks.insert(id1, task1);
        tasks.insert(id2, task2);

        let blueprint = Blueprint::from_tasks(&tasks).unwrap();
        assert_eq!(blueprint.step_count(), 1);
        assert_eq!(blueprint.tasks_at_step(0).unwrap().len(), 2);
    }

    #[test]
    fn test_sequential_blueprint() {
        let mut tasks = HashMap::new();
        let task1 = create_dummy_task();
        let id1 = *task1.id();

        let future2 = future::ready(Ok(()));
        let task2 = Task::new(future2, vec![id1]);
        let id2 = *task2.id();

        tasks.insert(id1, task1);
        tasks.insert(id2, task2);

        let blueprint = Blueprint::from_tasks(&tasks).unwrap();
        assert_eq!(blueprint.step_count(), 2);
        assert_eq!(blueprint.tasks_at_step(0).unwrap().len(), 1);
        assert_eq!(blueprint.tasks_at_step(1).unwrap().len(), 1);
    }
}

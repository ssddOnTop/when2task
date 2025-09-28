use crate::blueprint::BlueprintError;
use crate::{Task, TaskId};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Step {
    pub tasks: Vec<TaskId>,
}
pub struct Blueprint {
    pub steps: Vec<Step>,
}

impl Blueprint {
    pub fn from_tasks<T, E>(tasks: &HashMap<TaskId, Task<T, E>>) -> Result<Self, BlueprintError> {
        // Validate that all dependencies exist
        for (task_id, task) in tasks {
            for dep_id in task.dependencies().into_iter() {
                if !tasks.contains_key(&dep_id) {
                    return Err(BlueprintError::MissingDependency(*task_id, dep_id));
                }
            }
        }

        // Perform topological sorting using Kahn's algorithm
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        let mut adjacency_list: HashMap<TaskId, Vec<TaskId>> = HashMap::new();

        // Initialize in-degrees for all tasks
        for task_id in tasks.keys() {
            in_degree.insert(*task_id, 0);
        }

        // Calculate in-degrees and build adjacency list
        for (task_id, task) in tasks {
            for dep_id in task.dependencies().into_iter() {
                adjacency_list.entry(dep_id).or_default().push(*task_id);
                *in_degree.get_mut(task_id).ok_or_else(|| {
                    BlueprintError::InternalError(format!(
                        "Task {} not found in_degree map during dependency calculation",
                        task_id
                    ))
                })? += 1;
            }
        }

        let mut steps = vec![];
        let mut processed = HashSet::new();

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
                processed.insert(*task_id);
            }

            steps.push(step);

            // Update in-degrees for dependent tasks
            for task_id in ready_tasks {
                if let Some(dependents) = adjacency_list.get(&task_id) {
                    for dependent_id in dependents {
                        if let Some(degree) = in_degree.get_mut(dependent_id) {
                            *degree -= 1;
                        }
                    }
                }
            }
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

        Ok(Blueprint { steps })
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
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
        Task::new_independent(future)
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

        let task2 = Task::new(future::ready(Ok(())), vec![id1]);
        let id2 = *task2.id();

        tasks.insert(id1, task1);
        tasks.insert(id2, task2);

        let blueprint = Blueprint::from_tasks(&tasks).unwrap();
        assert_eq!(blueprint.step_count(), 2);
        assert_eq!(blueprint.tasks_at_step(0).unwrap().len(), 1);
        assert_eq!(blueprint.tasks_at_step(1).unwrap().len(), 1);
    }
}

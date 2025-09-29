use crate::TaskId;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Dependency {
    /// No dependencies - can execute immediately
    #[default]
    None,

    /// Depends on a single specific task
    Task(TaskId),

    // /// All dependencies must be satisfied (most common case)
    // All(Vec<Dependency>),

    /*
        TODO: Any would be only available for parallel execution mode, it doesn't make sense in async.
        /// Any one of the dependencies must be satisfied
        Any(Vec<Dependency>),
    */
    // /// Negative dependency - execute when the dependency fails or doesn't exist
    // /// Useful for fallback tasks or cleanup operations
    // Not(Box<Dependency>),
    /// Combine deps
    // TODO: Drop ALL and rename this to And?
    Combine(Box<Dependency>, Box<Dependency>),
}

impl Dependency {
    pub fn and(self, dependency: impl Into<Dependency>) -> Self {
        Self::Combine(Box::new(self), Box::new(dependency.into()))
    }
    /// Check if this dependency is satisfied given a set of completed tasks
    pub fn is_satisfied(&self, completed_tasks: &std::collections::HashSet<TaskId>) -> bool {
        match self {
            Dependency::None => true,
            Dependency::Task(id) => completed_tasks.contains(id),
            // Dependency::All(deps) => deps.iter().all(|d| d.is_satisfied(completed_tasks)),
            // Dependency::Not(dep) => !dep.is_satisfied(completed_tasks),
            Dependency::Combine(a, b) => {
                a.is_satisfied(completed_tasks) && b.is_satisfied(completed_tasks)
            }
        }
    }
}

impl<'a> IntoIterator for &'a Dependency {
    type Item = TaskId;
    type IntoIter = DependencyIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Dependency {
    /// Returns an iterator over all TaskIds in this dependency
    pub fn iter(&self) -> DependencyIter<'_> {
        DependencyIter::new(self)
    }
}

/// Iterator over TaskIds in a Dependency
pub struct DependencyIter<'a> {
    stack: Vec<&'a Dependency>,
}

impl<'a> DependencyIter<'a> {
    fn new(dependency: &'a Dependency) -> Self {
        Self {
            stack: vec![dependency],
        }
    }
}

impl<'a> Iterator for DependencyIter<'a> {
    type Item = TaskId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(dep) = self.stack.pop() {
            match dep {
                Dependency::None => continue,
                Dependency::Task(task_id) => return Some(*task_id),
                Dependency::Combine(a, b) => {
                    self.stack.push(b);
                    self.stack.push(a);
                }
            }
        }
        None
    }
}

impl From<TaskId> for Dependency {
    fn from(task_id: TaskId) -> Self {
        Self::Task(task_id)
    }
}

impl<I: IntoIterator<Item = TaskId>> From<I> for Dependency {
    fn from(task_ids: I) -> Self {
        let mut unit = Dependency::None;
        for task in task_ids.into_iter().map(Dependency::from) {
            unit = Dependency::Combine(Box::new(unit), Box::new(task));
        }
        unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_task_ids() -> (TaskId, TaskId, TaskId) {
        (TaskId::generate(), TaskId::generate(), TaskId::generate())
    }

    #[test]
    fn test_dependency_satisfaction() {
        let (task1, task2, _) = create_test_task_ids();

        // Test None - always satisfied
        let none_dep = Dependency::None;
        assert!(none_dep.is_satisfied(&HashSet::new()));

        let mut completed = HashSet::new();
        completed.insert(task1);
        assert!(none_dep.is_satisfied(&completed));

        // Test Task - satisfied when task is completed
        let task_dep = Dependency::Task(task1);
        assert!(task_dep.is_satisfied(&completed));
        assert!(!task_dep.is_satisfied(&HashSet::new()));

        // Test different task not satisfied
        completed.clear();
        completed.insert(task2);
        assert!(!task_dep.is_satisfied(&completed));
    }

    #[test]
    fn test_dependency_combine() {
        let (task1, task2, _) = create_test_task_ids();
        let combine_dep = Dependency::Combine(
            Box::new(Dependency::Task(task1)),
            Box::new(Dependency::Task(task2)),
        );

        // Both tasks completed - satisfied
        let mut completed = HashSet::new();
        completed.insert(task1);
        completed.insert(task2);
        assert!(combine_dep.is_satisfied(&completed));

        // Only one task completed - not satisfied
        completed.remove(&task2);
        assert!(!combine_dep.is_satisfied(&completed));

        // No tasks completed - not satisfied
        assert!(!combine_dep.is_satisfied(&HashSet::new()));

        // Test combining with None
        let none_combine = Dependency::Combine(
            Box::new(Dependency::None),
            Box::new(Dependency::Task(task1)),
        );

        completed.clear();
        completed.insert(task1);
        assert!(none_combine.is_satisfied(&completed));
        assert!(!none_combine.is_satisfied(&HashSet::new()));
    }

    #[test]
    fn test_dependency_and_chaining() {
        let (task1, task2, task3) = create_test_task_ids();

        // Test basic and() method
        let combined = Dependency::Task(task1).and(Dependency::Task(task2));
        match combined {
            Dependency::Combine(a, b) => {
                assert!(matches!(*a, Dependency::Task(id) if id == task1));
                assert!(matches!(*b, Dependency::Task(id) if id == task2));
            }
            _ => panic!("Expected Combine variant"),
        }

        // Test chaining multiple dependencies
        let chained = Dependency::Task(task1)
            .and(Dependency::Task(task2))
            .and(Dependency::Task(task3));

        let mut completed = HashSet::new();
        completed.insert(task1);
        completed.insert(task2);
        completed.insert(task3);

        assert!(chained.is_satisfied(&completed));

        // Remove one task and it should not be satisfied
        completed.remove(&task3);
        assert!(!chained.is_satisfied(&completed));
    }

    #[test]
    fn test_dependency_iter() {
        // Test None - never returns anything
        let none_dep = Dependency::None;
        let mut iter = none_dep.iter();
        assert_eq!(iter.next(), None);

        // Test Task - returns task ID once
        let task_id = TaskId::generate();
        let task_dep = Dependency::Task(task_id);
        let mut iter = task_dep.iter();

        assert_eq!(iter.next(), Some(task_id));
        assert_eq!(iter.next(), None);

        // Test Combine - returns task IDs from left then right
        let (task1, task2, _) = create_test_task_ids();
        let combine_dep = Dependency::Combine(
            Box::new(Dependency::Task(task1)),
            Box::new(Dependency::Task(task2)),
        );

        let collected: Vec<TaskId> = combine_dep.iter().collect();
        assert_eq!(collected.len(), 2);
        assert!(collected.contains(&task1));
        assert!(collected.contains(&task2));
    }

    #[test]
    fn test_iter_collect() {
        let (task1, task2, task3) = create_test_task_ids();

        // Test collecting from Task
        let task_dep = Dependency::Task(task1);
        let collected: Vec<TaskId> = task_dep.iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], task1);

        // Test collecting from nested Combine
        let nested = Dependency::Combine(
            Box::new(Dependency::Combine(
                Box::new(Dependency::Task(task1)),
                Box::new(Dependency::Task(task2)),
            )),
            Box::new(Dependency::Task(task3)),
        );
        let collected: Vec<TaskId> = nested.iter().collect();
        assert_eq!(collected.len(), 3);
        assert!(collected.contains(&task1));
        assert!(collected.contains(&task2));
        assert!(collected.contains(&task3));
    }

    #[test]
    fn test_iter_for_loop() {
        let (task1, task2, _) = create_test_task_ids();
        let combine_dep = Dependency::Combine(
            Box::new(Dependency::Task(task1)),
            Box::new(Dependency::Task(task2)),
        );

        let mut count = 0;
        let mut found_tasks = Vec::new();

        for task_id in combine_dep.iter() {
            found_tasks.push(task_id);
            count += 1;
        }

        assert_eq!(count, 2);
        assert!(found_tasks.contains(&task1));
        assert!(found_tasks.contains(&task2));
    }

    #[test]
    fn test_iter_reusability() {
        let task_id = TaskId::generate();
        let dep = Dependency::Task(task_id);

        // Iterator doesn't consume the dependency - can be used multiple times
        let first_iter: Vec<TaskId> = dep.iter().collect();
        assert_eq!(first_iter, vec![task_id]);

        // Can iterate again without consuming the original dependency
        let second_iter: Vec<TaskId> = dep.iter().collect();
        assert_eq!(second_iter, vec![task_id]);

        // Original dependency is unchanged
        assert!(matches!(dep, Dependency::Task(id) if id == task_id));
    }

    #[test]
    fn test_dependency_from_conversions() {
        let (task1, task2, task3) = create_test_task_ids();

        // Test From<TaskId>
        let dep: Dependency = task1.into();
        match dep {
            Dependency::Task(id) => assert_eq!(id, task1),
            _ => panic!("Expected Task variant"),
        }

        // Test From<Vec<TaskId>> - empty
        let empty_deps: Dependency = Vec::<TaskId>::new().into();
        assert!(matches!(empty_deps, Dependency::None));

        // Test From<Vec<TaskId>> - single item
        let single_dep: Dependency = vec![task1].into();
        match single_dep {
            Dependency::Combine(none_box, task_box) => {
                assert!(matches!(*none_box, Dependency::None));
                assert!(matches!(*task_box, Dependency::Task(id) if id == task1));
            }
            _ => panic!("Expected Combine variant"),
        }

        // Test From<Vec<TaskId>> - multiple items (all must be satisfied)
        let multi_dep: Dependency = vec![task1, task2, task3].into();
        let mut completed = HashSet::new();
        completed.insert(task1);
        completed.insert(task2);
        completed.insert(task3);

        assert!(multi_dep.is_satisfied(&completed));

        completed.remove(&task2);
        assert!(!multi_dep.is_satisfied(&completed));
    }

    #[test]
    fn test_dependency_traits_and_complex_scenarios() {
        let (task1, task2, task3) = create_test_task_ids();

        // Test Default trait
        let default_dep = Dependency::default();
        assert!(matches!(default_dep, Dependency::None));
        assert!(default_dep.is_satisfied(&HashSet::new()));

        // Test Clone trait
        let original = Dependency::Task(task1);
        let cloned = original.clone();
        match (original, cloned) {
            (Dependency::Task(id1), Dependency::Task(id2)) => assert_eq!(id1, id2),
            _ => panic!("Expected both to be Task variants"),
        }

        // Test Debug trait
        let dep = Dependency::Task(task1);
        let debug_str = format!("{dep:?}");
        assert!(debug_str.contains("Task"));

        // Test complex chained dependency scenarios
        let complex_dep = Dependency::Task(task1)
            .and(Dependency::Task(task2))
            .and(Dependency::Task(task3));

        let mut completed = HashSet::new();

        // Progressive completion testing
        assert!(!complex_dep.is_satisfied(&completed));

        completed.insert(task1);
        assert!(!complex_dep.is_satisfied(&completed));

        completed.insert(task2);
        assert!(!complex_dep.is_satisfied(&completed));

        completed.insert(task3);
        assert!(complex_dep.is_satisfied(&completed));
    }
}

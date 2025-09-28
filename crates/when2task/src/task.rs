use crate::{Dependency, TaskId};
use derive_getters::Getters;
use std::future::Future;
use std::pin::Pin;

pub type UnitTask<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;

#[derive(Getters)]
pub struct Task<'a, T, E> {
    id: TaskId,
    #[getter(skip)]
    task: UnitTask<'a, T, E>,
    dependencies: Dependency,
}

impl<'a, T, E> Task<'a, T, E> {
    pub fn new<F: Future<Output = Result<T, E>> + 'a>(
        task: F,
        dependencies: impl Into<Dependency>,
    ) -> Self {
        let id = TaskId::generate();

        Self {
            id,
            task: Box::pin(task),
            dependencies: dependencies.into(),
        }
    }

    /// Convenience method to create a task with no dependencies
    pub fn new_independent<F: Future<Output = Result<T, E>> + 'a>(task: F) -> Self {
        Self::new(task, [])
    }
}

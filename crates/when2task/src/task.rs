use crate::TaskId;
use derive_getters::Getters;
use std::future::Future;
use std::pin::Pin;

pub type UnitTask<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;

#[derive(Getters)]
pub struct Task<'a, T, E> {
    id: TaskId,
    #[getter(skip)]
    task: UnitTask<'a, T, E>,
    dependencies: Vec<TaskId>,
}

impl<'a, T, E> Task<'a, T, E> {
    /// Get a reference to the task future (this consumes self to move the future)
    pub fn into_future(self) -> UnitTask<'a, T, E> {
        self.task
    }
    pub fn new<D: IntoIterator<Item = TaskId>, Task: Future<Output = Result<T, E>> + Send + 'a>(
        task: Task,
        dependencies: D,
    ) -> Self {
        let id = TaskId::generate();
        let dependencies = dependencies.into_iter().collect::<Vec<_>>();

        Self {
            id,
            task: Box::pin(task),
            dependencies,
        }
    }
}

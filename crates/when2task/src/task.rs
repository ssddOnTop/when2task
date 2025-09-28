use crate::TaskId;
use std::pin::Pin;
use derive_getters::Getters;

pub type UnitTask<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>;

#[derive(Getters)]
pub struct Task<'a, T, E> {
    id: TaskId,
    #[getter(skip)]
    task: UnitTask<'a, T, E>,
    dependencies: Vec<TaskId>,
}

impl<'a, T, E> Task<'a, T, E> {
    pub fn new<D: IntoIterator<Item = TaskId>>(task: UnitTask<'a, T, E>, dependencies: D) -> Self {
        let id = TaskId::generate();
        let dependencies = dependencies.into_iter().collect::<Vec<_>>();

        Self {
            id,
            task,
            dependencies,
        }
    }
}

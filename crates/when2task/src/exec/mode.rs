use derive_getters::Getters;
use std::pin::Pin;
use tokio::task::JoinHandle;

#[derive(Getters)]
pub struct ExecutionMode<T, E> {
    pub(crate) execution_fn: Option<
        Box<
            dyn Fn(
                    Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>>,
                ) -> JoinHandle<Result<T, E>>
                + Send
                + 'static,
        >,
    >,
}

impl<T, E> ExecutionMode<T, E> {
    /// Everything function is executed truly asynchronously
    /// For example, if a step has tasks A, B and C, we execute
    /// each of them asynchronously.
    pub fn true_async() -> Self {
        Self { execution_fn: None }
    }

    /// All the individual tasks in a step are executed in parallel,
    /// but we wait for all the tasks in the same step to complete.
    /// For example, if a step has tasks A, B and C, we execute
    /// the tasks in parallel and wait for all of them.
    pub fn pseudo_async<
        F: Fn(
                Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'static>>,
            ) -> JoinHandle<Result<T, E>>
            + Send
            + 'static,
    >(
        execution_fn: F,
    ) -> Self {
        Self {
            execution_fn: Some(Box::new(execution_fn)),
        }
    }

    /*pub fn parallel() -> Self {
        todo!()
    }*/
}

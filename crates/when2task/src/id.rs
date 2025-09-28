use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, PartialOrd, Ord)]
pub struct TaskId(u128);

impl TaskId {
    pub fn generate() -> Self {
        TaskId(uuid::Uuid::new_v4().as_u128())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u128);

impl TaskId {
    pub fn generate() -> Self {
        TaskId(uuid::Uuid::new_v4().as_u128())
    }
}

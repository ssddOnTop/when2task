use crate::blueprint::BlueprintError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Blueprint error: {0}")]
    BlueprintError(#[from] BlueprintError),
}

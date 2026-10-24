use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().as_u128() as u64)
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u64);

impl TabId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().as_u128() as u64)
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

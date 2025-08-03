use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RigidBody {
    pub momentum: i32,
}

impl RigidBody {
    pub fn new() -> Self {
        Self { momentum: 1 }
    }
}

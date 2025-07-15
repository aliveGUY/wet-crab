use crate::index::engine::components::{TransformComponent, ColliderComponent};

pub struct ColliderSystem;

impl ColliderSystem {
    pub fn update() {
        query!((TransformComponent, ColliderComponent), |_id, transform, collider| {
        });
    }
}

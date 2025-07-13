use crate::index::Transform;
use crate::index::Collider;

pub struct ColliderSystem;

impl ColliderSystem {
    pub fn update() {
        query!((Transform, Collider), |_id, transform, collider| {
            println!("🔍 Entity at position");
        });
    }
}

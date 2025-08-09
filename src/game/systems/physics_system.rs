use crate::index::engine::components::{ Collider, Transform };
use crate::{ query, query_get_all };

pub struct PhysicsSystem;

impl PhysicsSystem {
    pub fn update() {
        let all_colliders = query_get_all!(Collider, Transform);

        // Query entities that have both Transform and Collider components
        query!((Transform, Collider), |current_entity_id, current_transform, current_collider| {
            for (other_entity_id, other_collider, other_transform) in &all_colliders {
                if current_entity_id == *other_entity_id {
                    continue;
                }

                if current_collider.ignored_layers.contains(&other_collider.layer) {
                    continue;
                }

                if
                    current_collider
                        .clone()
                        .is_collides(
                            other_collider.clone(),
                            current_transform.clone(),
                            other_transform.clone()
                        )
                {
                }
            }
        })
    }
}

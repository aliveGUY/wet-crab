use crate::index::engine::systems::entity_component_system::{spawn, EntityId};
use crate::index::engine::components::{CameraComponent, Metadata, Transform};
use crate::index::PLAYER_ENTITY_ID;

pub fn spawn_player() -> EntityId {
    let player_entity_id = spawn();
    *PLAYER_ENTITY_ID.write().unwrap() = Some(player_entity_id.clone());
    crate::insert_many!(
        player_entity_id.clone(),
        CameraComponent::new(),
        Transform::new(0.0, 0.0, 0.0), // Transform component for position
        Metadata::new_with_role("Player Camera", Some("player"))
    );
    player_entity_id
}

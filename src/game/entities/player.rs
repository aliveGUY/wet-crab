use crate::index::engine::systems::entity_component_system::{spawn, EntityId};
use crate::index::engine::components::{CameraComponent, Metadata};
use crate::index::PLAYER_ENTITY_ID;

pub fn spawn_player() -> EntityId {
    let player_entity_id = spawn();
    *PLAYER_ENTITY_ID.write().unwrap() = Some(player_entity_id.clone());
    crate::insert_many!(
        player_entity_id.clone(),
        CameraComponent::new(),
        Metadata::new("Player Camera")
    );
    player_entity_id
}

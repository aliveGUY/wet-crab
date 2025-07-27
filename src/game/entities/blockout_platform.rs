use crate::index::engine::systems::entity_component_system::{ spawn, EntityId };
use crate::index::engine::components::{ Transform, Metadata };
use crate::index::engine::managers::assets_manager::{ Assets, get_static_object_copy };
use crate::index::PLAYER_ENTITY_ID;

fn get_player_position() -> [f32; 3] {
    let player_id_guard = PLAYER_ENTITY_ID.read().unwrap();
    if let Some(player_id) = player_id_guard.as_ref() {
        if let Some(transform) = crate::get_query_by_id!(player_id, (Transform)) {
            // Extract position from transform matrix
            let matrix = transform.get_matrix();
            return [matrix[3], matrix[7], matrix[11]]; // x, y, z from transform matrix
        }
    }
    [0.0, 0.0, 0.0] 
}

pub fn spawn_blockout_platform() -> EntityId {
    let block_entity_id = spawn();
    let player_position = get_player_position();

    crate::insert_many!(
        block_entity_id.clone(),
        get_static_object_copy(Assets::BlockoutPlatform),
        Transform::new(player_position[0], player_position[1], player_position[2]),
        Metadata::new("Blockout Platform")
    );
    block_entity_id
}

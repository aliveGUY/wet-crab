use crate::index::engine::systems::entity_component_system::{spawn, EntityId};
use crate::index::engine::components::{Transform, Metadata};
use crate::index::engine::managers::assets_manager::{Assets, get_static_object_copy};

pub fn spawn_blockout_platform() -> EntityId {
    let block_entity_id = spawn();
    crate::insert_many!(
        block_entity_id.clone(),
        get_static_object_copy(Assets::BlockoutPlatform),
        Transform::new(2.0, -3.0, -5.0),
        Metadata::new("Blockout Platform")
    );
    block_entity_id
}

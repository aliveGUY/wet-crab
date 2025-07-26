use crate::index::engine::systems::entity_component_system::{spawn, EntityId};
use crate::index::engine::components::{Transform, Metadata};
use crate::index::engine::managers::assets_manager::{Assets, get_static_object_copy};

pub fn spawn_chair() -> EntityId {
    let chair_entity_id = spawn();
    crate::insert_many!(
        chair_entity_id.clone(),
        get_static_object_copy(Assets::Chair),
        Transform::new(2.0, -3.0, -5.0),
        Metadata::new("Chair")
    );
    chair_entity_id
}

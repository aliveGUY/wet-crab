use crate::index::engine::modules::{spawn, EntityId};
use crate::index::engine::components::{Transform, Metadata};
use crate::index::engine::managers::assets_manager::{Assets, get_static_object_copy};

#[allow(dead_code)]
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

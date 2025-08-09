use crate::index::engine::modules::{spawn, EntityId};
use crate::index::engine::components::{Transform, Metadata};
use crate::index::engine::managers::assets_manager::{Assets, get_animated_object_copy};

#[allow(dead_code)]
pub fn spawn_testing_doll() -> EntityId {
    let doll_entity_id = spawn();
    crate::insert_many!(
        doll_entity_id.clone(),
        get_animated_object_copy(Assets::TestingDoll),
        Transform::new(-2.0, -3.0, -5.0),
        Metadata::new("TestingDoll", None, None)
    );
    doll_entity_id
}

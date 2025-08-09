//! ECS Macros - Simple macros that call ECS functions directly

pub use super::ecs::EntityId;

// ——————————————————————————————————————————————————————————— Entity Macros ————

#[macro_export]
macro_rules! insert_many {
    ($entity:expr $(, $comp:expr)+ $(,)?) => {
        {
            $(
                $crate::index::engine::modules::ecs::insert(&$entity, $comp);
            )+
        }
    };
}

#[macro_export]
macro_rules! query {
    // Single component
    (($c1:ty), | $id:ident, $a1:ident | $body:block) => {
        {
            let results = $crate::index::engine::modules::ecs::query_all::<$c1>();
            #[allow(unused_mut)]
            for ($id, mut $a1) in results {
                $body
                $crate::index::engine::modules::ecs::insert(&$id, $a1);
            }
        }
    };
    // Two components
    (($c1:ty, $c2:ty), | $id:ident, $a1:ident, $a2:ident | $body:block) => {
        {
            let results = $crate::index::engine::modules::ecs::query_all2::<$c1, $c2>();
            #[allow(unused_mut)]
            for ($id, mut $a1, mut $a2) in results {
                $body
                $crate::index::engine::modules::ecs::insert(&$id, $a1);
                $crate::index::engine::modules::ecs::insert(&$id, $a2);
            }
        }
    };
}

#[macro_export]
macro_rules! query_by_id {
    // Single component
    ($eid:expr, ($c1:ty), | $a1:ident | $body:block) => {
        {
            $crate::index::engine::modules::ecs::get_component_mut::<$c1, _, _>(&$eid, |$a1| $body);
        }
    };
    // Two components
    ($eid:expr, ($c1:ty, $c2:ty), | $a1:ident, $a2:ident | $body:block) => {
        {
            if let (Some(mut comp1), Some(mut comp2)) = (
                $crate::index::engine::modules::ecs::get_component::<$c1>(&$eid),
                $crate::index::engine::modules::ecs::get_component::<$c2>(&$eid)
            ) {
                let $a1 = &mut comp1;
                let $a2 = &mut comp2;
                $body
                $crate::index::engine::modules::ecs::insert(&$eid, comp1);
                $crate::index::engine::modules::ecs::insert(&$eid, comp2);
            }
        }
    };
}

#[macro_export]
macro_rules! get_query_by_id {
    ($eid:expr, ($c1:ty)) => {
        {
            $crate::index::engine::modules::ecs::get_component::<$c1>(&$eid)
        }
    };
}

#[macro_export]
macro_rules! query_get_all {
    // Single component
    ($c1:ty) => {
        {
            $crate::index::engine::modules::ecs::query_all::<$c1>()
        }
    };
    // Two components
    ($c1:ty, $c2:ty) => {
        {
            $crate::index::engine::modules::ecs::query_all2::<$c1, $c2>()
        }
    };
    // Three components
    ($c1:ty, $c2:ty, $c3:ty) => {
        {
            $crate::index::engine::modules::ecs::query_all3::<$c1, $c2, $c3>()
        }
    };
}

#[macro_export]
macro_rules! query_get_all_ids {
    ($c1:ty) => {
        {
            $crate::index::engine::modules::ecs::query_get_all_ids::<$c1>()
        }
    };
}

#[macro_export]
macro_rules! copy_entity {
    ($source_id:expr) => {
        {
            $crate::index::engine::modules::ecs::copy_entity(&$source_id)
        }
    };
}

#[macro_export]
macro_rules! delete_entity {
    ($entity_id:expr) => {
        {
            $crate::index::engine::modules::ecs::delete_entity(&$entity_id)
        }
    };
}

#[macro_export]
macro_rules! get_all_components_dyn {
    ($entity_id:expr) => {
        {
            $crate::index::engine::modules::ecs::get_all_components(&$entity_id)
        }
    };
}

// ——————————————————————————————————————————————————————————— Serialization Macros ————

/// Save the ECS state directly to JSON using serde_json::to_string_pretty
#[macro_export]
macro_rules! save_world {
    ($path:expr) => {
        {
            use std::fs;
            match $crate::index::engine::modules::ecs::serialize_to_json() {
                Ok(json) => {
                    match fs::write($path, json) {
                        Ok(()) => println!("💾 Saved world to {}", $path),
                        Err(e) => eprintln!("❌ Failed to write file {}: {}", $path, e),
                    }
                }
                Err(e) => eprintln!("❌ Failed to serialize world: {}", e),
            }
        }
    };
}

/// Load the ECS state directly from JSON using serde_json::from_str
#[macro_export]
macro_rules! load_world {
    ($path:expr) => {
        {
            use std::fs;
            match fs::read_to_string($path) {
                Ok(json) => {
                    match $crate::index::engine::modules::ecs::deserialize_from_json(&json) {
                        Ok(()) => {
                            println!("📂 Loaded world from {}", $path);
                            // Update UI if available
                            $crate::index::engine::modules::interface_system::InterfaceSystem::update_entities_list();
                        }
                        Err(e) => eprintln!("❌ Failed to deserialize world: {}", e),
                    }
                }
                Err(e) => eprintln!("❌ Failed to read file {}: {}", $path, e),
            }
        }
    };
}

// ——————————————————————————————————————————————————————————— Simple Functions ————

/// Simple spawn function that calls ECS directly
pub fn spawn() -> EntityId {
    super::ecs::spawn()
}

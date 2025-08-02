use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};

use crate::index::engine::systems::entity_component_system::{ WORLD, World };
use crate::index::engine::components::{ Metadata, Transform, Collider };
use crate::index::engine::components::camera::Camera;
use crate::index::engine::components::static_object3d::StaticObject3D;
use crate::index::engine::components::animated_object3d::AnimatedObject3D;
use crate::index::engine::Component;
use crate::index::PLAYER_ENTITY_ID;

// ================================================================================================
// SERIALIZABLE COMPONENT ENUM
// ================================================================================================

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum SerializableComponent {
    Transform(Transform),
    Metadata(Metadata),
    Camera(Camera),
    StaticObject3D(StaticObject3D),
    AnimatedObject3D(AnimatedObject3D),
    Collider(Collider),
}

// ================================================================================================
// ERROR TYPES
// ================================================================================================

#[derive(Debug)]
pub enum SerializationError {
    FileNotFound(String),
    JsonParseError(serde_json::Error),
    IoError(std::io::Error),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::FileNotFound(path) => write!(f, "File not found: {}", path),
            SerializationError::JsonParseError(err) => write!(f, "JSON parse error: {}", err),
            SerializationError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for SerializationError {}

// ================================================================================================
// SAVE/LOAD FUNCTIONS
// ================================================================================================

/// Save the current ECS state to a JSON file
pub fn try_save_world(path: &str) -> Result<(), SerializationError> {
    let Ok(absolute_path) = std::env::current_dir().map(|p| p.join(path)) else {
        return Err(SerializationError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other, "Failed to get current directory"
        )));
    };

    WORLD.with(|w| {
        let world = w.borrow();
        let mut all_entities_data = Vec::new();

        // Collect all entities and their components
        for (entity_id, _mask) in world.get_all_entities() {
            let mut entity_components = Vec::new();

            // Collect each component type
            if let Some(transform) = world.get_component_readonly::<Transform>(entity_id) {
                entity_components.push(SerializableComponent::Transform(transform.clone()));
            }
            if let Some(metadata) = world.get_component_readonly::<Metadata>(entity_id) {
                entity_components.push(SerializableComponent::Metadata(metadata.clone()));
            }
            if let Some(camera) = world.get_component_readonly::<Camera>(entity_id) {
                entity_components.push(SerializableComponent::Camera(camera.clone()));
            }
            if let Some(static_obj) = world.get_component_readonly::<StaticObject3D>(entity_id) {
                entity_components.push(SerializableComponent::StaticObject3D(static_obj.clone()));
            }
            if let Some(animated_obj) = world.get_component_readonly::<AnimatedObject3D>(entity_id) {
                entity_components.push(SerializableComponent::AnimatedObject3D(animated_obj.clone()));
            }
            if let Some(collider) = world.get_component_readonly::<Collider>(entity_id) {
                entity_components.push(SerializableComponent::Collider(collider.clone()));
            }

            // Only add entities that have components
            if !entity_components.is_empty() {
                all_entities_data.push(entity_components);
            }
        }

        // Serialize to JSON
        let json = match serde_json::to_string_pretty(&all_entities_data) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("❌ JSON serialization failed: {}", err);
                return Err(SerializationError::JsonParseError(err));
            }
        };

        // Write to file
        let mut file = match File::create(&absolute_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("❌ Failed to create file {}: {}", absolute_path.display(), err);
                return Err(SerializationError::IoError(err));
            }
        };
        
        if let Err(err) = file.write_all(json.as_bytes()) {
            eprintln!("❌ Failed to write to file {}: {}", absolute_path.display(), err);
            return Err(SerializationError::IoError(err));
        };

        // Ensure data is written to disk
        if let Err(err) = file.flush() {
            eprintln!("❌ Failed to flush file {}: {}", absolute_path.display(), err);
            return Err(SerializationError::IoError(err));
        };

        println!("💾 Saved {} entities to {}", all_entities_data.len(), path);
        Ok(())
    })
}

/// Load ECS state from a JSON file
pub fn try_load_world(path: &str) -> Result<(), SerializationError> {
    let Ok(absolute_path) = std::env::current_dir().map(|p| p.join(path)) else {
        return Err(SerializationError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other, "Failed to get current directory"
        )));
    };

    WORLD.with(|w| {
        let mut world = w.borrow_mut();

        let data_str = match std::fs::read_to_string(&absolute_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    eprintln!("❌ JSON file is empty: {}", absolute_path.display());
                    return Err(SerializationError::JsonParseError(
                        serde_json::Error::io(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "JSON file is empty"
                        ))
                    ));
                }
                content
            },
            Err(err) => {
                eprintln!("❌ Failed to read file {}: {}", absolute_path.display(), err);
                return Err(SerializationError::FileNotFound(path.to_string()));
            }
        };

        let entities_data: Vec<Vec<SerializableComponent>> = match serde_json::from_str(&data_str) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("❌ JSON parsing failed for {}: {}", absolute_path.display(), err);
                eprintln!("❌ File content preview: {}", &data_str[..std::cmp::min(200, data_str.len())]);
                return Err(SerializationError::JsonParseError(err));
            }
        };

        // Clear current world
        *world = World::default();

        // Store count before consuming entities_data
        let entity_count = entities_data.len();

        // Reconstruct entities
        for component_list in entities_data {
            let entity_id = world.spawn();

            for component in component_list {
                match component {
                    SerializableComponent::Transform(mut transform) => {
                        // Update UI to reflect loaded state
                        transform.update_component_ui(&entity_id);
                        world.insert(&entity_id, transform);
                    }
                    SerializableComponent::Metadata(metadata) => {
                        // Handle role-based global variable restoration
                        if let Some(role) = metadata.role() {
                            match role {
                                "player" => {
                                    *PLAYER_ENTITY_ID.write().unwrap() = Some(entity_id.clone());
                                    println!("🔄 Auto-restored PLAYER_ENTITY_ID: {}", entity_id);
                                }
                                _ => {}
                            }
                        }
                        world.insert(&entity_id, metadata);
                    }
                    SerializableComponent::Camera(mut camera) => {
                        // Update UI to reflect loaded state
                        camera.update_component_ui(&entity_id);
                        world.insert(&entity_id, camera);
                    }
                    SerializableComponent::StaticObject3D(mut static_obj) => {
                        // OpenGL resources and UI are already properly initialized by custom deserialize
                        static_obj.update_component_ui(&entity_id);
                        world.insert(&entity_id, static_obj);
                    }
                    SerializableComponent::AnimatedObject3D(mut animated_obj) => {
                        // OpenGL resources and UI are already properly initialized by custom deserialize
                        animated_obj.update_component_ui(&entity_id);
                        world.insert(&entity_id, animated_obj);
                    }
                    SerializableComponent::Collider(mut collider) => {
                        // Update UI to reflect loaded state
                        collider.update_component_ui(&entity_id);
                        world.insert(&entity_id, collider);
                    }
                }
            }
        }

        println!("📂 Loaded {} entities from {}", entity_count, path);
        Ok(())
    })
}

// ================================================================================================
// MACROS
// ================================================================================================

/// Save the current ECS state to a JSON file
#[macro_export]
macro_rules! save_world {
    ($path:expr) => {
        {
        use crate::index::engine::systems::serialization::try_save_world;
        
        match try_save_world($path) {
            Ok(()) => {
                println!("✅ ECS state saved to: {}", $path);
            }
            Err(err) => {
                eprintln!("❌ Failed to save ECS state: {}", err);
            }
        }
        }
    };
}

/// Load ECS state from a JSON file
#[macro_export]
macro_rules! load_world {
    ($path:expr) => {
        {
        use crate::index::engine::systems::serialization::try_load_world;
        
        match try_load_world($path) {
            Ok(()) => {
                println!("✅ ECS state loaded from: {}", $path);
            }
            Err(err) => {
                eprintln!("❌ Failed to load ECS state: {}", err);
            }
        }
        }
    };
}

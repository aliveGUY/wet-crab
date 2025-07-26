use std::collections::HashMap;
use std::any::TypeId;
use std::fs::File;
use std::io::Write;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use slint::{VecModel, ModelRc, Model};

use crate::{ComponentUI, AttributeUI};
use crate::index::engine::systems::entity_component_system::{WORLD, World, Insertable, Component};
use crate::index::engine::components::{Metadata, Transform};
use crate::index::engine::components::camera::Camera;

// ================================================================================================
// SERIALIZABLE TYPES
// ================================================================================================

/// Serializable version of ComponentUI for JSON storage
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableComponentUI {
    pub name: String,
    pub attributes: Vec<SerializableAttributeUI>,
}

/// Serializable version of AttributeUI for JSON storage
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableAttributeUI {
    pub name: String,
    pub value: String,
    pub dt_type: String,
}

// ================================================================================================
// CONVERSION TRAITS
// ================================================================================================

impl From<&ComponentUI> for SerializableComponentUI {
    fn from(ui: &ComponentUI) -> Self {
        let attributes = (0..ui.attributes.row_count())
            .filter_map(|i| ui.attributes.row_data(i))
            .map(|attr| SerializableAttributeUI {
                name: attr.name.to_string(),
                value: attr.value.to_string(),
                dt_type: attr.dt_type.to_string(),
            })
            .collect();

        Self {
            name: ui.name.to_string(),
            attributes,
        }
    }
}

impl From<SerializableComponentUI> for ComponentUI {
    fn from(s: SerializableComponentUI) -> Self {
        let attrs = s.attributes.into_iter().map(|a| AttributeUI {
            name: a.name.into(),
            value: a.value.into(),
            dt_type: a.dt_type.into(),
        }).collect::<Vec<_>>();

        ComponentUI {
            name: s.name.into(),
            attributes: ModelRc::new(VecModel::from(attrs)),
        }
    }
}

// ================================================================================================
// ERROR TYPES
// ================================================================================================

#[derive(Debug)]
pub enum SerializationError {
    FileNotFound(String),
    JsonParseError(serde_json::Error),
    UnknownComponent(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::FileNotFound(path) => write!(f, "File not found: {}", path),
            SerializationError::JsonParseError(err) => write!(f, "JSON parse error: {}", err),
            SerializationError::UnknownComponent(name) => write!(f, "Unknown component type: {}", name),
            SerializationError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for SerializationError {}

// ================================================================================================
// COMPONENT CONSTRUCTION TRAIT
// ================================================================================================

/// Trait for components that can be constructed from ComponentUI
pub trait FromComponentUI: Component {
    fn from_ui(ui: &ComponentUI) -> Self;
}

// ================================================================================================
// COMPONENT LOADER FUNCTIONS
// ================================================================================================

fn load_metadata(ui: &ComponentUI) -> Box<dyn Insertable> {
    Box::new(Metadata::from_ui(ui))
}

fn load_transform(ui: &ComponentUI) -> Box<dyn Insertable> {
    Box::new(Transform::from_ui(ui))
}

fn load_camera(ui: &ComponentUI) -> Box<dyn Insertable> {
    Box::new(Camera::from_ui(ui))
}

// ================================================================================================
// COMPONENT LOADER REGISTRY
// ================================================================================================

/// Function type for component loaders
pub type ComponentLoaderFn = fn(&ComponentUI) -> Box<dyn Insertable>;

/// Global registry mapping component names to their TypeId and constructor functions
pub static COMPONENT_LOADERS: Lazy<HashMap<String, (TypeId, ComponentLoaderFn)>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Register basic component types using function pointers
    m.insert("Metadata".into(), (TypeId::of::<Metadata>(), load_metadata as ComponentLoaderFn));
    m.insert("Transform".into(), (TypeId::of::<Transform>(), load_transform as ComponentLoaderFn));
    m.insert("Camera".into(), (TypeId::of::<Camera>(), load_camera as ComponentLoaderFn));

    m
});

// ================================================================================================
// COMPONENT IMPLEMENTATIONS
// ================================================================================================

impl FromComponentUI for Metadata {
    fn from_ui(ui: &ComponentUI) -> Self {
        let mut metadata = Metadata::new("Default");
        metadata.apply_ui(ui);
        metadata
    }
}

impl FromComponentUI for Transform {
    fn from_ui(ui: &ComponentUI) -> Self {
        let mut transform = Transform::new(0.0, 0.0, 0.0);
        transform.apply_ui(ui);
        transform
    }
}

impl FromComponentUI for Camera {
    fn from_ui(ui: &ComponentUI) -> Self {
        let mut camera = Camera::new();
        camera.apply_ui(ui);
        camera
    }
}

// ================================================================================================
// SAVE/LOAD FUNCTIONS
// ================================================================================================

/// Save the current ECS state to a JSON file
pub fn try_save_world(path: &str) -> Result<(), SerializationError> {
    // Normalize path relative to current directory
    let absolute_path = std::env::current_dir()
        .map_err(SerializationError::IoError)?
        .join(path);

    WORLD.with(|w| {
        let world = w.borrow();
        let mut all_entities_data = Vec::new();

        // Collect all entities' component UI data
        for (entity_id, _mask) in world.get_all_entities() {
            let ui_list = world.get_all_components_ui_for_entity(entity_id);
            
            // Convert Slint ComponentUI to serializable format
            let serializable_components: Vec<SerializableComponentUI> = ui_list
                .into_iter()
                .map(|comp_ui| SerializableComponentUI::from(&comp_ui))
                .collect();
            
            all_entities_data.push(serializable_components);
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&all_entities_data)
            .map_err(SerializationError::JsonParseError)?;

        // Write to file
        let mut file = File::create(&absolute_path)
            .map_err(SerializationError::IoError)?;
        file.write_all(json.as_bytes())
            .map_err(SerializationError::IoError)?;

        Ok(())
    })
}

/// Load ECS state from a JSON file
pub fn try_load_world(path: &str) -> Result<(), SerializationError> {
    // Normalize path relative to current directory
    let absolute_path = std::env::current_dir()
        .map_err(SerializationError::IoError)?
        .join(path);

    WORLD.with(|w| {
        let mut world = w.borrow_mut();

        // Read and parse JSON
        let data_str = std::fs::read_to_string(&absolute_path)
            .map_err(|_| SerializationError::FileNotFound(path.to_string()))?;

        let entities_data: Vec<Vec<SerializableComponentUI>> = 
            serde_json::from_str(&data_str)
            .map_err(SerializationError::JsonParseError)?;

        // Clear current world
        *world = World::default();

        // Reconstruct entities
        for comp_list in entities_data {
            let entity_id = world.spawn();
            
            // Create components vector for batch insertion
            let mut components: Vec<Box<dyn Insertable>> = Vec::new();
            
            for serializable_comp in comp_list {
                // Convert back to Slint ComponentUI
                let component_ui = ComponentUI::from(serializable_comp.clone());
                
                // Use component loader registry to create component
                if let Some((_type_id, loader_fn)) = COMPONENT_LOADERS.get(&serializable_comp.name) {
                    let component = loader_fn(&component_ui);
                    components.push(component);
                } else {
                    return Err(SerializationError::UnknownComponent(serializable_comp.name));
                }
            }
            
            // Insert all components for this entity
            world.insert_dyn(&entity_id, components);
        }

        Ok(())
    })
}

// ================================================================================================
// MACROS
// ================================================================================================

/// Save the current ECS state to a JSON file
#[macro_export]
macro_rules! save_world {
    ($path:expr) => {{
        use crate::index::engine::systems::serialization::try_save_world;
        
        match try_save_world($path) {
            Ok(()) => {
                println!("✅ ECS state saved to: {}", $path);
            }
            Err(err) => {
                eprintln!("❌ Failed to save ECS state: {}", err);
            }
        }
    }};
}

/// Load ECS state from a JSON file
#[macro_export]
macro_rules! load_world {
    ($path:expr) => {{
        use crate::index::engine::systems::serialization::try_load_world;
        
        match try_load_world($path) {
            Ok(()) => {
                println!("✅ ECS state loaded from: {}", $path);
            }
            Err(err) => {
                eprintln!("❌ Failed to load ECS state: {}", err);
            }
        }
    }};
}

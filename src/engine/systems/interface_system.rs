use crate::index::engine::systems::entity_component_system::WORLD;
use crate::index::engine::components::{Metadata, Transform, Collider};
use crate::index::engine::components::camera::Camera;
use crate::index::engine::components::static_object3d::StaticObject3D;
use crate::{
    copy_entity, delete_entity, LevelEditorUI, InterfaceState
};
use crate::{query_get_all};
use slint::{VecModel, ModelRc, ComponentHandle};

pub struct InterfaceSystem {
    ui: LevelEditorUI,
}

impl InterfaceSystem {
    pub fn new() -> Self {
        let ui = LevelEditorUI::new().unwrap();
        
        // Set up callbacks
        let state = ui.global::<InterfaceState>();
        
        // Entity selection callback
        let ui_weak = ui.as_weak();
        state.on_entity_selected({
            move |entity_id| {
                println!("🎯 Entity selected: {}", entity_id);
                
                let (title, components_json) = WORLD.with(|w| {
                    let world = w.borrow();
                    
                    // Get entity metadata for title
                    let title = world.get_component_readonly::<Metadata>(&entity_id.to_string())
                        .map(|metadata| metadata.title().to_string())
                        .unwrap_or_else(|| "Unknown Entity".to_string());
                    
                    // Collect all components for this entity and serialize them
                    let mut components = Vec::new();
                    let entity_id_str = entity_id.to_string();
                    
                    // Check for Transform component
                    if let Some(transform) = world.get_component_readonly::<Transform>(&entity_id_str) {
                        if let Ok(data) = serde_json::to_value(transform) {
                            components.push(serde_json::json!({
                                "type": "Transform",
                                "data": data
                            }));
                        }
                    }
                    
                    // Check for Metadata component
                    if let Some(metadata) = world.get_component_readonly::<Metadata>(&entity_id_str) {
                        if let Ok(data) = serde_json::to_value(metadata) {
                            components.push(serde_json::json!({
                                "type": "Metadata", 
                                "data": data
                            }));
                        }
                    }
                    
                    // Check for Camera component
                    if let Some(camera) = world.get_component_readonly::<Camera>(&entity_id_str) {
                        if let Ok(data) = serde_json::to_value(camera) {
                            components.push(serde_json::json!({
                                "type": "Camera",
                                "data": data
                            }));
                        }
                    }
                    
                    // Check for Collider component
                    if let Some(collider) = world.get_component_readonly::<Collider>(&entity_id_str) {
                        if let Ok(data) = serde_json::to_value(collider) {
                            components.push(serde_json::json!({
                                "type": "Collider",
                                "data": data
                            }));
                        }
                    }
                    
                    // Check for StaticObject3D component
                    if let Some(static_obj) = world.get_component_readonly::<StaticObject3D>(&entity_id_str) {
                        if let Ok(data) = serde_json::to_value(static_obj) {
                            components.push(serde_json::json!({
                                "type": "StaticObject3D",
                                "data": data
                            }));
                        }
                    }
                    
                    // Serialize all components as JSON array
                    let components_json = serde_json::to_string_pretty(&components)
                        .unwrap_or_else(|_| "[]".to_string());
                    
                    (title, components_json)
                });
                
                // Update UI state
                if let Some(ui) = ui_weak.upgrade() {
                    let state = ui.global::<InterfaceState>();
                    state.set_selected_index(entity_id);
                    state.set_selected_title(title.into());
                    state.set_components_json(components_json.into());
                }
            }
        });

        // Component change callback - simplified to just handle JSON
        state.on_component_changed({
            move |entity_id, component_json| {
                println!("🔧 Component changed for entity {}: {}", entity_id, component_json);
                // TODO: Parse JSON and update component data
            }
        });

        // Entity deselection callback
        state.on_entity_deselected({
            move || {
                println!("🎯 Entity deselected");
            }
        });

        // Copy entity callback
        state.on_copy_entity({
            move |entity_id| {
                println!("📋 Copying entity: {}", entity_id);
                if let Some(new_entity_id) = copy_entity!(entity_id.to_string()) {
                    println!("✅ Entity copied: {} -> {}", entity_id, new_entity_id);
                } else {
                    println!("❌ Failed to copy entity: {}", entity_id);
                }
            }
        });

        // Delete entity callback
        state.on_delete_entity({
            move |entity_id| {
                println!("🗑️ Deleting entity: {}", entity_id);
                if delete_entity!(entity_id.to_string()) {
                    println!("✅ Entity deleted: {}", entity_id);
                } else {
                    println!("❌ Failed to delete entity: {}", entity_id);
                }
            }
        });

        // Save scene callback
        state.on_save_scene({
            move || {
                println!("💾 Saving scene...");
                crate::save_world!("src/assets/scenes/test_world.json");
            }
        });

        // Spawn blockout platform callback
        state.on_spawn_blockout_platform({
            move || {
                println!("🏗️ Spawning blockout platform...");
                crate::index::game::entities::spawn_blockout_platform();
            }
        });

        Self { ui }
    }

    pub fn update(&self) {
        // Update entity list
        let metadata_results = query_get_all!(Metadata);
        let mut entities = Vec::new();
        
        for (entity_id, metadata) in metadata_results {
            // For now, create a simple tuple - we'll need to define a proper struct later
            entities.push((entity_id.into(), metadata.title().into()));
        }
        
        let entities_model = VecModel::from(entities);
        let state = self.ui.global::<InterfaceState>();
        state.set_entities(ModelRc::new(entities_model).into());
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        self.ui.show()
    }

    pub fn hide(&self) -> Result<(), slint::PlatformError> {
        self.ui.hide()
    }

    // Add missing static methods that other parts of the codebase expect
    pub fn get_selection_state() -> (String, String) {
        // Return empty selection state for now
        ("".to_string(), "".to_string())
    }

    pub fn update_entity_tree_global() {
        // Static method for updating entity tree - placeholder for now
        println!("🔄 Updating entity tree globally");
    }

    pub fn initialize(_ui_app: &LevelEditorUI) {
        // Static initialization method - placeholder for now
        println!("🚀 Initializing interface system");
    }
}

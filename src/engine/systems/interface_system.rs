use crate::index::engine::components::{ Metadata };
use crate::{ copy_entity, delete_entity, LevelEditorUI, InterfaceState };
use crate::Entity; // Import the generated Slint Entity struct
use crate::{ query_get_all, get_all_components_dyn };
use slint::{ VecModel, ModelRc, ComponentHandle, Weak };
use std::sync::{ Mutex, OnceLock };
use serde_json::to_string;

static INTERFACE_SYSTEM: OnceLock<Mutex<InterfaceSystem>> = OnceLock::new();

pub struct InterfaceSystem {
    ui_weak: Weak<LevelEditorUI>,
}

impl InterfaceSystem {
    /// Initialize the singleton InterfaceSystem with UI reference
    pub fn initialize(ui_weak: Weak<LevelEditorUI>) {
        let instance = Self::new_with_ui(ui_weak);
        if INTERFACE_SYSTEM.set(Mutex::new(instance)).is_err() {
            panic!("InterfaceSystem should only be initialized once");
        }

        // Initial entity list update
        Self::update_entities_list();
    }

    /// Update the entity list in the UI (call this when ECS changes)
    pub fn update_entities_list() {
        if let Some(system) = INTERFACE_SYSTEM.get() {
            if let Ok(system) = system.lock() {
                system.update_entities_internal();
            }
        }
    }

    /// Get the current selection state (for render system compatibility)
    pub fn get_selection_state() -> (String, String) {
        if let Some(system) = INTERFACE_SYSTEM.get() {
            if let Ok(system) = system.lock() {
                if let Some(ui) = system.ui_weak.upgrade() {
                    let state = ui.global::<InterfaceState>();
                    return (
                        state.get_selected_index().to_string(),
                        state.get_hovered_entity_id().to_string(),
                    );
                }
            }
        }
        ("".to_string(), "".to_string())
    }

    /// Private constructor for singleton
    fn new_with_ui(ui_weak: Weak<LevelEditorUI>) -> Self {
        let ui = ui_weak.upgrade().expect("UI should be available during initialization");

        // Set up callbacks
        let state = ui.global::<InterfaceState>();

        // Entity selection callback
        state.on_entity_selected({
            move |entity_id| {
                let entity_id_string = entity_id.to_string();
                let components = get_all_components_dyn!(entity_id_string);

                let mut ser_components: Vec<String> = Vec::new();
                for component in components {
                    let result = to_string(&component);

                    if result.is_ok() {
                        ser_components.push(result.unwrap());
                    }
                }

                println!("{:#?}", ser_components)
            }
        });

        // Component change callback - handle component updates using existing deserialization
        state.on_component_changed({
            move |entity_id, component_json| {
                println!("🔧 Component changed for entity {}: {}", entity_id, component_json);
                // Self::update_component_from_json(entity_id.to_string(), component_json.to_string());
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
                InterfaceSystem::update_entities_list();
            }
        });

        Self { ui_weak: ui.as_weak() }
    }

    fn update_entities_internal(&self) {
        // Update entity list from ECS
        let metadata_results = query_get_all!(Metadata);

        println!("🔄 Updating entity list - found {} entities", metadata_results.len());

        // Create entities list for Slint with proper Entity struct format
        let mut entities = Vec::new();
        for (entity_id, metadata) in metadata_results {
            println!("  - Entity: {} - {}", entity_id, metadata.title());
            // Create proper Entity struct that matches the Slint definition
            entities.push(Entity {
                entity_id: entity_id.into(),
                title: metadata.title().into(),
            });
        }

        // Get the UI instance and update entities
        if let Some(ui) = self.ui_weak.upgrade() {
            let entities_model = VecModel::from(entities);
            let state = ui.global::<InterfaceState>();
            state.set_entities(ModelRc::new(entities_model).into());
            println!("✅ Entity list updated successfully");
        } else {
            println!("❌ UI instance not available for entity update");
        }
    }
}

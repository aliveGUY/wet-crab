use crate::index::engine::components::{ Metadata };
use crate::{ copy_entity, delete_entity, LevelEditorUI, InterfaceState };
use crate::Entity; // Import the generated Slint Entity struct
use crate::{ query_get_all, get_all_components_dyn };
use crate::{KeyValuePair, ComponentData}; // Import KeyValuePair and ComponentData from Slint
use slint::{ VecModel, ModelRc, ComponentHandle, Weak };
use std::sync::{ Mutex, OnceLock };
use serde_json::{ to_string, Value };

static INTERFACE_SYSTEM: OnceLock<Mutex<InterfaceSystem>> = OnceLock::new();

pub struct InterfaceSystem {
    ui_weak: Weak<LevelEditorUI>,
}

impl InterfaceSystem {
    /// Parse any JSON object into flat key-value pairs
    fn parse_json_to_key_value_pairs(json_str: &str) -> Vec<KeyValuePair> {
        let mut pairs = Vec::new();
        
        if let Ok(value) = serde_json::from_str::<Value>(json_str) {
            if let Value::Object(map) = value {
                for (key, val) in map {
                    let value_str = match val {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        Value::Array(_) | Value::Object(_) => val.to_string(),
                    };
                    
                    pairs.push(KeyValuePair {
                        key: key.into(),
                        value: value_str.into(),
                    });
                }
            }
        }
        
        pairs
    }

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
            let ui_weak_clone = ui.as_weak();
            move |entity_id| {
                let entity_id_string = entity_id.to_string();
                let components = get_all_components_dyn!(entity_id_string);

                println!("Entity clicked: {}", entity_id_string);

                // Parse each component into separate ComponentData objects
                let mut parsed_components = Vec::new();
                for component in components {
                    if let Ok(json_str) = to_string(&component) {
                        println!("{}", json_str);
                        let parsed_pairs = Self::parse_json_to_key_value_pairs(&json_str);
                        
                        // Extract component type from the parsed pairs
                        let component_type = parsed_pairs
                            .iter()
                            .find(|pair| pair.key.as_str() == "type")
                            .map(|pair| pair.value.as_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // Filter out the "type" field since it's used as the title
                        let filtered_pairs: Vec<KeyValuePair> = parsed_pairs
                            .into_iter()
                            .filter(|pair| pair.key.as_str() != "type")
                            .collect();

                        // Create ComponentData object
                        let fields_model = VecModel::from(filtered_pairs);
                        parsed_components.push(ComponentData {
                            component_type: component_type.into(),
                            data_json: json_str.into(),
                            fields: ModelRc::new(fields_model).into(),
                        });
                    }
                }

                // Update the UI with parsed component data
                if let Some(ui) = ui_weak_clone.upgrade() {
                    let state = ui.global::<InterfaceState>();
                    let components_model = VecModel::from(parsed_components);
                    state.set_parsed_components(ModelRc::new(components_model).into());
                }
            }
        });

        // Component change callback - handle component updates using existing deserialization
        state.on_component_changed({
            move |entity_id, component_json| {
                println!("🔧 Component changed for entity {}: {}", entity_id, component_json);
                Self::update_component_from_json(entity_id.to_string(), component_json.to_string());
            }
        });

        // Update component field callback - handle individual field updates
        state.on_update_component_field({
            move |entity_id, component_type, field_key, new_value| {
                println!("🔧 Field update: entity={}, component={}, field={}, value={}", 
                    entity_id, component_type, field_key, new_value);
                
                // Update the component field and reconstruct the component
                Self::update_component_field_internal(
                    entity_id.to_string(), 
                    component_type.to_string(), 
                    field_key.to_string(), 
                    new_value.to_string()
                );
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
                    InterfaceSystem::update_entities_list();
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
                    InterfaceSystem::update_entities_list();
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

    /// Update a specific field in a component and refresh the UI
    fn update_component_field_internal(
        entity_id: String, 
        component_type: String, 
        field_key: String, 
        new_value: String
    ) {
        println!("🔧 Updating component field: entity={}, component={}, field={}, value={}", 
            entity_id, component_type, field_key, new_value);

        // Get the current component data
        let components = get_all_components_dyn!(entity_id);
        
        for component in components {
            if let Ok(json_str) = to_string(&component) {
                // Parse the component JSON to check if it matches the component type
                if let Ok(mut json_value) = serde_json::from_str::<Value>(&json_str) {
                    if let Some(Value::String(comp_type)) = json_value.get("type") {
                        if comp_type == &component_type {
                            // Found the matching component, update the field
                            if let Some(obj) = json_value.as_object_mut() {
                                // Parse the new value appropriately
                                let parsed_value = Self::parse_field_value(&new_value);
                                obj.insert(field_key.clone(), parsed_value);
                                
                                // Convert back to JSON string
                                if let Ok(updated_json) = serde_json::to_string(&json_value) {
                                    println!("📝 Updated component JSON: {}", updated_json);
                                    
                                    // Update the ECS component using existing system
                                    Self::update_ecs_component(&entity_id, &updated_json);
                                    
                                    // Refresh the UI to show the updated component
                                    Self::refresh_selected_entity(&entity_id);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        println!("❌ Failed to find component {} for entity {}", component_type, entity_id);
    }

    /// Parse a string value into the appropriate JSON value type
    fn parse_field_value(value_str: &str) -> Value {
        // Try to parse as number first
        if let Ok(int_val) = value_str.parse::<i64>() {
            return Value::Number(serde_json::Number::from(int_val));
        }
        
        if let Ok(float_val) = value_str.parse::<f64>() {
            return Value::Number(serde_json::Number::from_f64(float_val).unwrap_or(serde_json::Number::from(0)));
        }
        
        // Try to parse as boolean
        if let Ok(bool_val) = value_str.parse::<bool>() {
            return Value::Bool(bool_val);
        }
        
        // Default to string
        Value::String(value_str.to_string())
    }

    /// Update the ECS component using existing deserialization system
    fn update_ecs_component(entity_id: &str, component_json: &str) {
        println!("🔄 Updating ECS component for entity {}", entity_id);
        println!("📝 Component JSON: {}", component_json);
        
        // Use the existing component-changed callback to handle deserialization
        // This leverages whatever deserialization system is already in place
        if let Some(system) = INTERFACE_SYSTEM.get() {
            if let Ok(system) = system.lock() {
                if let Some(ui) = system.ui_weak.upgrade() {
                    let state = ui.global::<InterfaceState>();
                    let entity_id_slint: slint::SharedString = entity_id.into();
                    let component_json_slint: slint::SharedString = component_json.into();
                    
                    // Dispatch to existing component-changed system
                    state.invoke_component_changed(entity_id_slint, component_json_slint);
                    println!("✅ Component update dispatched to existing system");
                }
            }
        }
    }

    /// Refresh the UI for the currently selected entity
    fn refresh_selected_entity(entity_id: &str) {
        if let Some(system) = INTERFACE_SYSTEM.get() {
            if let Ok(system) = system.lock() {
                if let Some(ui) = system.ui_weak.upgrade() {
                    let state = ui.global::<InterfaceState>();
                    
                    // Check if this entity is currently selected
                    if state.get_selected_index().to_string() == entity_id {
                        // Re-trigger entity selection to refresh the component display
                        let entity_id_slint: slint::SharedString = entity_id.into();
                        state.invoke_entity_selected(entity_id_slint);
                    }
                }
            }
        }
    }

    /// Update component from JSON using generic deserialization
    fn update_component_from_json(entity_id: String, component_json: String) {
        println!("🔄 Deserializing component JSON for entity {}", entity_id);
        
        // Use the generic Component enum deserialization - leverages existing serde type tagging
        match serde_json::from_str::<crate::index::engine::systems::ecs::Component>(&component_json) {
            Ok(component) => {
                crate::index::engine::systems::ecs::insert(&entity_id, component);
                println!("✅ Component updated successfully using generic deserialization");
            },
            Err(e) => {
                println!("❌ Failed to deserialize component: {}", e);
            }
        }
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

use once_cell::sync::Lazy;
use std::sync::Mutex;
use slint::{ ComponentHandle, Weak, SharedString, VecModel, ModelRc, Model };
use crate::{ InterfaceState, LevelEditorUI, ComponentUI, AttributeUI };
use crate::{ query_get_all, get_all_components_by_id, query_by_id };
use crate::index::engine::components::{ Metadata, Transform };
use crate::index::engine::components::camera::Camera;
use crate::index::engine::components::static_object3d::StaticObject3D;
use crate::index::engine::components::animated_object3d::AnimatedObject3D;
use crate::index::engine::Component;

pub struct InterfaceSystem {
    ui_handle: Weak<LevelEditorUI>,
    selected_entity_id: Option<String>,
}

// Global instance for backward compatibility with existing code
static INTERFACE_SYSTEM: Lazy<Mutex<Option<InterfaceSystem>>> = Lazy::new(|| Mutex::new(None));

impl InterfaceSystem {
    /// Create a new InterfaceSystem instance
    pub fn new(ui_context: &LevelEditorUI) -> Self {
        let ui_handle = ui_context.as_weak();

        let system = Self {
            ui_handle,
            selected_entity_id: None,
        };

        // Set up callbacks
        let state = ui_context.global::<InterfaceState>();

        // Set up attribute change callback
        state.on_attribute_changed({
            move |entity_id, component_name, attribute_name, new_value| {
                Self::handle_attribute_change(
                    entity_id.to_string(),
                    component_name.to_string(),
                    attribute_name.to_string(),
                    new_value.to_string()
                );
            }
        });

        // Set up entity selection callback
        state.on_entity_selected({
            move |entity_id| {
                Self::handle_entity_selected(entity_id.to_string());
            }
        });

        // Set up entity deselection callback
        state.on_entity_deselected({
            move || {
                Self::handle_entity_deselected();
            }
        });

        system
    }

    /// Handle entity selection - load and populate components
    pub fn handle_entity_selected(entity_id: String) {
        println!("🎯 Entity selected: {}", entity_id);

        // Load components for this entity
        let components_ui = get_all_components_by_id!(entity_id);
        Self::populate_components_ui(components_ui);
    }

    /// Handle entity deselection - clear components
    pub fn handle_entity_deselected() {
        println!("❌ Entity deselected - clearing components");

        // Clear components UI
        Self::clear_components_ui();
    }

    /// Populate components in UI
    fn populate_components_ui(components: Vec<ComponentUI>) {
        if let Some(ref system) = INTERFACE_SYSTEM.lock().unwrap().as_ref() {
            system.update_ui_components(components);
        }
    }

    /// Clear components from UI
    fn clear_components_ui() {
        if let Some(ref system) = INTERFACE_SYSTEM.lock().unwrap().as_ref() {
            system.update_ui_components(Vec::new()); // Empty components list
        }
    }

    /// Update the UI with component data - direct pass-through, no conversion needed
    fn update_ui_components(&self, components: Vec<ComponentUI>) {
        let ui = match self.ui_handle.upgrade() {
            Some(ui) => ui,
            None => {
                return;
            }
        };

        // No conversion needed - components already return Slint ComponentUI
        let state = ui.global::<InterfaceState>();
        let components_model = VecModel::from(components);
        state.set_components(ModelRc::new(components_model).into());
    }

    /// Update the entity tree
    pub fn update_entity_tree(&self) {
        let ui = match self.ui_handle.upgrade() {
            Some(ui) => ui,
            None => {
                return;
            }
        };

        let state = ui.global::<InterfaceState>();
        let all_entities_with_metadata = query_get_all!(Metadata);

        let entities_model: VecModel<(SharedString, SharedString)> = VecModel::default();

        for (entity_id, metadata) in all_entities_with_metadata {
            let entity_data = (SharedString::from(entity_id), SharedString::from(metadata.title()));
            entities_model.push(entity_data);
        }

        let entity_count = entities_model.row_count();
        state.set_entities(ModelRc::new(entities_model).into());

        println!("Updated entity tree with {} entities", entity_count);
    }

    /// Set the selected entity
    pub fn set_selected_entity(&mut self, entity_id: Option<String>) {
        self.selected_entity_id = entity_id.clone();

        let ui = match self.ui_handle.upgrade() {
            Some(ui) => ui,
            None => {
                return;
            }
        };

        let state = ui.global::<InterfaceState>();
        let selection = entity_id.unwrap_or_default();
        state.set_selected_index(SharedString::from(selection));
    }

    /// Get the currently selected entity ID
    pub fn get_selected_entity(&self) -> Option<&String> {
        self.selected_entity_id.as_ref()
    }

    /// Handle attribute changes from UI - update ECS components immediately
    pub fn handle_attribute_change(
        entity_id: String,
        component_name: String,
        attribute_name: String,
        new_value: String
    ) {
        println!(
            "🔄 Attribute changed: Entity={}, Component={}, Attribute={}, Value={}",
            entity_id,
            component_name,
            attribute_name,
            new_value
        );

        // Create a ComponentUI with the single changed attribute to apply to ECS
        let changed_attribute = AttributeUI {
            name: attribute_name.into(),
            value: new_value.into(),
            dt_type: "STRING".into(), // We'll let the component handle type conversion
        };

        let component_ui = ComponentUI {
            name: component_name.clone().into(),
            attributes: ModelRc::new(VecModel::from(vec![changed_attribute])),
        };

        // Apply the change to the appropriate component type in ECS
        match component_name.as_str() {
            "Metadata" => {
                query_by_id!(entity_id, (Metadata), |metadata| {
                    metadata.apply_ui(&component_ui);
                });
            }
            "Transform" => {
                query_by_id!(entity_id, (Transform), |transform| {
                    transform.apply_ui(&component_ui);
                });
            }
            "Camera" => {
                query_by_id!(entity_id, (Camera), |camera| {
                    camera.apply_ui(&component_ui);
                });
            }
            "Static Object 3D" => {
                query_by_id!(entity_id, (StaticObject3D), |static_obj| {
                    static_obj.apply_ui(&component_ui);
                });
            }
            "Animated Object 3D" => {
                query_by_id!(entity_id, (AnimatedObject3D), |animated_obj| {
                    animated_obj.apply_ui(&component_ui);
                });
            }
            _ => {
                println!("⚠️ Unknown component type: {}", component_name);
            }
        }
    }

    // Static methods for backward compatibility with existing code

    /// Initialize the global InterfaceSystem instance (replaces old initialize method)
    pub fn initialize(ui_context: &LevelEditorUI) {
        let system = Self::new(ui_context);
        *INTERFACE_SYSTEM.lock().unwrap() = Some(system);
    }

    /// Update the entity tree using the global instance (static method)
    pub fn update_entity_tree_global() {
        if let Some(ref system) = INTERFACE_SYSTEM.lock().unwrap().as_ref() {
            system.update_entity_tree();
        }
    }

    /// Set the selected entity using the global instance
    pub fn set_selected_element(entity_id: &str) {
        if let Some(ref mut system) = INTERFACE_SYSTEM.lock().unwrap().as_mut() {
            system.set_selected_entity(Some(entity_id.to_string()));
        }
    }
}

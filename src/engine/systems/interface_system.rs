use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;
use std::any::TypeId;
use slint::{ ComponentHandle, Weak, SharedString, VecModel, ModelRc, Model };
use crate::{ InterfaceState, LevelEditorUI, ComponentUI };
use crate::{ query_get_all, get_all_components_by_id };
use crate::index::engine::components::{ Metadata, Transform };
use crate::index::engine::components::camera::Camera;
use crate::index::engine::components::static_object3d::StaticObject3D;
use crate::index::engine::components::animated_object3d::AnimatedObject3D;
use crate::index::engine::systems::entity_component_system::WORLD;

pub struct InterfaceSystem {
    ui_handle: Weak<LevelEditorUI>,
    selected_entity_id: Option<String>,
}

// Component type mapping for dynamic lookup
static COMPONENT_TYPE_MAP: Lazy<HashMap<&'static str, TypeId>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("Metadata", TypeId::of::<Metadata>());
    m.insert("Transform", TypeId::of::<Transform>());
    m.insert("Camera", TypeId::of::<Camera>());
    m.insert("Static Object 3D", TypeId::of::<StaticObject3D>());
    m.insert("Animated Object 3D", TypeId::of::<AnimatedObject3D>());
    m
});

static INTERFACE_SYSTEM: Lazy<Mutex<Option<InterfaceSystem>>> = Lazy::new(|| Mutex::new(None));

impl InterfaceSystem {
    /// Handle component changes from UI - now uses dynamic lookup instead of hardcoded match!
    pub fn handle_component_change(
        entity_id: String,
        component_name: String,
        updated_component: ComponentUI
    ) {
        println!(
            "🔄 Component changed: Entity={}, Component={}, Attributes={}",
            entity_id,
            component_name,
            updated_component.attributes.row_count()
        );

        // Dynamic component lookup using TypeId and StoreDyn - no more hardcoded match!
        if let Some(&type_id) = COMPONENT_TYPE_MAP.get(component_name.as_str()) {
            WORLD.with(|w| {
                let mut world = w.borrow_mut();
                if world.apply_component_ui_by_type(&entity_id, &type_id, &updated_component) {
                    println!("✅ Applied UI changes to {} component for entity {}", component_name, entity_id);
                } else {
                    println!("⚠️ No store found for component type: {}", component_name);
                }
            });
        } else {
            println!("⚠️ Unknown component type: {}", component_name);
        }
    }

    /// Create a new InterfaceSystem instance
    pub fn new(ui_context: &LevelEditorUI) -> Self {
        let ui_handle = ui_context.as_weak();

        let system = Self {
            ui_handle,
            selected_entity_id: None,
        };

        // Set up callbacks
        let state = ui_context.global::<InterfaceState>();

        // Set up component change callback
        state.on_component_changed({
            move |entity_id, component_name, updated_component| {
                Self::handle_component_change(
                    entity_id.to_string(),
                    component_name.to_string(),
                    updated_component
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

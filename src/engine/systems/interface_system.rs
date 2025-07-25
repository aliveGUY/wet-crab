use once_cell::sync::OnceCell;
use std::sync::Mutex;
use slint::{ ComponentHandle, Weak, SharedString, VecModel, ModelRc, Model };
use crate::{
    InterfaceState,
    LevelEditorUI,
    ComponentUI as SlintComponentUI,
    AttributeUI as SlintAttributeUI,
};
use crate::{ query_get_all, get_all_components_by_id };
use crate::index::engine::components::Metadata;
use crate::index::engine::systems::entity_component_system::ComponentUI;

static UI_HANDLE: OnceCell<Weak<LevelEditorUI>> = OnceCell::new();
static SELECTED_ENTITY_ID: OnceCell<Mutex<SharedString>> = OnceCell::new();
static CACHED_COMPONENTS: OnceCell<Mutex<Vec<ComponentUI>>> = OnceCell::new();

pub struct InterfaceSystem;

impl InterfaceSystem {
    // Convert from Rust ComponentUI to Slint ComponentUI
    fn convert_to_slint_component_ui(rust_component: &ComponentUI) -> SlintComponentUI {
        let slint_attributes: Vec<SlintAttributeUI> = rust_component.attributes
            .iter()
            .map(|attr| SlintAttributeUI {
                name: attr.name.clone(),
                value: attr.value.clone(),
                dt_type: attr.dt_type.clone(),
            })
            .collect();

        SlintComponentUI {
            name: rust_component.name.clone(),
            attributes: ModelRc::new(VecModel::from(slint_attributes)),
        }
    }

    pub fn initialize(ui_context: &LevelEditorUI) {
        UI_HANDLE.set(ui_context.as_weak()).unwrap_or_else(|_| {
            panic!("UI_HANDLE already set");
        });

        let state = ui_context.global::<InterfaceState>();

        SELECTED_ENTITY_ID.set(Mutex::new(SharedString::from(""))).ok();
        CACHED_COMPONENTS.set(Mutex::new(Vec::new())).ok();

        state.on_selected_changed(|entity_id: SharedString| {
            if let Some(mutex) = SELECTED_ENTITY_ID.get() {
                *mutex.lock().unwrap() = entity_id.clone();
                // Clear cache when selection changes
                if let Some(cache_mutex) = CACHED_COMPONENTS.get() {
                    cache_mutex.lock().unwrap().clear();
                }
            }
        });
    }

    pub fn update() {
        let selected_id = SELECTED_ENTITY_ID.get()
            .map(|mutex| mutex.lock().unwrap().clone())
            .unwrap_or_else(|| SharedString::from(""));

        if selected_id == "" {
            return;
        }

        let components_ui = get_all_components_by_id!(selected_id.to_string());

        // Check if data has actually changed
        let has_changed = {
            if let Some(cache_mutex) = CACHED_COMPONENTS.get() {
                let cached = cache_mutex.lock().unwrap();
                if cached.len() != components_ui.len() {
                    true
                } else {
                    // Compare each component's data
                    cached
                        .iter()
                        .zip(components_ui.iter())
                        .any(|(cached_comp, new_comp)| {
                            Self::component_data_changed(cached_comp, new_comp)
                        })
                }
            } else {
                true // No cache yet, so it's changed
            }
        };

        // Only update UI if data has actually changed
        if has_changed {
            println!("[UI] Updating components for entity: {}", selected_id);
            
            // Update cache
            if let Some(cache_mutex) = CACHED_COMPONENTS.get() {
                *cache_mutex.lock().unwrap() = components_ui.clone();
            }

            // Convert to Slint ComponentUI
            let slint_components_ui: Vec<SlintComponentUI> = components_ui
                .iter()
                .map(|c| Self::convert_to_slint_component_ui(c))
                .collect();

            let ui = UI_HANDLE.get()
                .expect("UI_HANDLE not initialized")
                .upgrade()
                .expect("UI instance already dropped");

            let state = ui.global::<InterfaceState>();

            let components_model = VecModel::from(slint_components_ui);
            state.set_components(ModelRc::new(components_model).into());
        }
    }

    // Helper function to check if component data has changed
    fn component_data_changed(cached: &ComponentUI, new: &ComponentUI) -> bool {
        if cached.name != new.name || cached.attributes.len() != new.attributes.len() {
            return true;
        }

        // Compare each attribute
        cached.attributes
            .iter()
            .zip(new.attributes.iter())
            .any(|(cached_attr, new_attr)| {
                cached_attr.name != new_attr.name ||
                    cached_attr.value != new_attr.value ||
                    cached_attr.dt_type != new_attr.dt_type
            })
    }

    pub fn update_entity_tree() {
        let ui = UI_HANDLE.get()
            .expect("UI_HANDLE not initialized")
            .upgrade()
            .expect("UI instance already dropped");

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

    pub fn set_selected_element(entity_id: &str) {
        let ui = UI_HANDLE.get()
            .expect("UI_HANDLE not initialized")
            .upgrade()
            .expect("UI instance already dropped");

        let state = ui.global::<InterfaceState>();
        state.set_selected_index(SharedString::from(entity_id));
    }
}

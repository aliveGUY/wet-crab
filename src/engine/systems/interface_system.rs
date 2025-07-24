use once_cell::sync::OnceCell;
use slint::{ ComponentHandle, Weak, SharedString, VecModel, ModelRc, Model };
use crate::{ InterfaceState, LevelEditorUI };
use crate::query_get_all;
use crate::index::engine::components::Metadata;

static UI_HANDLE: OnceCell<Weak<LevelEditorUI>> = OnceCell::new();

pub struct InterfaceSystem;

impl InterfaceSystem {
    pub fn initialize(ui_context: &LevelEditorUI) {
        UI_HANDLE.set(ui_context.as_weak()).unwrap_or_else(|_| {
            panic!("UI_HANDLE already set");
        });
    }

    pub fn update() {}

    pub fn update_entity_tree() {
        let ui = UI_HANDLE.get()
            .expect("UI_HANDLE not initialized")
            .upgrade()
            .expect("UI instance already dropped");

        let state = ui.global::<InterfaceState>();
        
        let all_entities_with_metadata = query_get_all!(Metadata);
        
        let entities_model: VecModel<(SharedString, SharedString)> = VecModel::default();
        
        for (entity_id, metadata) in all_entities_with_metadata {
            let entity_data = (
                SharedString::from(entity_id),
                SharedString::from(metadata.title()),
            );
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

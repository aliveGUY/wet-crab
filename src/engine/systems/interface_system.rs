use once_cell::sync::OnceCell;
use slint::{ ComponentHandle, Weak };
use crate::{ InterfaceState, LevelEditorUI };

static UI_HANDLE: OnceCell<Weak<LevelEditorUI>> = OnceCell::new();

pub struct InterfaceSystem;

impl InterfaceSystem {
    pub fn initialize(ui_context: &LevelEditorUI) {
        UI_HANDLE.set(ui_context.as_weak()).unwrap_or_else(|_| {
            panic!("UI_HANDLE already set");
        });
    }

    pub fn update() {}

    pub fn set_selected_element(index: i32) {
        let ui = UI_HANDLE.get()
            .expect("UI_HANDLE not initialized")
            .upgrade()
            .expect("UI instance already dropped");

        let state = ui.global::<InterfaceState>();
        state.set_selected_index(index);
    }
}

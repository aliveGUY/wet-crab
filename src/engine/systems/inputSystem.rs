use std::sync::{ Arc, OnceLock };
use std::any::Any;
use super::eventSystem::{ Event, EventSystem, EventType };

pub trait InputHandler: Send + Sync {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event>;
    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event>;
}

static INPUT_SYSTEM: OnceLock<InputSystem> = OnceLock::new();

pub struct InputSystem {
    handler: Arc<dyn InputHandler>,
}

impl InputSystem {
    pub fn initialize(handler: Box<dyn InputHandler>) {
        INPUT_SYSTEM.set(InputSystem {
            handler: Arc::from(handler),
        }).expect("InputSystem already initialized");
    }

    pub fn instance() -> &'static InputSystem {
        INPUT_SYSTEM.get().expect("InputSystem not initialized")
    }

    pub fn receive_mouse_event(&self, raw_event: &dyn Any) {
        if let Some(event) = self.handler.receive_mouse_event(raw_event) {
            EventSystem::notify(event);
        }
    }

    pub fn receive_key_event(&self, raw_event: &dyn Any) {
        if let Some(event) = self.handler.receive_key_event(raw_event) {
            EventSystem::notify(event);
        }
    }
}

pub struct DesktopInputHandler {}

impl DesktopInputHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputHandler for DesktopInputHandler {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event> {
        if let Some(position) = raw_event.downcast_ref::<(i32, i32)>() {
            let euler_deltas = crate::index::engine::utils::inputUtils::mouse_delta_to_euler(
                position.0 as f64,
                position.1 as f64
            );

            return Some(Event {
                event_type: EventType::RotateCamera,
                payload: Box::new(euler_deltas),
            });
        }

        None
    }

    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event> {
        if let Some(direction) = raw_event.downcast_ref::<String>() {
            return Some(Event {
                event_type: EventType::Move,
                payload: Box::new(direction.clone()),
            });
        }

        None
    }
}

pub struct BrowserInputHandler {}

impl BrowserInputHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputHandler for BrowserInputHandler {
    fn receive_mouse_event(&self, raw_event: &dyn Any) -> Option<Event> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(mouse_event) = raw_event.downcast_ref::<web_sys::MouseEvent>() {
                let movement_x = mouse_event.movement_x();
                let movement_y = mouse_event.movement_y();

                if movement_x != 0 || movement_y != 0 {
                    let euler_deltas =
                        crate::index::engine::utils::inputUtils::mouse_delta_to_euler(
                            movement_x as f64,
                            movement_y as f64
                        );

                    return Some(Event {
                        event_type: EventType::RotateCamera,
                        payload: Box::new(euler_deltas),
                    });
                }
            }
        }

        None
    }

    fn receive_key_event(&self, raw_event: &dyn Any) -> Option<Event> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(key_event) = raw_event.downcast_ref::<web_sys::KeyboardEvent>() {
                let key_code = key_event.code();

                let direction = match key_code.as_str() {
                    "KeyW" => Some("forward"),
                    "KeyS" => Some("backward"),
                    "KeyA" => Some("left"),
                    "KeyD" => Some("right"),
                    "Space" => Some("up"),
                    "ShiftLeft" | "ShiftRight" => Some("down"),
                    _ => None,
                };

                if let Some(dir) = direction {
                    return Some(Event {
                        event_type: EventType::Move,
                        payload: Box::new(dir.to_string()),
                    });
                }
            }
        }

        None
    }
}

impl std::fmt::Debug for InputSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputSystem").field("has_handler", &true).finish()
    }
}

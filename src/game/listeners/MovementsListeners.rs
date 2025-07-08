use crate::index::event_system::{ Event, EventListener };
use crate::index::game_state::add_camera_rotation_delta;

pub struct CameraRotationListener;
pub struct MovementListener;

impl EventListener for CameraRotationListener {
    fn update(&self, event: &Event) {
        if let Some([pitch_delta, yaw_delta]) = event.payload.downcast_ref::<[f32; 2]>() {
            // Directly use the pitch and yaw deltas from the event
            add_camera_rotation_delta(*pitch_delta, *yaw_delta);
            println!("Applied camera rotation delta: pitch={}, yaw={}", pitch_delta, yaw_delta);
        } else {
            println!("CameraRotationListener received incompatible payload.");
        }
    }
}

impl EventListener for MovementListener {
    fn update(&self, event: &Event) {
        if let Some(direction) = event.payload.downcast_ref::<String>() {
            println!("Moving in direction: {}", direction);
        } else {
            println!("MovementListener received incompatible payload.");
        }
    }
}

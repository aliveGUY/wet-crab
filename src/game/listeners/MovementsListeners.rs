use crate::index::event_system::{Event, EventListener};

pub struct CameraRotationListener;
pub struct MovementListener;

impl EventListener for CameraRotationListener {
    fn update(&self, event: &Event) {
        if let Some(quaternion) = event.payload.downcast_ref::<[f32; 4]>() {
            println!("Rotating camera with quaternion: {:?}", quaternion);
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

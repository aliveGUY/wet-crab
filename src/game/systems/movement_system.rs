// Import types and functions from parent scope
use crate::index::engine::components::{SystemTrait, CameraComponent};
use crate::index::engine::systems::event_system::Event;
use crate::index::PLAYER_ENTITY_ID;

#[derive(Debug)]
pub struct CameraRotationSystem;

#[derive(Debug)]
pub struct MovementSystem;

impl SystemTrait for CameraRotationSystem {
    fn event(&self, event: &Event) {
        let player_entity_id = match PLAYER_ENTITY_ID.read().unwrap().as_ref() {
            Some(id) => id.clone(),
            None => return,
        };
        
        let [pitch_delta, yaw_delta] = match event.payload.downcast_ref::<[f32; 2]>() {
            Some(deltas) => *deltas,
            None => return,
        };

        query_by_id!(player_entity_id, (CameraComponent), |camera| {
            camera.add_rotation_delta(pitch_delta, yaw_delta);
        });
    }
}

impl SystemTrait for MovementSystem {
    fn event(&self, event: &Event) {
        let player_entity_id = match PLAYER_ENTITY_ID.read().unwrap().as_ref() {
            Some(id) => id.clone(),
            None => return,
        };
        
        let dir_text = match event.payload.downcast_ref::<String>() {
            Some(s) => s.as_str(),
            None => return,
        };

        let mut fwd = false;
        let mut back = false;
        let mut left = false;
        let mut right = false;

        for token in dir_text.split('-') {
            match token {
                "forward" => fwd = true,
                "backward" => back = true,
                "left" => left = true,
                "right" => right = true,
                _ => {}
            }
        }

        if fwd && back {
            fwd = false;
            back = false;
        }
        if left && right {
            left = false;
            right = false;
        }

        const STEP: f32 = 0.1;

        query_by_id!(player_entity_id, (CameraComponent), |camera| {
            match (fwd, back, left, right) {
                (true, false, false, false) => camera.move_forward(STEP),
                (false, true, false, false) => camera.move_back(STEP),
                (false, false, false, true) => camera.move_right(STEP),
                (false, false, true, false) => camera.move_left(STEP),

                (true, false, true, false) => camera.move_forward_left(STEP),
                (true, false, false, true) => camera.move_forward_right(STEP),
                (false, true, true, false) => camera.move_back_left(STEP),
                (false, true, false, true) => camera.move_back_right(STEP),

                _ => {}
            }
        });
    }
}

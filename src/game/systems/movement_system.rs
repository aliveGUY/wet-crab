// Import types and functions from parent scope
use crate::index::engine::components::{SystemTrait, CameraComponent, Transform};
use crate::index::engine::modules::event_system::Event;
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

        crate::query_by_id!(player_entity_id, (CameraComponent), |camera| {
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
        
        let direction_string = match event.payload.downcast_ref::<String>() {
            Some(s) => s.as_str(),
            None => return,
        };

        if direction_string.is_empty() { return; }

        // Parse direction string and apply transforms directly
        crate::query_by_id!(player_entity_id, (CameraComponent, Transform), |camera, transform| {
            let (forward_vec, right_vec, up_vec) = camera.get_basis_vectors();
            let mut total_movement = [0.0, 0.0, 0.0];

            // Process each direction token
            for token in direction_string.split('-') {
                match token {
                    "forward" => {
                        total_movement[0] += -forward_vec[0];
                        total_movement[1] += -forward_vec[1];
                        total_movement[2] += -forward_vec[2];
                    },
                    "backward" => {
                        total_movement[0] += forward_vec[0];
                        total_movement[1] += forward_vec[1];
                        total_movement[2] += forward_vec[2];
                    },
                    "left" => {
                        total_movement[0] += -right_vec[0];
                        total_movement[1] += -right_vec[1];
                        total_movement[2] += -right_vec[2];
                    },
                    "right" => {
                        total_movement[0] += right_vec[0];
                        total_movement[1] += right_vec[1];
                        total_movement[2] += right_vec[2];
                    },
                    "up" => {
                        total_movement[0] += up_vec[0];
                        total_movement[1] += up_vec[1];
                        total_movement[2] += up_vec[2];
                    },
                    "down" => {
                        total_movement[0] += -up_vec[0];
                        total_movement[1] += -up_vec[1];
                        total_movement[2] += -up_vec[2];
                    },
                    _ => {}
                }
            }

            // Apply movement with speed and timing
            let movement_speed = 5.0;
            let delta_time = 1.0 / 60.0;
            let movement_distance = movement_speed * delta_time;
            
            total_movement[0] *= movement_distance;
            total_movement[1] *= movement_distance;
            total_movement[2] *= movement_distance;
            
            // Apply to transform using the new component-based approach
            transform.translate(
                total_movement[0], 
                total_movement[1], 
                total_movement[2]
            );
        });
    }
}

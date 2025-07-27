// Import types and functions from parent scope
use crate::index::engine::components::{SystemTrait, CameraComponent, Transform};
use crate::index::engine::systems::event_system::Event;
use crate::index::engine::Component; // Import Component trait for update_component_ui
use crate::index::engine::utils::math::{mat4x4_translate, mat4x4_mul};
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
            camera.update_component_ui(&player_entity_id); // Update UI after component change
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
        let mut up = false;
        let mut down = false;

        for token in dir_text.split('-') {
            match token {
                "forward" => fwd = true,
                "backward" => back = true,
                "left" => left = true,
                "right" => right = true,
                "up" => up = true,
                "down" => down = true,
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
        if up && down {
            up = false;
            down = false;
        }

        // Direct movement - apply movement immediately to transform

        // Only move if any movement keys are pressed
        if fwd || back || left || right || up || down {
            // Get camera basis vectors and apply direct movement
            crate::query_by_id!(player_entity_id, (CameraComponent, Transform), |camera, transform| {
                // Get camera basis vectors for movement calculation
                let (forward_vec, right_vec, up_vec) = camera.get_basis_vectors();
                
                // Calculate movement direction vector
                let mut movement_direction = [0.0, 0.0, 0.0];
                
                // Handle horizontal movement
                match (fwd, back, left, right) {
                    (true, false, false, false) => {
                        movement_direction[0] += -forward_vec[0];
                        movement_direction[1] += -forward_vec[1];
                        movement_direction[2] += -forward_vec[2];
                    },
                    (false, true, false, false) => {
                        movement_direction[0] += forward_vec[0];
                        movement_direction[1] += forward_vec[1];
                        movement_direction[2] += forward_vec[2];
                    },
                    (false, false, false, true) => {
                        movement_direction[0] += right_vec[0];
                        movement_direction[1] += right_vec[1];
                        movement_direction[2] += right_vec[2];
                    },
                    (false, false, true, false) => {
                        movement_direction[0] += -right_vec[0];
                        movement_direction[1] += -right_vec[1];
                        movement_direction[2] += -right_vec[2];
                    },
                    (true, false, true, false) => {
                        let s = 0.70710677; // sqrt(2)/2 for diagonal movement
                        movement_direction[0] += (-forward_vec[0] - right_vec[0]) * s;
                        movement_direction[1] += (-forward_vec[1] - right_vec[1]) * s;
                        movement_direction[2] += (-forward_vec[2] - right_vec[2]) * s;
                    },
                    (true, false, false, true) => {
                        let s = 0.70710677;
                        movement_direction[0] += (-forward_vec[0] + right_vec[0]) * s;
                        movement_direction[1] += (-forward_vec[1] + right_vec[1]) * s;
                        movement_direction[2] += (-forward_vec[2] + right_vec[2]) * s;
                    },
                    (false, true, true, false) => {
                        let s = 0.70710677;
                        movement_direction[0] += (forward_vec[0] - right_vec[0]) * s;
                        movement_direction[1] += (forward_vec[1] - right_vec[1]) * s;
                        movement_direction[2] += (forward_vec[2] - right_vec[2]) * s;
                    },
                    (false, true, false, true) => {
                        let s = 0.70710677;
                        movement_direction[0] += (forward_vec[0] + right_vec[0]) * s;
                        movement_direction[1] += (forward_vec[1] + right_vec[1]) * s;
                        movement_direction[2] += (forward_vec[2] + right_vec[2]) * s;
                    },
                    _ => {}
                }
                
                // Handle vertical movement
                match (up, down) {
                    (true, false) => {
                        movement_direction[0] += up_vec[0];
                        movement_direction[1] += up_vec[1];
                        movement_direction[2] += up_vec[2];
                    },
                    (false, true) => {
                        movement_direction[0] += -up_vec[0];
                        movement_direction[1] += -up_vec[1];
                        movement_direction[2] += -up_vec[2];
                    },
                    _ => {}
                }
                
                // Scale movement by speed and delta time (assuming 60fps for now)
                let movement_speed = 5.0; // units per second
                let delta_time = 1.0 / 60.0; // ~16ms frame time
                let movement_distance = movement_speed * delta_time;
                
                movement_direction[0] *= movement_distance;
                movement_direction[1] *= movement_distance;
                movement_direction[2] *= movement_distance;
                
                // Apply movement directly to transform
                let translation_matrix = mat4x4_translate(
                    movement_direction[0], 
                    movement_direction[1], 
                    movement_direction[2]
                );
                let new_matrix = mat4x4_mul(translation_matrix, *transform.get_matrix());
                *transform.get_matrix_mut() = new_matrix;
                
                // Update UI to reflect the new position
                transform.update_component_ui(&player_entity_id);
                camera.update_component_ui(&player_entity_id);
            });
        }
    }
}

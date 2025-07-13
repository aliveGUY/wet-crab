// Import types and functions from parent scope
use crate::index::System;
use crate::index::event_system::Event;
use crate::index::game_state::{
    add_camera_rotation_delta,
    move_camera_forward,
    move_camera_back,
    move_camera_left,
    move_camera_right,
    move_camera_forward_left,
    move_camera_forward_right,
    move_camera_back_left,
    move_camera_back_right,
};

#[derive(Debug)]
pub struct CameraRotationSystem;

#[derive(Debug)]
pub struct MovementSystem;

impl System for CameraRotationSystem {
    fn event(&self, event: &Event) {
        if let Some([pitch_delta, yaw_delta]) = event.payload.downcast_ref::<[f32; 2]>() {
            add_camera_rotation_delta(*pitch_delta, *yaw_delta);
        }
    }
}

impl System for MovementSystem {
    fn event(&self, event: &Event) {
        let dir_text = match event.payload.downcast_ref::<String>() {
            Some(s) => s.as_str(),
            None => {
                return;
            }
        };

        let mut fwd = false;
        let mut back = false;
        let mut left = false;
        let mut right = false;

        for token in dir_text.split('-') {
            match token {
                "forward" => {
                    fwd = true;
                }
                "backward" => {
                    back = true;
                }
                "left" => {
                    left = true;
                }
                "right" => {
                    right = true;
                }
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

        match (fwd, back, left, right) {
            (true, false, false, false) => move_camera_forward(STEP),
            (false, true, false, false) => move_camera_back(STEP),
            (false, false, false, true) => move_camera_right(STEP),
            (false, false, true, false) => move_camera_left(STEP),

            (true, false, true, false) => move_camera_forward_left(STEP),
            (true, false, false, true) => move_camera_forward_right(STEP),
            (false, true, true, false) => move_camera_back_left(STEP),
            (false, true, false, true) => move_camera_back_right(STEP),

            _ => {}
        }
    }
}

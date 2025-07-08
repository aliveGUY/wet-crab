use once_cell::sync::OnceCell;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::index::math::{
    Mat4x4,
    mat4x4_identity,
    mat4x4_from_euler,
    mat4x4_translate,
    mat4x4_mul,
};

#[derive(Debug)]
struct GameState {
    camera_position: [f32; 3],
    camera_pitch: f32,
    camera_yaw: f32,
    camera_roll: f32,
    camera_transform: Mat4x4,
    transform_dirty: bool,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            camera_position: [0.0, 0.0, 0.0],
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            camera_roll: 0.0,
            camera_transform: mat4x4_identity(),
            transform_dirty: true,
        }
    }

    fn update_transform_matrix(&mut self) {
        if self.transform_dirty {
            let rotation_matrix = mat4x4_from_euler(
                self.camera_pitch,
                self.camera_yaw,
                self.camera_roll
            );
            let translation_matrix = mat4x4_translate(
                self.camera_position[0],
                self.camera_position[1],
                self.camera_position[2]
            );

            self.camera_transform = mat4x4_mul(translation_matrix, rotation_matrix);
            self.transform_dirty = false;
        }
    }

    fn get_transform_matrix(&mut self) -> Mat4x4 {
        self.update_transform_matrix();
        self.camera_transform
    }
}

static GAME_STATE: OnceCell<Arc<RwLock<GameState>>> = OnceCell::new();

pub fn initialize_game_state() {
    let state = Arc::new(RwLock::new(GameState::new()));
    GAME_STATE.set(state).expect("GameState already initialized");
}

fn get_game_state() -> Arc<RwLock<GameState>> {
    GAME_STATE.get().expect("GameState not initialized").clone()
}

pub fn get_camera_transform() -> Mat4x4 {
    let state = get_game_state();
    let result = match state.try_write() {
        Ok(mut guard) => guard.get_transform_matrix(),
        Err(_) => {
            match state.try_read() {
                Ok(guard) => guard.camera_transform,
                Err(_) => mat4x4_identity(),
            }
        }
    };
    result
}

pub fn set_camera_translation(x: f32, y: f32, z: f32) {
    let state = get_game_state();
    let _result = match state.try_write() {
        Ok(mut guard) => {
            guard.camera_position = [x, y, z];
            guard.transform_dirty = true;
            true
        }
        Err(_) => false,
    };
}

pub fn add_camera_rotation_delta(pitch_delta: f32, yaw_delta: f32) {
    let state = get_game_state();
    let _result = match state.try_write() {
        Ok(mut guard) => {
            // Invert yaw to fix horizontal rotation direction
            guard.camera_yaw -= yaw_delta;  // Note: minus instead of plus
            guard.camera_pitch += pitch_delta;

            guard.camera_pitch = guard.camera_pitch.clamp(-1.5, 1.5);
            guard.transform_dirty = true;  // Mark for rebuild
            true
        }
        Err(_) => false,
    };
}

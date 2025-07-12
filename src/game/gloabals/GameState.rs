use once_cell::sync::OnceCell;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::f32::consts::PI;

use crate::index::math::{
    Mat4x4,
    mat4x4_identity,
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
            self.camera_transform = build_view_matrix(
                self.camera_position,
                self.camera_pitch,
                self.camera_yaw
            );
            self.transform_dirty = false;
        }
    }

    fn get_transform_matrix(&mut self) -> Mat4x4 {
        self.update_transform_matrix();
        self.camera_transform
    }
}

fn build_view_matrix(pos: [f32; 3], pitch: f32, yaw: f32) -> Mat4x4 {
    // FPS-style camera with NO ROLL - manual matrix construction
    let cp = pitch.cos();
    let sp = pitch.sin();
    let cy = yaw.cos();
    let sy = yaw.sin();

    // Camera basis vectors (right-handed coordinate system)
    // Forward vector (camera looks down negative Z in camera space)
    let forward = [sy * cp, -sp, -cy * cp];
    // Right vector (cross product of world up and forward)
    let right = [cy, 0.0, sy];
    // Up vector (cross product of forward and right)
    let up = [sy * sp, cp, -cy * sp];

    // View matrix = [R|t] where R is rotation (camera basis) and t is translation
    // For view matrix, we need the inverse transform: R^T and -R^T * pos
    let tx = -(right[0] * pos[0] + right[1] * pos[1] + right[2] * pos[2]);
    let ty = -(up[0] * pos[0] + up[1] * pos[1] + up[2] * pos[2]);
    let tz = -(forward[0] * pos[0] + forward[1] * pos[1] + forward[2] * pos[2]);

    // Construct view matrix directly (row-major)
    [
        right[0],   right[1],   right[2],   tx,
        up[0],      up[1],      up[2],      ty,
        forward[0], forward[1], forward[2], tz,
        0.0,        0.0,        0.0,        1.0,
    ]
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
            guard.camera_yaw += yaw_delta; // Note: minus instead of plus
            guard.camera_pitch += pitch_delta;

            guard.camera_pitch = guard.camera_pitch.clamp(-1.5, 1.5);
            guard.transform_dirty = true; // Mark for rebuild
            true
        }
        Err(_) => false,
    };
}

#[inline]
fn basis_from_yaw(yaw: f32) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let cy = yaw.cos();
    let sy = yaw.sin();

    // forward lies in X-Z, length 1
    let forward = [sy, 0.0, -cy];
    // right is perpendicular to forward, still in X-Z
    let right = [cy, 0.0, sy];
    // world-up stays constant
    let up = [0.0, 1.0, 0.0];

    (forward, right, up)
}

fn move_camera_relative(forward: f32, right: f32, up: f32) {
    let state = get_game_state();
    if let Ok(mut guard) = state.try_write() {
        // use *only* yaw for movement
        let (f, r, u) = basis_from_yaw(guard.camera_yaw);

        guard.camera_position[0] += forward * f[0] + right * r[0] + up * u[0];
        guard.camera_position[1] += forward * f[1] + right * r[1] + up * u[1];
        guard.camera_position[2] += forward * f[2] + right * r[2] + up * u[2];

        guard.transform_dirty = true;
    };
}

// public helpers – now call the new relative mover
pub fn move_camera_forward(step: f32) {
    move_camera_relative(step, 0.0, 0.0);
}
pub fn move_camera_back(step: f32) {
    move_camera_relative(-step, 0.0, 0.0);
}
pub fn move_camera_right(step: f32) {
    move_camera_relative(0.0, step, 0.0);
}
pub fn move_camera_left(step: f32) {
    move_camera_relative(0.0, -step, 0.0);
}
pub fn move_camera_up(step: f32) {
    move_camera_relative(0.0, 0.0, step);
}
pub fn move_camera_down(step: f32) {
    move_camera_relative(0.0, 0.0, -step);
}

pub fn move_camera_forward_right(step: f32) {
    let s = step * 0.70710677; // 1/√2, keeps speed constant
    move_camera_relative(s, s, 0.0);
}
pub fn move_camera_forward_left(step: f32) {
    let s = step * 0.70710677;
    move_camera_relative(s, -s, 0.0);
}
pub fn move_camera_back_right(step: f32) {
    let s = step * 0.70710677;
    move_camera_relative(-s, s, 0.0);
}
pub fn move_camera_back_left(step: f32) {
    let s = step * 0.70710677;
    move_camera_relative(-s, -s, 0.0);
}

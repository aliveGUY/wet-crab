pub fn calculate_movement_direction(w: bool, a: bool, s: bool, d: bool) -> String {
    let forward = w && !s;
    let back = s && !w;
    let left = a && !d;
    let right = d && !a;

    match (forward, back, left, right) {
        (true, false, true, false) => "forward-left".to_string(),
        (true, false, false, true) => "forward-right".to_string(),
        (false, true, true, false) => "back-left".to_string(),
        (false, true, false, true) => "back-right".to_string(),
        (true, false, false, false) => "forward".to_string(),
        (false, true, false, false) => "back".to_string(),
        (false, false, true, false) => "left".to_string(),
        (false, false, false, true) => "right".to_string(),
        _ => "idle".to_string(),
    }
}

pub fn mouse_delta_to_euler(delta_x: f64, delta_y: f64) -> [f32; 2] {
    let sensitivity = 0.002;
    let yaw_delta = (delta_x * sensitivity) as f32;
    let pitch_delta = (delta_y * sensitivity) as f32;
    [pitch_delta, yaw_delta]
}

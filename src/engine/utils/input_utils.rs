pub fn calculate_movement_direction(w: bool, a: bool, s: bool, d: bool) -> String {
    let forward = w && !s;
    let back = s && !w;
    let left = a && !d;
    let right = d && !a;

    match (forward, back, left, right) {
        (true, false, true, false) => "forward-left".to_string(),
        (true, false, false, true) => "forward-right".to_string(),
        (false, true, true, false) => "backward-left".to_string(),
        (false, true, false, true) => "backward-right".to_string(),
        (true, false, false, false) => "forward".to_string(),
        (false, true, false, false) => "backward".to_string(),
        (false, false, true, false) => "left".to_string(),
        (false, false, false, true) => "right".to_string(),
        _ => "idle".to_string(),
    }
}

pub fn calculate_movement_direction_3d(w: bool, a: bool, s: bool, d: bool, up: bool, down: bool) -> String {
    let forward = w && !s;
    let back = s && !w;
    let left = a && !d;
    let right = d && !a;
    let vertical_up = up && !down;
    let vertical_down = down && !up;

    // Build direction components
    let mut components = Vec::new();
    
    // Add horizontal movement
    match (forward, back, left, right) {
        (true, false, true, false) => {
            components.push("forward");
            components.push("left");
        },
        (true, false, false, true) => {
            components.push("forward");
            components.push("right");
        },
        (false, true, true, false) => {
            components.push("backward");
            components.push("left");
        },
        (false, true, false, true) => {
            components.push("backward");
            components.push("right");
        },
        (true, false, false, false) => components.push("forward"),
        (false, true, false, false) => components.push("backward"),
        (false, false, true, false) => components.push("left"),
        (false, false, false, true) => components.push("right"),
        _ => {} // No horizontal movement
    }
    
    // Add vertical movement
    match (vertical_up, vertical_down) {
        (true, false) => components.push("up"),
        (false, true) => components.push("down"),
        _ => {} // No vertical movement
    }
    
    // Join components or return idle
    if components.is_empty() {
        "idle".to_string()
    } else {
        components.join("-")
    }
}

pub fn mouse_delta_to_euler(delta_x: f64, delta_y: f64) -> [f32; 2] {
    let sensitivity = 0.002;
    let yaw_delta = (delta_x * sensitivity) as f32;
    // Natural FPS camera feel: mouse up -> look up, mouse down -> look down
    let pitch_delta = (delta_y * sensitivity) as f32;
    [pitch_delta, yaw_delta]  // return as [pitch_delta, yaw_delta]
}

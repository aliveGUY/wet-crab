pub type Mat4x4 = [f32; 16];

pub fn mat4x4_identity() -> Mat4x4 {
    [
      1.0, 0.0, 0.0, 0.0, 
      0.0, 1.0, 0.0, 0.0, 
      0.0, 0.0, 1.0, 0.0, 
      0.0, 0.0, 0.0, 1.0
    ]
}

pub fn mat4x4_translate(x: f32, y: f32, z: f32) -> Mat4x4 {
    [
      1.0, 0.0, 0.0,  x, 
      0.0, 1.0, 0.0,  y,
      0.0, 0.0, 1.0,  z, 
      0.0, 0.0, 0.0, 1.0
    ]
}

#[allow(dead_code)]
pub fn mat4x4_rot_x(angle: f32) -> Mat4x4 {
    let c = angle.cos();
    let s = angle.sin();

    [
      1.0, 0.0, 0.0, 0.0,
      0.0,  c,  -s,  0.0,
      0.0,  s,   c,  0.0,
      0.0, 0.0, 0.0, 1.0
    ]
}

#[allow(dead_code)]
pub fn mat4x4_rot_y(angle: f32) -> Mat4x4 {
    let c = angle.cos();
    let s = angle.sin();

    [
       c,  0.0, -s,  0.0, 
      0.0, 1.0, 0.0, 0.0, 
       s,  0.0,  c,  0.0, 
      0.0, 0.0, 0.0, 1.0
    ]
}

#[allow(dead_code)]
pub fn mat4x4_rot_z(angle: f32) -> Mat4x4 {
    let c = angle.cos();
    let s = angle.sin();

    [
       c,  -s,  0.0, 0.0,
       s,   c,  0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0
    ]
}


pub fn mat4x4_scale(x: f32, y: f32, z: f32) -> Mat4x4 {
    [
       x,  0.0, 0.0, 0.0, 
      0.0,  y,  0.0, 0.0, 
      0.0, 0.0,  z,  0.0, 
      0.0, 0.0, 0.0, 1.0
    ]
}

pub fn mat4x4_from_quat(quat: [f32; 4]) -> Mat4x4 {
    let [x, y, z, w] = quat;
    let x2 = x * x;
    let y2 = y * y;
    let z2 = z * z;
    let w2 = w * w;

    let xy = 2.0 * x * y;
    let xz = 2.0 * x * z;
    let xw = 2.0 * x * w;
    let yz = 2.0 * y * z;
    let yw = 2.0 * y * w;
    let zw = 2.0 * z * w;

    [
        w2 + x2 - y2 - z2,  xy - zw,            xz + yw,            0.0,
        xy + zw,            w2 - x2 + y2 - z2,  yz - xw,            0.0,
        xz - yw,            yz + xw,            w2 - x2 - y2 + z2,  0.0,
        0.0,                0.0,                0.0,                1.0,
    ]
}

pub fn mat4x4_transpose(matrix: Mat4x4) -> Mat4x4 {
    let mut ret = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        ret[col * 4 + row] = matrix[row * 4 + col];
    }
    ret
}

pub fn vec4_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

pub fn mat4x4_row(mat: &Mat4x4, row: usize) -> [f32; 4] {
    let start_idx = row * 4;
    [mat[start_idx], mat[start_idx + 1], mat[start_idx + 2], mat[start_idx + 3]]
}

pub fn mat4x4_col(mat: &Mat4x4, col: usize) -> [f32; 4] {
    [mat[col], mat[4 + col], mat[8 + col], mat[12 + col]]
}

pub fn mat4x4_mul(a: Mat4x4, b: Mat4x4) -> Mat4x4 {
    let mut ret = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        let a_row = mat4x4_row(&a, row);
        let b_col = mat4x4_col(&b, col);
        ret[i] = vec4_dot(a_row, b_col);
    }
    ret
}

pub fn mat4x4_perspective(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4x4 {
    let f = 1.0 / (fov_y_radians * 0.5).tan();
    let range_inv = 1.0 / (near - far);
    
    [
        f / aspect_ratio, 0.0, 0.0,                          0.0,
        0.0,              f,   0.0,                          0.0,
        0.0,              0.0, (near + far) * range_inv,     (2.0 * near * far) * range_inv,
        0.0,              0.0, -1.0,                         0.0,
    ]
}

// Linear interpolation utility function
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

// Build view matrix from position and Euler angles
pub fn build_view_matrix(pos: [f32; 3], pitch: f32, yaw: f32) -> Mat4x4 {
    let cp = pitch.cos();
    let sp = pitch.sin();
    let cy = yaw.cos();
    let sy = yaw.sin();

    let forward = [-sy * cp, sp, cy * cp];
    let right = [cy, 0.0, sy];
    let up = [sy * sp, cp, -cy * sp];

    let tx = -(right[0] * pos[0] + right[1] * pos[1] + right[2] * pos[2]);
    let ty = -(up[0] * pos[0] + up[1] * pos[1] + up[2] * pos[2]);
    let tz = -(forward[0] * pos[0] + forward[1] * pos[1] + forward[2] * pos[2]);

    [
        right[0],   right[1],   right[2],   tx,
        up[0],      up[1],      up[2],      ty,
        forward[0], forward[1], forward[2], tz,
        0.0,        0.0,        0.0,        1.0,
    ]
}

// Calculate world transform for a node in a skeleton hierarchy
pub fn node_world_txfm(nodes: &[crate::index::engine::components::AnimatedObject3D::Node], idx: usize) -> Mat4x4 {
    let node = &nodes[idx];

    let mut node_txfm = mat4x4_scale(node.scale[0], node.scale[1], node.scale[2]);
    node_txfm = mat4x4_mul(mat4x4_from_quat(node.rotation), node_txfm);
    node_txfm = mat4x4_mul(
        mat4x4_translate(node.translation[0], node.translation[1], node.translation[2]),
        node_txfm
    );

    if node.parent != u32::MAX {
        node_txfm = mat4x4_mul(node_world_txfm(nodes, node.parent as usize), node_txfm);
    }

    node_txfm
}

// Extract translation from a 4x4 transformation matrix
pub fn mat4x4_extract_translation(matrix: &Mat4x4) -> [f32; 3] {
    [matrix[3], matrix[7], matrix[11]]
}

// Extract scale from a 4x4 transformation matrix
pub fn mat4x4_extract_scale(matrix: &Mat4x4) -> [f32; 3] {
    let sx = (matrix[0] * matrix[0] + matrix[1] * matrix[1] + matrix[2] * matrix[2]).sqrt();
    let sy = (matrix[4] * matrix[4] + matrix[5] * matrix[5] + matrix[6] * matrix[6]).sqrt();
    let sz = (matrix[8] * matrix[8] + matrix[9] * matrix[9] + matrix[10] * matrix[10]).sqrt();
    [sx, sy, sz]
}

// Extract Euler angles (in radians) from a 4x4 transformation matrix
// Returns [pitch, yaw, roll] in radians
pub fn mat4x4_extract_euler_angles(matrix: &Mat4x4) -> [f32; 3] {
    // First extract scale to normalize the rotation part
    let scale = mat4x4_extract_scale(matrix);
    
    // Normalize the rotation matrix by dividing by scale
    let r00 = matrix[0] / scale[0];
    let r01 = matrix[1] / scale[0];
    let r02 = matrix[2] / scale[0];
    let _r10 = matrix[4] / scale[1];
    let r11 = matrix[5] / scale[1];
    let _r12 = matrix[6] / scale[1];
    let r20 = matrix[8] / scale[2];
    let r21 = matrix[9] / scale[2];
    let r22 = matrix[10] / scale[2];
    
    // Extract Euler angles (YXZ order)
    let pitch = (-r21).asin().clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    let yaw = if pitch.cos().abs() > 0.0001 {
        r20.atan2(r22)
    } else {
        r02.atan2(r00)
    };
    let roll = if pitch.cos().abs() > 0.0001 {
        r01.atan2(r11)
    } else {
        0.0
    };
    
    [pitch, yaw, roll]
}

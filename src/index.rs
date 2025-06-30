use glow::HasContext;
use std::collections::{ HashMap, HashSet };
use gltf::{ animation::Property, buffer::Data };
use std::error::Error;

// === MATH HELPERS ===

type mat4x4 = [f32; 16];

fn mat4x4_rot_y(angle_radians: f32) -> mat4x4 {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    [cos, 0.0, -sin, 0.0, 0.0, 1.0, 0.0, 0.0, sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_translate(x: f32, y: f32, z: f32) -> mat4x4 {
    [1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_mul(a: mat4x4, b: mat4x4) -> mat4x4 {
    let mut result = [0.0; 16];
    for i in 0..16 {
        let row = i / 4;
        let col = i % 4;
        for k in 0..4 {
            result[i] += a[row * 4 + k] * b[k * 4 + col];
        }
    }
    result
}

fn mat4x4_perspective(near: f32, far: f32) -> mat4x4 {
    let a = -far / (far - near);
    let b = (-far * near) / (far - near);
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, a, b, 0.0, 0.0, -1.0, 0.0]
}

fn mat4x4_from_transform(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> mat4x4 {
    // Convert quaternion to rotation matrix
    let [x, y, z, w] = rotation;
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;

    // Create transformation matrix with scale, rotation, and translation
    [
        scale[0] * (1.0 - (yy + zz)),
        scale[0] * (xy - wz),
        scale[0] * (xz + wy),
        translation[0],
        scale[1] * (xy + wz),
        scale[1] * (1.0 - (xx + zz)),
        scale[1] * (yz - wx),
        translation[1],
        scale[2] * (xz - wy),
        scale[2] * (yz + wx),
        scale[2] * (1.0 - (xx + yy)),
        translation[2],
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

fn mat4x4_identity() -> mat4x4 {
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn mat4x4_extract_translation(matrix: &mat4x4) -> [f32; 3] {
    [matrix[3], matrix[7], matrix[11]]
}

fn mat4x4_extract_rotation_direction(matrix: &mat4x4) -> [f32; 3] {
    // Extract the forward direction (Z-axis) from the rotation matrix
    // This gives us the direction the bone is pointing
    [matrix[2], matrix[6], matrix[10]]
}

fn normalize_vector(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.001 {
        [v[0] / length, v[1] / length, v[2] / length]
    } else {
        [0.0, 1.0, 0.0] // Default to up direction
    }
}

fn mat4x4_transform_direction(matrix: &mat4x4, direction: [f32; 3]) -> [f32; 3] {
    // Transform direction vector (w=0) by the rotation part of the matrix
    [
        matrix[0] * direction[0] + matrix[1] * direction[1] + matrix[2] * direction[2],
        matrix[4] * direction[0] + matrix[5] * direction[1] + matrix[6] * direction[2],
        matrix[8] * direction[0] + matrix[9] * direction[1] + matrix[10] * direction[2],
    ]
}

// === SKELETON DATA STRUCTURES ===

#[derive(Debug, Clone)]
pub struct Bone {
    pub start_pos: [f32; 3], // World position of bone start
    pub end_pos: [f32; 3], // World position of bone end
    pub node_index: usize, // Reference to GLTF node
    pub parent_index: Option<usize>, // Parent bone index
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub root_bones: Vec<usize>, // Indices of root bones
}

// === MESH PARSING ===

pub fn parse_joints_and_weights() -> Result<
    (Vec<[u16; 4]>, Vec<[f32; 4]>),
    Box<dyn std::error::Error>
> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");
    let buffer_data = include_bytes!("assets/meshes/guy.bin");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;
    let mesh = document.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive in mesh")?;
    let buffers = vec![gltf::buffer::Data(buffer_data.to_vec())];

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let joints: Vec<[u16; 4]> = if let Some(joints_reader) = reader.read_joints(0) {
        match joints_reader {
            gltf::mesh::util::ReadJoints::U8(iter) => {
                iter.map(|j| [j[0] as u16, j[1] as u16, j[2] as u16, j[3] as u16]).collect()
            }
            gltf::mesh::util::ReadJoints::U16(iter) => {
                iter.collect()
            }
        }
    } else {
        return Err("No joints attribute found".into());
    };

    let weights: Vec<[f32; 4]> = if let Some(weights_reader) = reader.read_weights(0) {
        match weights_reader {
            gltf::mesh::util::ReadWeights::U8(iter) => {
                iter.map(|w| [w[0] as f32, w[1] as f32, w[2] as f32, w[3] as f32]).collect()
            }
            gltf::mesh::util::ReadWeights::U16(iter) => {
                iter.map(|w| [w[0] as f32, w[1] as f32, w[2] as f32, w[3] as f32]).collect()
            }
            gltf::mesh::util::ReadWeights::F32(iter) => {
                iter.collect()
            }
        }
    } else {
        return Err("No weights attribute found".into());
    };

    Ok((joints, weights))
}

#[derive(Debug)]
pub struct ParsedAnimationChannel {
    pub target_node: usize,
    pub transform_type: String,
    pub timestamps_count: usize,
    pub timestamps: Vec<f32>,
    pub transforms: Vec<Vec<f32>>, // Either vec3 or vec4
}

pub fn parse_animation_channels() -> Result<Vec<ParsedAnimationChannel>, Box<dyn Error>> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");
    let buffer_data = include_bytes!("assets/meshes/guy.bin");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;
    let buffers = vec![Data(buffer_data.to_vec())];

    // Extract first animation
    let animation = document.animations().next().ok_or("No animation found")?;
    let mut parsed_channels = Vec::new();

    // Extract all channels from this animation
    for channel in animation.channels() {
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

        let target = channel.target();
        let target_node = target.node().index();
        let property = target.property();
        let transform_type = format!("{:?}", property).to_lowercase();

        // Extract timestamps
        let timestamps: Vec<f32> = reader
            .read_inputs()
            .ok_or("Missing input timestamps")?
            .collect();

        let timestamps_count = timestamps.len();

        // Extract real transform data from GLTF animation
        let transforms: Vec<Vec<f32>> = {
            if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translations) => {
                        translations.map(|t| vec![t[0], t[1], t[2]]).collect()
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        rotations
                            .into_f32()
                            .map(|r| vec![r[0], r[1], r[2], r[3]])
                            .collect()
                    }
                    gltf::animation::util::ReadOutputs::Scales(scales) => {
                        scales.map(|s| vec![s[0], s[1], s[2]]).collect()
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(weights) => {
                        weights
                            .into_f32()
                            .map(|w| vec![w])
                            .collect()
                    }
                }
            } else {
                // Fallback to dummy data if reading fails
                match property {
                    Property::Translation => {
                        (0..timestamps_count).map(|_| vec![0.0, 0.0, 0.0]).collect()
                    }
                    Property::Scale => {
                        (0..timestamps_count).map(|_| vec![1.0, 1.0, 1.0]).collect()
                    }
                    Property::Rotation => {
                        (0..timestamps_count).map(|_| vec![0.0, 0.0, 0.0, 1.0]).collect()
                    }
                    Property::MorphTargetWeights => {
                        (0..timestamps_count).map(|_| vec![0.0]).collect()
                    }
                    _ => {
                        return Err("Unexpected animation property type".into());
                    }
                }
            }
        };

        parsed_channels.push(ParsedAnimationChannel {
            target_node,
            transform_type,
            timestamps_count,
            timestamps,
            transforms,
        });
    }

    Ok(parsed_channels)
}

pub fn parse_mesh() -> Result<
    (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u16; 4]>, Vec<[f32; 4]>, Vec<u32>),
    Box<dyn std::error::Error>
> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");
    let buffer_data = include_bytes!("assets/meshes/guy.bin");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;

    let mesh = document.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive in mesh")?;

    let buffers = vec![gltf::buffer::Data(buffer_data.to_vec())];

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or("No position attribute")?
        .collect();
    let normals: Vec<[f32; 3]> = reader.read_normals().ok_or("No normal attribute")?.collect();
    
    // Parse joint data for skeletal animation
    let joints: Vec<[u16; 4]> = reader
        .read_joints(0)
        .ok_or("No joint attribute")?
        .into_u16()
        .collect();
    
    // Parse weight data for skeletal animation
    let weights: Vec<[f32; 4]> = if let Some(weights_reader) = reader.read_weights(0) {
        match weights_reader {
            gltf::mesh::util::ReadWeights::U8(iter) => {
                iter.map(|w| [w[0] as f32 / 255.0, w[1] as f32 / 255.0, w[2] as f32 / 255.0, w[3] as f32 / 255.0]).collect()
            }
            gltf::mesh::util::ReadWeights::U16(iter) => {
                iter.map(|w| [w[0] as f32 / 65535.0, w[1] as f32 / 65535.0, w[2] as f32 / 65535.0, w[3] as f32 / 65535.0]).collect()
            }
            gltf::mesh::util::ReadWeights::F32(iter) => {
                iter.collect()
            }
        }
    } else {
        return Err("No weights attribute found".into());
    };
    
    let indices: Vec<u32> = reader.read_indices().ok_or("No indices found")?.into_u32().collect();

    println!("🦴 Parsed mesh with {} vertices, {} joints per vertex, {} weights per vertex", positions.len(), 4, 4);
    println!("🦴 First vertex joints: {:?}", joints.get(0));
    println!("🦴 First vertex weights: {:?}", weights.get(0));

    Ok((positions, normals, joints, weights, indices))
}

pub fn parse_nodes() -> Result<Skeleton, Box<dyn std::error::Error>> {
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");

    let gltf = gltf::Gltf::from_slice(gltf_data)?;
    let document = gltf.document;

    // Get skeleton joints from skin definition
    let joint_indices = get_skeleton_joints(&document)?;

    println!("🦴 Found skeleton with {} joints: {:?}", joint_indices.len(), joint_indices);

    // Build parent-child relationships for ALL nodes
    let mut parents: HashMap<usize, usize> = HashMap::new();

    for scene in document.scenes() {
        for root_node in scene.nodes() {
            build_parent_map(root_node, None, &mut parents);
        }
    }

    // Calculate world transforms for all nodes
    let world_transforms = calculate_world_transforms(&document, &parents);

    // Generate bones ONLY from skeleton joints
    let bones = generate_skeleton_bones(&document, &world_transforms, &parents, &joint_indices);

    // Find root bones (bones without parents in the skeleton)
    let root_bones: Vec<usize> = bones
        .iter()
        .enumerate()
        .filter_map(|(i, bone)| if bone.parent_index.is_none() { Some(i) } else { None })
        .collect();

    println!("🦴 Generated {} skeleton bones from {} joints", bones.len(), joint_indices.len());
    for (i, bone) in bones.iter().enumerate() {
        let node = document.nodes().nth(bone.node_index).unwrap();
        let node_name = node.name().unwrap_or("unnamed");
        println!(
            "  Bone {}: {} (Node {}) -> [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
            i,
            node_name,
            bone.node_index,
            bone.start_pos[0],
            bone.start_pos[1],
            bone.start_pos[2],
            bone.end_pos[0],
            bone.end_pos[1],
            bone.end_pos[2]
        );
    }

    Ok(Skeleton { bones, root_bones })
}

fn build_parent_map(
    node: gltf::Node,
    parent_index: Option<usize>,
    parents: &mut HashMap<usize, usize>
) {
    if let Some(parent) = parent_index {
        parents.insert(node.index(), parent);
    }

    for child in node.children() {
        build_parent_map(child, Some(node.index()), parents);
    }
}

fn calculate_world_transforms(
    document: &gltf::Document,
    parents: &HashMap<usize, usize>
) -> HashMap<usize, mat4x4> {
    let mut world_transforms: HashMap<usize, mat4x4> = HashMap::new();

    // Calculate transforms recursively, starting from root nodes
    for node in document.nodes() {
        if !parents.contains_key(&node.index()) {
            // This is a root node
            calculate_node_world_transform(node, &mat4x4_identity(), &mut world_transforms);
        }
    }

    world_transforms
}

fn calculate_node_world_transform(
    node: gltf::Node,
    parent_world_transform: &mat4x4,
    world_transforms: &mut HashMap<usize, mat4x4>
) {
    // Get local transform
    let (translation, rotation, scale) = node.transform().decomposed();
    let local_transform = mat4x4_from_transform(translation, rotation, scale);

    // Calculate world transform
    let world_transform = mat4x4_mul(*parent_world_transform, local_transform);
    world_transforms.insert(node.index(), world_transform);

    // Recursively calculate for children
    for child in node.children() {
        calculate_node_world_transform(child, &world_transform, world_transforms);
    }
}

fn generate_bones_from_nodes(
    document: &gltf::Document,
    world_transforms: &HashMap<usize, mat4x4>,
    parents: &HashMap<usize, usize>
) -> Vec<Bone> {
    let mut bones = Vec::new();

    for node in document.nodes() {
        if let Some(parent_index) = parents.get(&node.index()) {
            // This node has a parent, so create a bone from parent to this node
            if
                let (Some(parent_transform), Some(child_transform)) = (
                    world_transforms.get(parent_index),
                    world_transforms.get(&node.index()),
                )
            {
                let start_pos = mat4x4_extract_translation(parent_transform);
                let end_pos = mat4x4_extract_translation(child_transform);

                bones.push(Bone {
                    start_pos,
                    end_pos,
                    node_index: node.index(),
                    parent_index: Some(*parent_index),
                });
            }
        }
    }

    bones
}

fn get_skeleton_joints(
    document: &gltf::Document
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    // Get the first skin (assuming single skeleton)
    let skin = document.skins().next().ok_or("No skin found in GLTF")?;

    // Extract joint indices from the skin
    let joint_indices: Vec<usize> = skin
        .joints()
        .map(|joint| joint.index())
        .collect();

    Ok(joint_indices)
}

fn generate_skeleton_bones(
    document: &gltf::Document,
    world_transforms: &HashMap<usize, mat4x4>,
    parents: &HashMap<usize, usize>,
    joint_indices: &[usize]
) -> Vec<Bone> {
    let mut bones = Vec::new();

    // Create one bone per joint (child-agnostic approach)
    for &joint_index in joint_indices {
        if let Some(joint_transform) = world_transforms.get(&joint_index) {
            let joint_pos = mat4x4_extract_translation(joint_transform);

            // Create fixed-length bone (2.5 units) from this joint
            // Bone extends in the joint's local Y direction
            let fixed_bone_vector = [0.0, 2.5, 0.0];
            let transformed_vector = mat4x4_transform_direction(joint_transform, fixed_bone_vector);

            let bone_end_pos = [
                joint_pos[0] + transformed_vector[0],
                joint_pos[1] + transformed_vector[1],
                joint_pos[2] + transformed_vector[2],
            ];

            bones.push(Bone {
                start_pos: joint_pos,
                end_pos: bone_end_pos,
                node_index: joint_index,
                parent_index: parents.get(&joint_index).copied(),
            });
        }
    }

    bones
}

fn skeleton_to_line_vertices(skeleton: &Skeleton) -> Vec<f32> {
    let mut vertices = Vec::new();

    for bone in &skeleton.bones {
        // Add start position
        vertices.extend_from_slice(&bone.start_pos);
        // Add end position
        vertices.extend_from_slice(&bone.end_pos);
    }

    vertices
}

#[derive(Debug, Clone)]
pub struct NodeTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn slerp(q1: [f32; 4], q2: [f32; 4], t: f32) -> [f32; 4] {
    let mut q1 = q1;
    let mut q2 = q2;
    let mut dot = q1
        .iter()
        .zip(q2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    if dot < 0.0 {
        dot = -dot;
        q2 = [-q2[0], -q2[1], -q2[2], -q2[3]];
    }

    if dot > 0.9995 {
        let result: Vec<f32> = q1
            .iter()
            .zip(q2.iter())
            .map(|(a, b)| lerp(*a, *b, t))
            .collect();
        let norm = result
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        return [result[0] / norm, result[1] / norm, result[2] / norm, result[3] / norm];
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();

    let s0 = (theta_0 - theta).sin() / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    [
        s0 * q1[0] + s1 * q2[0],
        s0 * q1[1] + s1 * q2[1],
        s0 * q1[2] + s1 * q2[2],
        s0 * q1[3] + s1 * q2[3],
    ]
}

fn update_skeleton_from_transforms(
    skeleton: &mut Skeleton,
    node_transforms: &HashMap<usize, NodeTransform>
) {
    // Parse GLTF to get base structure and parent relationships
    let gltf_data = include_bytes!("assets/meshes/guy.gltf");
    let gltf = gltf::Gltf::from_slice(gltf_data).unwrap();
    let document = gltf.document;

    // Build parent relationships
    let mut parents: HashMap<usize, usize> = HashMap::new();
    for scene in document.scenes() {
        for root_node in scene.nodes() {
            build_parent_map(root_node, None, &mut parents);
        }
    }

    // Calculate animated world transforms for all joints in hierarchical order
    let animated_world_transforms = calculate_animated_world_transforms(
        &document,
        &parents,
        node_transforms
    );

    // Update skeleton bones based on animated joint positions
    update_bones_from_joint_transforms(skeleton, &animated_world_transforms, &parents);
}

fn calculate_animated_world_transforms(
    document: &gltf::Document,
    parents: &HashMap<usize, usize>,
    node_transforms: &HashMap<usize, NodeTransform>
) -> HashMap<usize, mat4x4> {
    let mut world_transforms: HashMap<usize, mat4x4> = HashMap::new();

    // Process nodes in hierarchical order (parents before children)
    for node in document.nodes() {
        if !parents.contains_key(&node.index()) {
            // This is a root node
            calculate_animated_node_world_transform(
                node,
                &mat4x4_identity(),
                &mut world_transforms,
                node_transforms
            );
        }
    }

    world_transforms
}

fn calculate_animated_node_world_transform(
    node: gltf::Node,
    parent_world_transform: &mat4x4,
    world_transforms: &mut HashMap<usize, mat4x4>,
    node_transforms: &HashMap<usize, NodeTransform>
) {
    // Get local transform - use animated transform if available, otherwise use base transform
    let local_transform = if let Some(animated_transform) = node_transforms.get(&node.index()) {
        mat4x4_from_transform(
            animated_transform.translation,
            animated_transform.rotation,
            animated_transform.scale
        )
    } else {
        // Use base transform from GLTF
        let (translation, rotation, scale) = node.transform().decomposed();
        mat4x4_from_transform(translation, rotation, scale)
    };

    // Calculate world transform by combining parent world transform with local transform
    let world_transform = mat4x4_mul(*parent_world_transform, local_transform);
    world_transforms.insert(node.index(), world_transform);

    // Recursively calculate for children
    for child in node.children() {
        calculate_animated_node_world_transform(
            child,
            &world_transform,
            world_transforms,
            node_transforms
        );
    }
}

fn update_bones_from_joint_transforms(
    skeleton: &mut Skeleton,
    world_transforms: &HashMap<usize, mat4x4>,
    _parents: &HashMap<usize, usize>
) {
    // Update each bone based on the animated joint positions
    // Each joint gets its own fixed-length bone (child-agnostic)
    for bone in &mut skeleton.bones {
        let joint_index = bone.node_index;

        if let Some(joint_transform) = world_transforms.get(&joint_index) {
            let joint_pos = mat4x4_extract_translation(joint_transform);

            // Create fixed-length bone (2.5 units) from this joint
            // Bone extends in the joint's local Y direction
            let fixed_bone_vector = [0.0, 2.5, 0.0];
            let transformed_vector = mat4x4_transform_direction(joint_transform, fixed_bone_vector);

            bone.start_pos = joint_pos;
            bone.end_pos = [
                joint_pos[0] + transformed_vector[0],
                joint_pos[1] + transformed_vector[1],
                joint_pos[2] + transformed_vector[2],
            ];
        }
    }
}

fn has_real_animation_data(channels: &[ParsedAnimationChannel]) -> bool {
    // Check if any channel has non-zero/non-identity transforms
    for channel in channels {
        for transform in &channel.transforms {
            match channel.transform_type.as_str() {
                "translation" => {
                    // Check if any translation is non-zero
                    if
                        transform.len() >= 3 &&
                        (transform[0] != 0.0 || transform[1] != 0.0 || transform[2] != 0.0)
                    {
                        return true;
                    }
                }
                "rotation" => {
                    // Check if any rotation is not identity quaternion [0,0,0,1]
                    if
                        transform.len() >= 4 &&
                        (transform[0] != 0.0 ||
                            transform[1] != 0.0 ||
                            transform[2] != 0.0 ||
                            transform[3] != 1.0)
                    {
                        return true;
                    }
                }
                "scale" => {
                    // Check if any scale is not identity [1,1,1]
                    if
                        transform.len() >= 3 &&
                        (transform[0] != 1.0 || transform[1] != 1.0 || transform[2] != 1.0)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn apply_animation(
    time_since_start: f32,
    animation_channels: &[ParsedAnimationChannel],
    node_transforms: &mut HashMap<usize, NodeTransform>
) {
    for channel in animation_channels {
        if channel.timestamps.is_empty() || channel.transforms.is_empty() {
            continue;
        }

        let duration = *channel.timestamps.last().unwrap();
        let rel_time = time_since_start % duration;

        let mut last_index = 0;
        for (i, &timestamp) in channel.timestamps.iter().enumerate() {
            if rel_time >= timestamp {
                last_index = i;
            } else {
                break;
            }
        }

        let next_index = if last_index + 1 < channel.timestamps.len() {
            last_index + 1
        } else {
            last_index
        };

        let t0 = channel.timestamps[last_index];
        let t1 = channel.timestamps[next_index];
        let delta = if t1 - t0 > 0.0 { t1 - t0 } else { 1.0 };
        let alpha = (rel_time - t0) / delta;

        let transform0 = &channel.transforms[last_index];
        let transform1 = &channel.transforms[next_index];

        let node = node_transforms.entry(channel.target_node).or_insert_with(|| NodeTransform {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        });

        match channel.transform_type.as_str() {
            "translation" => {
                for i in 0..3 {
                    node.translation[i] = lerp(transform0[i], transform1[i], alpha);
                }
            }
            "scale" => {
                for i in 0..3 {
                    node.scale[i] = lerp(transform0[i], transform1[i], alpha);
                }
            }
            "rotation" => {
                let q0 = [transform0[0], transform0[1], transform0[2], transform0[3]];
                let q1 = [transform1[0], transform1[1], transform1[2], transform1[3]];
                node.rotation = slerp(q0, q1, alpha);
            }
            _ => {}
        }
    }
}

// === SHADER COMPILATION ===

fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    mut source: String
) -> Result<glow::Shader, String> {
    unsafe {
        let is_web = cfg!(target_arch = "wasm32");
        if is_web {
            source = source.replace("#VERSION", "#version 300 es\nprecision mediump float;");
        } else {
            source = source.replace("#VERSION", "#version 330 core");
        }

        let shader = gl.create_shader(shader_type)?;
        gl.shader_source(shader, &source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(format!("Shader compile error: {}", log));
        }
        Ok(shader)
    }
}

// === BUFFER SETUP ===

fn setup_buffers_with_joints_and_weights(
    gl: &glow::Context,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    joints: &[[u16; 4]],
    weights: &[[f32; 4]],
    indices: &[u32]
) -> Result<(glow::VertexArray, glow::Buffer, glow::Buffer, glow::Buffer, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let main_vbo = gl.create_buffer()?;
        let joints_vbo = gl.create_buffer()?;
        let ebo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));

        // Buffer 1: Positions + Normals (all f32)
        let mut main_vertex_data = Vec::new();
        for i in 0..positions.len() {
            main_vertex_data.extend_from_slice(&positions[i]);  // 3 f32
            main_vertex_data.extend_from_slice(&normals[i]);    // 3 f32
        }

        // Buffer 2: Joints only (all u16)
        let joints_data: Vec<u16> = joints.iter().flatten().copied().collect();

        // Buffer 3: Weights only (all f32)
        let weights_data: Vec<f32> = weights.iter().flatten().copied().collect();

        // Setup main buffer (positions + normals)
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(main_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&main_vertex_data),
            glow::STATIC_DRAW
        );

        let main_stride = 6 * (std::mem::size_of::<f32>() as i32); // 3 pos + 3 normal
        // Location 0: positions
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, main_stride, 0);
        gl.enable_vertex_attrib_array(0);
        // Location 1: normals  
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, main_stride, 3 * 4);
        gl.enable_vertex_attrib_array(1);

        // Setup joints buffer
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(joints_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&joints_data),
            glow::STATIC_DRAW
        );

        // Location 2: joints (4 u16 = 8 bytes stride)
        gl.vertex_attrib_pointer_i32(2, 4, glow::UNSIGNED_SHORT, 8, 0);
        gl.enable_vertex_attrib_array(2);

        // Setup weights buffer
        let weights_vbo = gl.create_buffer()?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(weights_vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&weights_data),
            glow::STATIC_DRAW
        );

        // Location 3: weights (4 f32 = 16 bytes stride)
        gl.vertex_attrib_pointer_f32(3, 4, glow::FLOAT, false, 16, 0);
        gl.enable_vertex_attrib_array(3);

        // Setup index buffer
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(indices),
            glow::STATIC_DRAW
        );

        gl.bind_vertex_array(None);
        Ok((vao, main_vbo, joints_vbo, weights_vbo, ebo))
    }
}

fn setup_buffers(
    gl: &glow::Context,
    vertices: &[f32],
    indices: &[u32]
) -> Result<(glow::VertexArray, glow::Buffer, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;
        let ebo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW
        );
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(indices),
            glow::STATIC_DRAW
        );

        let stride = 6 * (std::mem::size_of::<f32>() as i32);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 3 * 4);
        gl.enable_vertex_attrib_array(1);

        gl.bind_vertex_array(None);
        Ok((vao, vbo, ebo))
    }
}

fn setup_line_buffers(
    gl: &glow::Context,
    vertices: &[f32]
) -> Result<(glow::VertexArray, glow::Buffer), String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(vertices),
            glow::STATIC_DRAW
        );
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.bind_vertex_array(None);
        Ok((vao, vbo))
    }
}

// === PROGRAM ===

pub struct Program {
    gl: glow::Context,
    shader_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    joints_vbo: glow::Buffer, // Separate buffer for joint data
    weights_vbo: glow::Buffer, // Separate buffer for weight data
    ebo: glow::Buffer,
    index_count: i32,
    line_vao: glow::VertexArray,
    line_vbo: glow::Buffer,
    original_skeleton: Skeleton, // Keep the working skeleton
    skeleton: Skeleton, // Current skeleton (may be animated)
    bone_count: i32,
    animation_channels: Vec<ParsedAnimationChannel>,
    node_transforms: HashMap<usize, NodeTransform>,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        let (positions, normals, joints, weights, indices) = parse_mesh().map_err(|e|
            format!("Failed to parse mesh: {}", e)
        )?;

        let original_skeleton = parse_nodes().map_err(|e|
            format!("Failed to parse skeleton: {}", e)
        )?;

        let animation_channels = match parse_animation_channels() {
            Ok(channels) => channels,
            Err(e) => {
                println!("⚠️ Failed to parse animation: {}", e);
                Vec::new()
            }
        };

        // Check if we have real animation data
        let has_real_animation = has_real_animation_data(&animation_channels);
        println!("🎬 Animation data detected: {}", if has_real_animation {
            "REAL"
        } else {
            "DUMMY"
        });

        let mut node_transforms: HashMap<usize, NodeTransform> = HashMap::new();
        let mut skeleton = original_skeleton.clone();

        // Only apply animation if we have real data
        if has_real_animation {
            apply_animation(0.0, &animation_channels, &mut node_transforms);
            update_skeleton_from_transforms(&mut skeleton, &node_transforms);
        }
        // Otherwise, keep the original skeleton unchanged

        // Create vertices from positions and normals for mesh rendering
        let vertices: Vec<f32> = positions
            .iter()
            .zip(normals.iter())
            .flat_map(|(p, n)| [p[0], p[1], p[2], n[0], n[1], n[2]])
            .collect();

        let line_vertices = skeleton_to_line_vertices(&skeleton);

        let bone_count = skeleton.bones.len() as i32;

        unsafe {
            let vs_src = include_str!("assets/shaders/vertex.glsl");
            let fs_src = include_str!("assets/shaders/fragment.glsl");

            let vs = compile_shader(&gl, glow::VERTEX_SHADER, vs_src.to_string())?;
            let fs = compile_shader(&gl, glow::FRAGMENT_SHADER, fs_src.to_string())?;

            let program = gl.create_program()?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.bind_attrib_location(program, 0, "vPos");
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                return Err(format!("Program link error: {}", log));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);
            gl.use_program(Some(program));
            gl.enable(glow::DEPTH_TEST);

            let (vao, vbo, joints_vbo, weights_vbo, ebo) = setup_buffers_with_joints_and_weights(&gl, &positions, &normals, &joints, &weights, &indices)?;
            let (line_vao, line_vbo) = setup_line_buffers(&gl, &line_vertices)?;

            Ok(Self {
                gl,
                shader_program: program,
                vao,
                vbo,
                joints_vbo,
                weights_vbo,
                ebo,
                index_count: indices.len() as i32,
                line_vao,
                line_vbo,
                original_skeleton,
                skeleton,
                bone_count,
                animation_channels,
                node_transforms,
            })
        }
    }

    pub fn render(&mut self, width: u32, height: u32, delta_time: f32) -> Result<(), String> {
        // Only apply animation if we have real animation data
        if has_real_animation_data(&self.animation_channels) {
            apply_animation(delta_time, &self.animation_channels, &mut self.node_transforms);
            update_skeleton_from_transforms(&mut self.skeleton, &self.node_transforms);

            // Update line vertices with animated skeleton
            let line_vertices = skeleton_to_line_vertices(&self.skeleton);
            unsafe {
                self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.line_vbo));
                self.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&line_vertices),
                    glow::DYNAMIC_DRAW
                );
            }
        } else {
            // Use original skeleton - no animation updates needed
            // The line buffer already contains the original skeleton vertices
        }

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));

            let angle = delta_time.rem_euclid(std::f32::consts::TAU);
            let world = mat4x4_mul(mat4x4_translate(0.0, 0.0, -6.0), mat4x4_rot_y(angle));
            let view = mat4x4_perspective(0.1, 10.0);

            if let Some(u) = self.gl.get_uniform_location(self.shader_program, "world_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u), true, &world);
            }
            if let Some(u) = self.gl.get_uniform_location(self.shader_program, "viewport_txfm") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u), true, &view);
            }
            if let Some(u) = self.gl.get_uniform_location(self.shader_program, "preview_joint") {
                self.gl.uniform_1_u32(Some(&u), 1); // Set preview joint to 0
            }

            // Render skeleton lines (no joints attribute needed)
            self.gl.line_width(8.0);
            self.gl.bind_vertex_array(Some(self.line_vao));
            self.gl.draw_arrays(glow::LINES, 0, self.bone_count * 2);
            self.gl.bind_vertex_array(None);

            // Render the mesh with joints
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        unsafe {
            self.gl.delete_program(self.shader_program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.joints_vbo);
            self.gl.delete_buffer(self.weights_vbo);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_vertex_array(self.line_vao);
            self.gl.delete_buffer(self.line_vbo);
        }
    }
}

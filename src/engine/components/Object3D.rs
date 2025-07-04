// Import components
mod transform_mod {
    include!("Tranform.rs");
}
mod mesh_mod {
    include!("Mesh.rs");
}

pub use transform_mod::Transform;
pub use mesh_mod::Mesh;

// Optional components for skeletal animation
#[derive(Debug, Clone)]
pub struct Node {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub parent: u32,
}

pub struct Skeleton {
    pub nodes: Vec<Node>,
    pub joint_ids: Vec<u32>,
    pub joint_inverse_mats: Vec<[f32; 16]>,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Translation = 0,
    Rotation = 1,
    Scale = 2,
}

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub target: u32,
    pub animation_type: AnimationType,
    pub num_timesteps: usize,
    pub times: Vec<f32>,
    pub data: Vec<f32>,
}

impl AnimationChannel {
    pub fn components(&self) -> usize {
        match self.animation_type {
            AnimationType::Translation | AnimationType::Scale => 3,
            AnimationType::Rotation => 4,
        }
    }
}

// Main Object3D struct
pub struct Object3D {
    pub transform: Transform,
    pub mesh: Mesh,
    pub skeleton: Option<Skeleton>,
    pub animation_channels: Vec<AnimationChannel>,
}

impl Object3D {
    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
            mesh: Mesh::new(),
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    pub fn with_mesh(mesh: Mesh) -> Self {
        Self {
            transform: Transform::new(),
            mesh,
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    pub fn with_transform_and_mesh(transform: Transform, mesh: Mesh) -> Self {
        Self {
            transform,
            mesh,
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    pub fn set_skeleton(&mut self, skeleton: Skeleton) {
        self.skeleton = Some(skeleton);
    }

    pub fn set_animation_channels(&mut self, channels: Vec<AnimationChannel>) {
        self.animation_channels = channels;
    }

    pub fn add_animation_channel(&mut self, channel: AnimationChannel) {
        self.animation_channels.push(channel);
    }

    pub fn has_skeleton(&self) -> bool {
        self.skeleton.is_some()
    }

    pub fn has_animations(&self) -> bool {
        !self.animation_channels.is_empty()
    }

    pub fn get_transform_matrix(&mut self) -> [f32; 16] {
        self.transform.get_matrix()
    }

    pub fn is_renderable(&self) -> bool {
        self.mesh.is_valid()
    }
}

impl Default for Object3D {
    fn default() -> Self {
        Self::new()
    }
}

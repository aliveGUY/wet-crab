// Import components
mod transform_mod {
    include!("Tranform.rs");
}
mod mesh_mod {
    include!("Mesh.rs");
}
mod material_mod {
    include!("Material.rs");
}
mod skeleton_mod {
    include!("Skeleton.rs");
}
mod animation_mod {
    include!("AnimationState.rs");
}

pub use transform_mod::Transform;
pub use mesh_mod::Mesh;
pub use material_mod::Material;
pub use skeleton_mod::*;
pub use animation_mod::*;

#[derive(Clone)]
pub struct Object3D {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Option<Material>,
    pub skeleton: Option<Skeleton>,
    pub animation_channels: Vec<AnimationChannel>,
}

impl Object3D {
    pub fn new() -> Self {
        Self {
            transform: Transform::new(),
            mesh: Mesh::new(),
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    pub fn with_mesh(mesh: Mesh) -> Self {
        Self {
            transform: Transform::new(),
            mesh,
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_transform_and_mesh(transform: Transform, mesh: Mesh) -> Self {
        Self {
            transform,
            mesh,
            material: None,
            skeleton: None,
            animation_channels: Vec::new(),
        }
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = Some(material);
    }

    #[allow(dead_code)]
    pub fn has_material(&self) -> bool {
        self.material.is_some()
    }

    pub fn set_skeleton(&mut self, skeleton: Skeleton) {
        self.skeleton = Some(skeleton);
    }

    pub fn set_animation_channels(&mut self, channels: Vec<AnimationChannel>) {
        self.animation_channels = channels;
    }

    #[allow(dead_code)]
    pub fn add_animation_channel(&mut self, channel: AnimationChannel) {
        self.animation_channels.push(channel);
    }

    #[allow(dead_code)]
    pub fn has_skeleton(&self) -> bool {
        self.skeleton.is_some()
    }

    #[allow(dead_code)]
    pub fn has_animations(&self) -> bool {
        !self.animation_channels.is_empty()
    }

    pub fn get_transform_matrix(&mut self) -> [f32; 16] {
        self.transform.get_matrix()
    }

    #[allow(dead_code)]
    pub fn is_renderable(&self) -> bool {
        self.mesh.is_valid()
    }
}

impl Default for Object3D {
    fn default() -> Self {
        Self::new()
    }
}

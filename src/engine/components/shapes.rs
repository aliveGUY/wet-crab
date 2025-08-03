use serde::{Serialize, Deserialize};

pub type Vec3 = [f32; 3];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Sphere {
        radius: f32,
    },
    Capsule {
        radius: f32,
        height: f32,
    },
    Box {
        half_extents: Vec3,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
}

impl Shape {
    pub fn get_shape_name(&self) -> String {
        match self {
            Shape::Sphere { radius } => format!("Sphere (r: {:.2})", radius),
            Shape::Capsule { radius, height } => format!("Capsule (r: {:.2}, h: {:.2})", radius, height),
            Shape::Box { half_extents } => format!("Box ({:.2}, {:.2}, {:.2})", half_extents[0], half_extents[1], half_extents[2]),
            Shape::Cylinder { radius, height } => format!("Cylinder (r: {:.2}, h: {:.2})", radius, height),
        }
    }
}

use crate::index::engine::{ components::{ Shape, Transform }, Component };
use crate::{ ComponentUI, AttributeUI };
use serde::{ Serialize, Deserialize };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ColliderLayer {
    Environment,
    Player,
}

#[derive(Serialize, Clone, Debug)]
pub struct Collider {
    pub shape: Shape,
    pub is_hidden: bool,
    pub layer: ColliderLayer,
    pub ignored_layers: Vec<ColliderLayer>,
    #[serde(skip)]
    component_ui: Rc<RefCell<ComponentUI>>,
}

// Custom deserialization to properly initialize component UI
impl<'de> Deserialize<'de> for Collider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        use serde::de::{ self, MapAccess, Visitor };
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Shape,
            IsHidden,
            Layer,
            IgnoredLayers,
        }

        struct ColliderVisitor;

        impl<'de> Visitor<'de> for ColliderVisitor {
            type Value = Collider;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Collider")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Collider, V::Error> where V: MapAccess<'de> {
                let mut shape = None;
                let mut is_hidden = None;
                let mut layer = None;
                let mut ignored_layers = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Shape => {
                            if shape.is_some() {
                                return Err(de::Error::duplicate_field("shape"));
                            }
                            shape = Some(map.next_value()?);
                        }
                        Field::IsHidden => {
                            if is_hidden.is_some() {
                                return Err(de::Error::duplicate_field("is_hidden"));
                            }
                            is_hidden = Some(map.next_value()?);
                        }
                        Field::Layer => {
                            if layer.is_some() {
                                return Err(de::Error::duplicate_field("layer"));
                            }
                            layer = Some(map.next_value()?);
                        }
                        Field::IgnoredLayers => {
                            if ignored_layers.is_some() {
                                return Err(de::Error::duplicate_field("ignored_layers"));
                            }
                            ignored_layers = Some(map.next_value()?);
                        }
                    }
                }

                let shape = shape.ok_or_else(|| de::Error::missing_field("shape"))?;
                let is_hidden = is_hidden.unwrap_or(false);
                let layer = layer.ok_or_else(|| de::Error::missing_field("layer"))?;
                let ignored_layers = ignored_layers.unwrap_or_else(Vec::new);

                // Create new collider with proper UI initialization
                let mut collider = Collider::new(shape, layer, ignored_layers);

                // Set the actual is_hidden value from deserialized data
                collider.is_hidden = is_hidden;

                Ok(collider)
            }
        }

        const FIELDS: &'static [&'static str] = &["shape", "is_hidden", "layer", "ignored_layers"];
        deserializer.deserialize_struct("Collider", FIELDS, ColliderVisitor)
    }
}

impl Collider {
    pub fn new(shape: Shape, layer: ColliderLayer, ignored_layers: Vec<ColliderLayer>) -> Self {
        let attributes = Self::create_attributes_for_shape(&shape, false, &layer);

        let component_ui = ComponentUI {
            name: "Collider".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            shape,
            layer,
            ignored_layers,
            is_hidden: false,
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    fn create_attributes_for_shape(
        shape: &Shape,
        is_hidden: bool,
        layer: &ColliderLayer
    ) -> Vec<AttributeUI> {
        let mut attributes = vec![
            AttributeUI {
                name: "Is Hidden".into(),
                dt_type: "BOOLEAN".into(),
                value: is_hidden.to_string().into(),
            },
            AttributeUI {
                name: "Layer".into(),
                dt_type: "STRING".into(),
                value: format!("{:?}", layer).into(),
            }
        ];

        // Add shape-specific attributes
        match shape {
            Shape::Box { half_extents } => {
                attributes.extend(
                    vec![
                        AttributeUI {
                            name: "Shape Type".into(),
                            dt_type: "STRING".into(),
                            value: "Box".into(),
                        },
                        AttributeUI {
                            name: "Half Extent X".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", half_extents[0]).into(),
                        },
                        AttributeUI {
                            name: "Half Extent Y".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", half_extents[1]).into(),
                        },
                        AttributeUI {
                            name: "Half Extent Z".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", half_extents[2]).into(),
                        }
                    ]
                );
            }
            Shape::Sphere { radius } => {
                attributes.extend(
                    vec![
                        AttributeUI {
                            name: "Shape Type".into(),
                            dt_type: "STRING".into(),
                            value: "Sphere".into(),
                        },
                        AttributeUI {
                            name: "Radius".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", radius).into(),
                        }
                    ]
                );
            }
            Shape::Capsule { radius, height } => {
                attributes.extend(
                    vec![
                        AttributeUI {
                            name: "Shape Type".into(),
                            dt_type: "STRING".into(),
                            value: "Capsule".into(),
                        },
                        AttributeUI {
                            name: "Radius".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", radius).into(),
                        },
                        AttributeUI {
                            name: "Height".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", height).into(),
                        }
                    ]
                );
            }
            Shape::Cylinder { radius, height } => {
                attributes.extend(
                    vec![
                        AttributeUI {
                            name: "Shape Type".into(),
                            dt_type: "STRING".into(),
                            value: "Cylinder".into(),
                        },
                        AttributeUI {
                            name: "Radius".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", radius).into(),
                        },
                        AttributeUI {
                            name: "Height".into(),
                            dt_type: "FLOAT".into(),
                            value: format!("{:.3}", height).into(),
                        }
                    ]
                );
            }
        }

        attributes
    }

    pub fn is_collides(self, other: Collider, self_txfm: Transform, other_txfm: Transform) -> bool {
        match (&self.shape, &other.shape) {
            (Shape::Box { .. }, Shape::Box { .. }) =>
                collision_check_box_box(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Box { .. }, Shape::Capsule { .. }) =>
                collision_check_box_capsule(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Box { .. }, Shape::Cylinder { .. }) =>
                collision_check_box_cylinder(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Box { .. }, Shape::Sphere { .. }) =>
                collision_check_box_sphere(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Capsule { .. }, Shape::Box { .. }) =>
                collision_check_box_capsule(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Capsule { .. }, Shape::Capsule { .. }) =>
                collision_check_capsule_capsule(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Capsule { .. }, Shape::Cylinder { .. }) =>
                collision_check_capsule_cylinder(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Capsule { .. }, Shape::Sphere { .. }) =>
                collision_check_capsule_sphere(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Cylinder { .. }, Shape::Box { .. }) =>
                collision_check_box_cylinder(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Cylinder { .. }, Shape::Capsule { .. }) =>
                collision_check_capsule_cylinder(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Cylinder { .. }, Shape::Cylinder { .. }) =>
                collision_check_cylinder_cylinder(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Cylinder { .. }, Shape::Sphere { .. }) =>
                collision_check_cylinder_sphere(self.shape, other.shape, self_txfm, other_txfm),
            (Shape::Sphere { .. }, Shape::Box { .. }) =>
                collision_check_box_sphere(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Sphere { .. }, Shape::Capsule { .. }) =>
                collision_check_capsule_sphere(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Sphere { .. }, Shape::Cylinder { .. }) =>
                collision_check_cylinder_sphere(other.shape, self.shape, other_txfm, self_txfm),
            (Shape::Sphere { .. }, Shape::Sphere { .. }) =>
                collision_check_sphere_sphere(self.shape, other.shape, self_txfm, other_txfm),
        }
    }
}

// ================================================================================================
// COLLISION DETECTION IMPLEMENTATION
// ================================================================================================

use crate::index::engine::utils::math::{
    Vec3, dot, cross, len2, dist2, dist_point_segment2, segment_segment_distance2,
    mat4x4_extract_translation, mat4x4_extract_scale
};

#[derive(Clone)]
struct OBB {
    center: Vec3,
    axes: [Vec3; 3],     // orthonormal axes in world space
    half_extents: Vec3,  // local half-extents * world scale
}

/// Extracts the world-space OBB data from a Box shape and its Transform
fn compute_world_obb(shape: &Shape, txfm: &Transform) -> OBB {
    if let Shape::Box { half_extents } = shape {
        let matrix = txfm.get_matrix();
        
        // World translation
        let center = mat4x4_extract_translation(matrix);
        
        // World scale
        let scale = mat4x4_extract_scale(matrix);
        
        // Extract and normalize world axes from transform matrix
        let axes = [
            {
                let v = [matrix[0], matrix[4], matrix[8]];
                let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
                if len > 1e-8 { [v[0]/len, v[1]/len, v[2]/len] } else { [1.0, 0.0, 0.0] }
            },
            {
                let v = [matrix[1], matrix[5], matrix[9]];
                let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
                if len > 1e-8 { [v[0]/len, v[1]/len, v[2]/len] } else { [0.0, 1.0, 0.0] }
            },
            {
                let v = [matrix[2], matrix[6], matrix[10]];
                let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
                if len > 1e-8 { [v[0]/len, v[1]/len, v[2]/len] } else { [0.0, 0.0, 1.0] }
            },
        ];
        
        // Half-extents scaled by world scale
        let world_he = [
            half_extents[0] * scale[0],
            half_extents[1] * scale[1],
            half_extents[2] * scale[2],
        ];
        
        OBB { center, axes, half_extents: world_he }
    } else {
        panic!("compute_world_obb called on non-box shape");
    }
}

/// 3D separating-axis test for two OBBs using up to 15 axes
fn obb_obb_sat(a: &OBB, b: &OBB) -> bool {
    let mut axes: Vec<Vec3> = Vec::with_capacity(15);
    
    // Add face normals of both OBBs
    axes.extend_from_slice(&a.axes);
    axes.extend_from_slice(&b.axes);
    
    // Add cross products of edge directions (skip if nearly parallel)
    for &u in &a.axes {
        for &v in &b.axes {
            let cr = cross(u, v);
            let len2cr = len2(cr);
            if len2cr > 1e-6 {
                let len = len2cr.sqrt();
                axes.push([cr[0]/len, cr[1]/len, cr[2]/len]);
            }
        }
    }
    
    let d = [b.center[0] - a.center[0], b.center[1] - a.center[1], b.center[2] - a.center[2]];
    
    // Test separation on each axis
    for axis in axes {
        // Project distance between centers
        let dist = dot(d, axis).abs();
        
        // Project extents of A
        let proj_a = a.axes.iter()
            .zip(a.half_extents.iter())
            .map(|(ax, he)| dot(*ax, axis).abs() * he)
            .sum::<f32>();
            
        // Project extents of B
        let proj_b = b.axes.iter()
            .zip(b.half_extents.iter())
            .map(|(ax, he)| dot(*ax, axis).abs() * he)
            .sum::<f32>();
            
        // If separated on this axis, no collision
        if dist > proj_a + proj_b {
            return false;
        }
    }
    
    true // No separating axis found, collision detected
}

fn collision_check_box_box(
    a_shape: Shape,
    b_shape: Shape,
    a_transform: Transform,
    b_transform: Transform
) -> bool {
    let obb_a = compute_world_obb(&a_shape, &a_transform);
    let obb_b = compute_world_obb(&b_shape, &b_transform);
    obb_obb_sat(&obb_a, &obb_b)
}

fn collision_check_sphere_sphere(
    a_shape: Shape,
    b_shape: Shape,
    a_transform: Transform,
    b_transform: Transform
) -> bool {
    if let (Shape::Sphere { radius: ra }, Shape::Sphere { radius: rb }) = (a_shape, b_shape) {
        let ca = mat4x4_extract_translation(a_transform.get_matrix());
        let cb = mat4x4_extract_translation(b_transform.get_matrix());
        let sum_radii = ra + rb;
        
        // Use squared distance to avoid sqrt
        let dist_sq = (cb[0] - ca[0]).powi(2) + (cb[1] - ca[1]).powi(2) + (cb[2] - ca[2]).powi(2);
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

fn collision_check_box_sphere(
    box_shape: Shape,
    sphere_shape: Shape,
    box_transform: Transform,
    sphere_transform: Transform
) -> bool {
    if let (Shape::Box { half_extents }, Shape::Sphere { radius }) = (box_shape, sphere_shape) {
        let obb = compute_world_obb(&Shape::Box { half_extents }, &box_transform);
        let sphere_center = mat4x4_extract_translation(sphere_transform.get_matrix());
        
        // Transform sphere center to box local space
        let to_sphere = [
            sphere_center[0] - obb.center[0],
            sphere_center[1] - obb.center[1],
            sphere_center[2] - obb.center[2]
        ];
        
        // Project to box local coordinates
        let mut local = [
            dot(to_sphere, obb.axes[0]),
            dot(to_sphere, obb.axes[1]),
            dot(to_sphere, obb.axes[2])
        ];
        
        // Clamp to box bounds
        for i in 0..3 {
            local[i] = local[i].max(-obb.half_extents[i]).min(obb.half_extents[i]);
        }
        
        // Transform back to world space
        let closest_point = [
            obb.center[0] + obb.axes[0][0] * local[0] + obb.axes[1][0] * local[1] + obb.axes[2][0] * local[2],
            obb.center[1] + obb.axes[0][1] * local[0] + obb.axes[1][1] * local[1] + obb.axes[2][1] * local[2],
            obb.center[2] + obb.axes[0][2] * local[0] + obb.axes[1][2] * local[1] + obb.axes[2][2] * local[2]
        ];
        
        // Check if distance to closest point is within sphere radius
        let dist_sq = dist2(sphere_center, closest_point);
        dist_sq <= radius * radius
    } else {
        false
    }
}

fn collision_check_capsule_sphere(
    capsule_shape: Shape,
    sphere_shape: Shape,
    capsule_transform: Transform,
    sphere_transform: Transform
) -> bool {
    if let (Shape::Capsule { radius: cap_radius, height }, Shape::Sphere { radius: sphere_radius }) = (capsule_shape, sphere_shape) {
        let cap_center = mat4x4_extract_translation(capsule_transform.get_matrix());
        let cap_scale = mat4x4_extract_scale(capsule_transform.get_matrix());
        let sphere_center = mat4x4_extract_translation(sphere_transform.get_matrix());
        
        // Capsule segment endpoints (assuming Y-axis alignment)
        let half_height = height * 0.5 * cap_scale[1];
        let p0 = [cap_center[0], cap_center[1] - half_height, cap_center[2]];
        let p1 = [cap_center[0], cap_center[1] + half_height, cap_center[2]];
        
        // Distance from sphere center to capsule segment
        let dist_sq = dist_point_segment2(sphere_center, p0, p1);
        let sum_radii = cap_radius + sphere_radius;
        
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

fn collision_check_capsule_capsule(
    a_shape: Shape,
    b_shape: Shape,
    a_transform: Transform,
    b_transform: Transform
) -> bool {
    if let (Shape::Capsule { radius: ra, height: ha }, Shape::Capsule { radius: rb, height: hb }) = (a_shape, b_shape) {
        let ca = mat4x4_extract_translation(a_transform.get_matrix());
        let sa = mat4x4_extract_scale(a_transform.get_matrix());
        let cb = mat4x4_extract_translation(b_transform.get_matrix());
        let sb = mat4x4_extract_scale(b_transform.get_matrix());
        
        // Capsule A segment endpoints
        let half_ha = ha * 0.5 * sa[1];
        let a1 = [ca[0], ca[1] - half_ha, ca[2]];
        let a2 = [ca[0], ca[1] + half_ha, ca[2]];
        
        // Capsule B segment endpoints
        let half_hb = hb * 0.5 * sb[1];
        let b1 = [cb[0], cb[1] - half_hb, cb[2]];
        let b2 = [cb[0], cb[1] + half_hb, cb[2]];
        
        // Segment-segment distance
        let dist_sq = segment_segment_distance2(a1, a2, b1, b2);
        let sum_radii = ra + rb;
        
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

fn collision_check_cylinder_sphere(
    cylinder_shape: Shape,
    sphere_shape: Shape,
    cylinder_transform: Transform,
    sphere_transform: Transform
) -> bool {
    if let (Shape::Cylinder { radius: cyl_radius, height }, Shape::Sphere { radius: sphere_radius }) = (cylinder_shape, sphere_shape) {
        let cyl_center = mat4x4_extract_translation(cylinder_transform.get_matrix());
        let cyl_scale = mat4x4_extract_scale(cylinder_transform.get_matrix());
        let sphere_center = mat4x4_extract_translation(sphere_transform.get_matrix());
        
        // Cylinder segment endpoints (assuming Y-axis alignment)
        let half_height = height * 0.5 * cyl_scale[1];
        let p0 = [cyl_center[0], cyl_center[1] - half_height, cyl_center[2]];
        let p1 = [cyl_center[0], cyl_center[1] + half_height, cyl_center[2]];
        
        // Distance from sphere center to cylinder axis
        let dist_sq = dist_point_segment2(sphere_center, p0, p1);
        let sum_radii = cyl_radius + sphere_radius;
        
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

fn collision_check_box_capsule(
    box_shape: Shape,
    capsule_shape: Shape,
    box_transform: Transform,
    capsule_transform: Transform
) -> bool {
    if let (Shape::Box { half_extents }, Shape::Capsule { radius, height }) = (box_shape, capsule_shape) {
        let obb = compute_world_obb(&Shape::Box { half_extents }, &box_transform);
        let cap_center = mat4x4_extract_translation(capsule_transform.get_matrix());
        let cap_scale = mat4x4_extract_scale(capsule_transform.get_matrix());
        
        // Capsule segment endpoints
        let half_height = height * 0.5 * cap_scale[1];
        let p0 = [cap_center[0], cap_center[1] - half_height, cap_center[2]];
        let p1 = [cap_center[0], cap_center[1] + half_height, cap_center[2]];
        
        // Approximate by checking both endpoints against the OBB
        let dist_sq_0 = {
            let to_p0 = [p0[0] - obb.center[0], p0[1] - obb.center[1], p0[2] - obb.center[2]];
            let mut local = [dot(to_p0, obb.axes[0]), dot(to_p0, obb.axes[1]), dot(to_p0, obb.axes[2])];
            for i in 0..3 {
                local[i] = local[i].max(-obb.half_extents[i]).min(obb.half_extents[i]);
            }
            let closest = [
                obb.center[0] + obb.axes[0][0] * local[0] + obb.axes[1][0] * local[1] + obb.axes[2][0] * local[2],
                obb.center[1] + obb.axes[0][1] * local[0] + obb.axes[1][1] * local[1] + obb.axes[2][1] * local[2],
                obb.center[2] + obb.axes[0][2] * local[0] + obb.axes[1][2] * local[1] + obb.axes[2][2] * local[2]
            ];
            dist2(p0, closest)
        };
        
        let dist_sq_1 = {
            let to_p1 = [p1[0] - obb.center[0], p1[1] - obb.center[1], p1[2] - obb.center[2]];
            let mut local = [dot(to_p1, obb.axes[0]), dot(to_p1, obb.axes[1]), dot(to_p1, obb.axes[2])];
            for i in 0..3 {
                local[i] = local[i].max(-obb.half_extents[i]).min(obb.half_extents[i]);
            }
            let closest = [
                obb.center[0] + obb.axes[0][0] * local[0] + obb.axes[1][0] * local[1] + obb.axes[2][0] * local[2],
                obb.center[1] + obb.axes[0][1] * local[0] + obb.axes[1][1] * local[1] + obb.axes[2][1] * local[2],
                obb.center[2] + obb.axes[0][2] * local[0] + obb.axes[1][2] * local[1] + obb.axes[2][2] * local[2]
            ];
            dist2(p1, closest)
        };
        
        let min_dist_sq = dist_sq_0.min(dist_sq_1);
        min_dist_sq <= radius * radius
    } else {
        false
    }
}

fn collision_check_box_cylinder(
    box_shape: Shape,
    cylinder_shape: Shape,
    box_transform: Transform,
    cylinder_transform: Transform
) -> bool {
    // Treat cylinder like capsule for box collision
    if let (Shape::Box { half_extents }, Shape::Cylinder { radius, height }) = (box_shape, cylinder_shape) {
        collision_check_box_capsule(
            Shape::Box { half_extents },
            Shape::Capsule { radius, height },
            box_transform,
            cylinder_transform
        )
    } else {
        false
    }
}

fn collision_check_cylinder_cylinder(
    a_shape: Shape,
    b_shape: Shape,
    a_transform: Transform,
    b_transform: Transform
) -> bool {
    if let (Shape::Cylinder { radius: ra, height: ha }, Shape::Cylinder { radius: rb, height: hb }) = (a_shape, b_shape) {
        let ca = mat4x4_extract_translation(a_transform.get_matrix());
        let sa = mat4x4_extract_scale(a_transform.get_matrix());
        let cb = mat4x4_extract_translation(b_transform.get_matrix());
        let sb = mat4x4_extract_scale(b_transform.get_matrix());
        
        // Cylinder A segment endpoints
        let half_ha = ha * 0.5 * sa[1];
        let a1 = [ca[0], ca[1] - half_ha, ca[2]];
        let a2 = [ca[0], ca[1] + half_ha, ca[2]];
        
        // Cylinder B segment endpoints
        let half_hb = hb * 0.5 * sb[1];
        let b1 = [cb[0], cb[1] - half_hb, cb[2]];
        let b2 = [cb[0], cb[1] + half_hb, cb[2]];
        
        // Segment-segment distance
        let dist_sq = segment_segment_distance2(a1, a2, b1, b2);
        let sum_radii = ra + rb;
        
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

fn collision_check_capsule_cylinder(
    capsule_shape: Shape,
    cylinder_shape: Shape,
    capsule_transform: Transform,
    cylinder_transform: Transform
) -> bool {
    if let (Shape::Capsule { radius: cap_radius, height: cap_height }, Shape::Cylinder { radius: cyl_radius, height: cyl_height }) = (capsule_shape, cylinder_shape) {
        let cap_center = mat4x4_extract_translation(capsule_transform.get_matrix());
        let cap_scale = mat4x4_extract_scale(capsule_transform.get_matrix());
        let cyl_center = mat4x4_extract_translation(cylinder_transform.get_matrix());
        let cyl_scale = mat4x4_extract_scale(cylinder_transform.get_matrix());
        
        // Capsule segment endpoints
        let cap_half_height = cap_height * 0.5 * cap_scale[1];
        let cap1 = [cap_center[0], cap_center[1] - cap_half_height, cap_center[2]];
        let cap2 = [cap_center[0], cap_center[1] + cap_half_height, cap_center[2]];
        
        // Cylinder segment endpoints
        let cyl_half_height = cyl_height * 0.5 * cyl_scale[1];
        let cyl1 = [cyl_center[0], cyl_center[1] - cyl_half_height, cyl_center[2]];
        let cyl2 = [cyl_center[0], cyl_center[1] + cyl_half_height, cyl_center[2]];
        
        // Segment-segment distance
        let dist_sq = segment_segment_distance2(cap1, cap2, cyl1, cyl2);
        let sum_radii = cap_radius + cyl_radius;
        
        dist_sq <= sum_radii * sum_radii
    } else {
        false
    }
}

impl Component for Collider {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        use slint::Model;

        let mut new_is_hidden = self.is_hidden;
        let mut new_layer = self.layer.clone();
        let mut shape_changed = false;
        let mut new_shape = self.shape.clone();

        println!("🔧 Applying UI changes to Collider...");
        println!("  📊 Component UI has {} attributes", component_ui.attributes.row_count());

        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
                match attribute.name.as_str() {
                    "Is Hidden" => {
                        if let Ok(value) = attribute.value.parse::<bool>() {
                            if value != new_is_hidden {
                                println!("  📝 Is Hidden: {} -> {}", new_is_hidden, value);
                                new_is_hidden = value;
                            }
                        }
                    }
                    "Layer" => {
                        let new_layer_value = match attribute.value.as_str() {
                            "Environment" => ColliderLayer::Environment,
                            "Player" => ColliderLayer::Player,
                            _ => new_layer.clone(),
                        };
                        if new_layer_value != new_layer {
                            println!("  📝 Layer: {:?} -> {:?}", new_layer, new_layer_value);
                            new_layer = new_layer_value;
                        }
                    }
                    "Half Extent X" => {
                        if let Shape::Box { ref mut half_extents } = new_shape {
                            if let Ok(value) = attribute.value.parse::<f32>() {
                                if (half_extents[0] - value).abs() > 0.001 {
                                    println!("  📝 Half Extent X: {:.3} -> {:.3}", half_extents[0], value);
                                    half_extents[0] = value;
                                    shape_changed = true;
                                }
                            }
                        }
                    }
                    "Half Extent Y" => {
                        if let Shape::Box { ref mut half_extents } = new_shape {
                            if let Ok(value) = attribute.value.parse::<f32>() {
                                if (half_extents[1] - value).abs() > 0.001 {
                                    println!("  📝 Half Extent Y: {:.3} -> {:.3}", half_extents[1], value);
                                    half_extents[1] = value;
                                    shape_changed = true;
                                }
                            }
                        }
                    }
                    "Half Extent Z" => {
                        if let Shape::Box { ref mut half_extents } = new_shape {
                            if let Ok(value) = attribute.value.parse::<f32>() {
                                if (half_extents[2] - value).abs() > 0.001 {
                                    println!("  📝 Half Extent Z: {:.3} -> {:.3}", half_extents[2], value);
                                    half_extents[2] = value;
                                    shape_changed = true;
                                }
                            }
                        }
                    }
                    "Radius" => {
                        if let Ok(value) = attribute.value.parse::<f32>() {
                            match new_shape {
                                Shape::Sphere { ref mut radius } => {
                                    if (*radius - value).abs() > 0.001 {
                                        println!("  📝 Sphere Radius: {:.3} -> {:.3}", *radius, value);
                                        *radius = value;
                                        shape_changed = true;
                                    }
                                }
                                Shape::Capsule { ref mut radius, .. } => {
                                    if (*radius - value).abs() > 0.001 {
                                        println!("  📝 Capsule Radius: {:.3} -> {:.3}", *radius, value);
                                        *radius = value;
                                        shape_changed = true;
                                    }
                                }
                                Shape::Cylinder { ref mut radius, .. } => {
                                    if (*radius - value).abs() > 0.001 {
                                        println!("  📝 Cylinder Radius: {:.3} -> {:.3}", *radius, value);
                                        *radius = value;
                                        shape_changed = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    "Height" => {
                        if let Ok(value) = attribute.value.parse::<f32>() {
                            match new_shape {
                                Shape::Capsule { ref mut height, .. } => {
                                    if (*height - value).abs() > 0.001 {
                                        println!("  📝 Capsule Height: {:.3} -> {:.3}", *height, value);
                                        *height = value;
                                        shape_changed = true;
                                    }
                                }
                                Shape::Cylinder { ref mut height, .. } => {
                                    if (*height - value).abs() > 0.001 {
                                        println!("  📝 Cylinder Height: {:.3} -> {:.3}", *height, value);
                                        *height = value;
                                        shape_changed = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Track if any changes were made
        let changes_made = shape_changed || 
                          new_is_hidden != self.is_hidden || 
                          new_layer != self.layer;

        // Apply changes to component state
        self.is_hidden = new_is_hidden;
        self.layer = new_layer;
        if shape_changed {
            self.shape = new_shape;
        }

        // If changes were made, refresh the UI to reflect new state
        if changes_made {
            println!("🔄 Refreshing Collider UI after applying changes");
            self.update_component_ui("UI_REFRESH");
        } else {
            println!("📋 No changes detected in Collider UI");
        }
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        let mut ui = self.component_ui.borrow_mut();

        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "Is Hidden" => {
                        attr.value = self.is_hidden.to_string().into();
                    }
                    "Layer" => {
                        attr.value = format!("{:?}", self.layer).into();
                    }
                    "Half Extent X" => {
                        if let Shape::Box { half_extents } = &self.shape {
                            attr.value = format!("{:.3}", half_extents[0]).into();
                        }
                    }
                    "Half Extent Y" => {
                        if let Shape::Box { half_extents } = &self.shape {
                            attr.value = format!("{:.3}", half_extents[1]).into();
                        }
                    }
                    "Half Extent Z" => {
                        if let Shape::Box { half_extents } = &self.shape {
                            attr.value = format!("{:.3}", half_extents[2]).into();
                        }
                    }
                    "Radius" => {
                        match &self.shape {
                            Shape::Sphere { radius } => {
                                attr.value = format!("{:.3}", radius).into();
                            }
                            Shape::Capsule { radius, .. } => {
                                attr.value = format!("{:.3}", radius).into();
                            }
                            Shape::Cylinder { radius, .. } => {
                                attr.value = format!("{:.3}", radius).into();
                            }
                            _ => {}
                        }
                    }
                    "Height" => {
                        match &self.shape {
                            Shape::Capsule { height, .. } => {
                                attr.value = format!("{:.3}", height).into();
                            }
                            Shape::Cylinder { height, .. } => {
                                attr.value = format!("{:.3}", height).into();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }

        println!("🔄 Collider UI updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}

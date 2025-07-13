#[derive(Debug, Clone, Copy)]
pub enum ColliderShape {
    AABB { half_extents: [f32; 2] },
    Circle { radius: f32 },
}

pub struct ColliderSystem;

impl ColliderSystem {
    pub fn check_collision(pos_a: [f32; 2], shape_a: ColliderShape,
                           pos_b: [f32; 2], shape_b: ColliderShape) -> bool {
        match (shape_a, shape_b) {
            (ColliderShape::AABB { half_extents: ha }, ColliderShape::AABB { half_extents: hb }) => {
                let dx = (pos_a[0] - pos_b[0]).abs();
                let dy = (pos_a[1] - pos_b[1]).abs();
                dx <= (ha[0] + hb[0]) && dy <= (ha[1] + hb[1])
            }

            (ColliderShape::Circle { radius: ra }, ColliderShape::Circle { radius: rb }) => {
                let dx = pos_a[0] - pos_b[0];
                let dy = pos_a[1] - pos_b[1];
                let dist_sq = dx * dx + dy * dy;
                let radius_sum = ra + rb;
                dist_sq <= radius_sum * radius_sum
            }

            (ColliderShape::AABB { half_extents: ha }, ColliderShape::Circle { radius: rb }) => {
                Self::aabb_vs_circle(pos_a, ha, pos_b, rb)
            }

            (ColliderShape::Circle { radius: ra }, ColliderShape::AABB { half_extents: hb }) => {
                Self::aabb_vs_circle(pos_b, hb, pos_a, ra)
            }
        }
    }

    fn aabb_vs_circle(aabb_pos: [f32; 2], half_extents: [f32; 2], circle_pos: [f32; 2], radius: f32) -> bool {
        let dx = (circle_pos[0] - aabb_pos[0]).clamp(-half_extents[0], half_extents[0]);
        let dy = (circle_pos[1] - aabb_pos[1]).clamp(-half_extents[1], half_extents[1]);

        let closest_x = aabb_pos[0] + dx;
        let closest_y = aabb_pos[1] + dy;

        let dist_x = closest_x - circle_pos[0];
        let dist_y = closest_y - circle_pos[1];

        let dist_sq = dist_x * dist_x + dist_y * dist_y;
        dist_sq <= radius * radius
    }
}

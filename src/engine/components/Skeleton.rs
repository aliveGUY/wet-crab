#[derive(Debug, Clone)]
pub struct Node {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub parent: u32,
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub nodes: Vec<Node>,
    pub joint_ids: Vec<u32>,
    pub joint_inverse_mats: Vec<[f32; 16]>,
}

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

use crate::index::engine::components::AnimatedObject3D::{Skeleton, AnimationChannel, AnimationType};
use crate::index::engine::utils::math::lerp;

#[derive(Clone)]
pub struct Animator {
    time_since_start: f32,
    animation_speed: f32, // FPS - default 30.0
    frame_count: u64,     // Internal frame counter for timing
}

impl Animator {
    pub fn new() -> Self {
        Self {
            time_since_start: 0.0,
            animation_speed: 30.0, // Default 30 FPS
            frame_count: 0,
        }
    }

    pub fn update_with_data(&mut self, animation_channels: &[AnimationChannel], skeleton: &mut Skeleton) {
        self.advance_time();
        self.apply_animation_with_data(animation_channels, skeleton);
    }

    fn advance_time(&mut self) {
        self.frame_count += 1;
        // Assume 60 FPS base rate, then apply speed multiplier
        let base_frame_time = 1.0 / 60.0; // 60 FPS = ~0.0167 seconds per frame
        let speed_multiplier = self.animation_speed / 30.0; // 30 FPS is "normal" speed
        let effective_frame_time = base_frame_time * speed_multiplier;
        self.time_since_start += effective_frame_time;
    }

    #[allow(dead_code)]
    pub fn get_time(&self) -> f32 {
        self.time_since_start
    }

    pub fn set_animation_speed(&mut self, speed: f32) {
        self.animation_speed = speed;
    }

    pub fn get_animation_speed(&self) -> f32 {
        self.animation_speed
    }

    #[allow(dead_code)]
    pub fn set_fps(&mut self, fps: f32) {
        self.animation_speed = fps;
    }

    fn apply_animation_with_data(&self, animation_channels: &[AnimationChannel], skeleton: &mut Skeleton) {
        for channel in animation_channels {
            if channel.times.is_empty() {
                continue;
            }

            let rel_time_since_start = self.time_since_start % channel.times[channel.num_timesteps - 1];

            let mut last_timestep = 0;
            for (i, &time) in channel.times.iter().enumerate().rev() {
                if rel_time_since_start >= time {
                    last_timestep = i;
                    break;
                }
            }

            let next_timestep = if last_timestep + 1 < channel.num_timesteps {
                last_timestep + 1
            } else {
                last_timestep
            };

            let components = channel.components();
            let last_data = &channel.data[last_timestep * components..(last_timestep + 1) * components];
            let next_data = &channel.data[next_timestep * components..(next_timestep + 1) * components];

            let last_time = channel.times[last_timestep];
            let next_time = channel.times[next_timestep];
            let t = if next_time != last_time {
                (rel_time_since_start - last_time) / (next_time - last_time)
            } else {
                0.0
            };

            let mut out = vec![0.0; components];
            for i in 0..components {
                out[i] = lerp(last_data[i], next_data[i], t);
            }

            if let Some(node) = skeleton.nodes.get_mut(channel.target as usize) {
                match channel.animation_type {
                    AnimationType::Translation => {
                        node.translation[0] = out[0];
                        node.translation[1] = out[1];
                        node.translation[2] = out[2];
                    }
                    AnimationType::Rotation => {
                        node.rotation[0] = out[0];
                        node.rotation[1] = out[1];
                        node.rotation[2] = out[2];
                        node.rotation[3] = out[3];
                    }
                    AnimationType::Scale => {
                        node.scale[0] = out[0];
                        node.scale[1] = out[1];
                        node.scale[2] = out[2];
                    }
                }
            }
        }
    }
}

impl Default for Animator {
    fn default() -> Self {
        Self::new()
    }
}

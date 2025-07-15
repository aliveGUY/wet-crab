use std::sync::Arc;
use egui::{ Context, RawInput, FullOutput };
use egui_glow::Painter;
use glow::HasContext;
use super::bridge::PlatformBridge;
use super::bridge_factory::create_platform_bridge;

pub struct UIManager {
    egui_ctx: Context,
    painter: Painter,
    bridge: Box<dyn PlatformBridge>,
    frame_times: Vec<f32>,
    fps: f32,
}

impl UIManager {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        let egui_ctx = Context::default();
        let painter = Painter::new(gl, "", None, false).map_err(|e|
            format!("Failed to create painter: {}", e)
        )?;
        let bridge = create_platform_bridge();

        // Log platform initialization
        bridge.log_message(
            &format!("UIManager initialized on platform: {}", bridge.get_platform_name())
        );

        Ok(Self {
            egui_ctx,
            painter,
            bridge,
            frame_times: Vec::with_capacity(60),
            fps: 0.0,
        })
    }

    pub fn handle_input(&mut self, _event: &dyn std::any::Any) {
        // Input handling can be added later for mouse interaction
    }

    pub fn update(&mut self, gl: &glow::Context, width: u32, height: u32) {
        // Update FPS calculation using bridge
        let frame_time = self.bridge.get_frame_delta();

        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        if !self.frame_times.is_empty() {
            let avg_frame_time: f32 =
                self.frame_times.iter().sum::<f32>() / (self.frame_times.len() as f32);
            self.fps = if avg_frame_time > 0.0 { 1.0 / avg_frame_time } else { 0.0 };
        }

        // Render the GUI
        self.render_gui(gl, width, height);
    }

    pub fn render(&mut self, _gl: &glow::Context) {
        // Rendering is handled in update method
    }

    fn render_gui(&mut self, gl: &glow::Context, width: u32, height: u32) {
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
        }

        // Get current time from bridge
        let current_time = self.bridge.get_current_time();
        let screen_scale = self.bridge.get_screen_scale();

        // Create RawInput for egui 0.32
        let raw_input = RawInput {
            screen_rect: Some(
                egui::Rect::from_min_size(
                    Default::default(),
                    egui::vec2(width as f32, height as f32)
                )
            ),
            time: Some(current_time),
            ..Default::default()
        };

        let fps = self.fps;
        let platform_name = self.bridge.get_platform_name();
        let bridge_ref = &*self.bridge;

        let FullOutput { shapes, textures_delta, .. } = self.egui_ctx.run(raw_input, |ctx| {
            egui::Window
                ::new("Debug Info")
                .default_pos(egui::pos2(10.0, 10.0))
                .default_size(egui::vec2(250.0, 120.0))
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Platform: {}", platform_name));
                    ui.label(format!("Scale: {:.1}x", screen_scale));

                    ui.separator();

                    ui.label(format!("FPS: {:.1}", fps));

                    ui.separator();

                    ui.button("Test Button")
                });
        });

        for (id, delta) in &textures_delta.set {
            self.painter.set_texture(*id, delta);
        }
        for id in &textures_delta.free {
            self.painter.free_texture(*id);
        }

        // Draw following the guide
        let clipped_primitives = self.egui_ctx.tessellate(shapes, screen_scale);
        self.painter.paint_primitives([width, height], screen_scale, &clipped_primitives);
    }

    pub fn cleanup(&mut self, _gl: &glow::Context) {
        // Cleanup if needed
    }
}

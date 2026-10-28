// Homebrew
mod gfx;
use gfx::GFX;
mod scene;
use scene::Scene;
mod bitmap;
use bitmap::{BitmapDescriptor, BitmapFont};
mod bitmap_renderer;

// Actual Libs
use std::{sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::bitmap_renderer::BitmapRenderer;

struct State {
    window: Arc<Window>,
    gfx: GFX,
    scene: Scene,
    font: BitmapFont,
}

struct App {
    state: Option<State>,
    angle: f32,
    last_frame: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("bmp-obj"))
                .unwrap(),
        );

        let gfx = pollster::block_on(GFX::new(window.clone()));

        let img = image::load_from_memory(include_bytes!("../assets/font1.png"))
            .unwrap()
            .to_rgba8();

        let (w, h) = img.dimensions();
        let scene = Scene::new(&gfx, &img, w, h);

        let font = BitmapFont::new(BitmapDescriptor::new(
            img,
            3,
            5,
            1,
            0,
            true,
            false,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
        ));

        let bmp_renderer = BitmapRenderer::new(&gfx, &font);

        self.last_frame = Instant::now();
        window.request_redraw();
        self.state = Some(State { window, gfx, scene, font });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    )
    {
        let Some(state) = self.state.as_mut() else { return };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    state.gfx.config.width = size.width;
                    state.gfx.config.height = size.height;
                    state.gfx.surface.configure(&state.gfx.device, &state.gfx.config);
                    state.scene.resize(&state.gfx);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;
                self.angle += 1.0 * dt;

                // For testing bc i don't have bmp-rust yet
                // All of this is still just testing wgpu
                let uv = state.font.get_char_uv('A');
                let rect_uv = [
                    [uv.u0, uv.v1],
                    [uv.u1, uv.v1],
                    [uv.u1, uv.v0],
                    [uv.u0, uv.v0],
                ];
                state.scene.render(&state.gfx, self.angle, rect_uv);

                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        state: None,
        angle: 0.0,
        last_frame: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}
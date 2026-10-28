// Homebrew
mod gfx;
use gfx::GFX;

mod bitmap;
use bitmap::{BitmapDescriptor, BitmapFont};

mod bitmap_renderer;
use bitmap_renderer::BitmapRenderer;

// Actual Libs
use std::{sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct State {
    window: Arc<Window>,
    gfx: GFX,
    font: BitmapFont,
    text: BitmapRenderer,
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

        let gfx = pollster::block_on(GFX::new(window.clone(), 0.2, 0.3, 0.3, 1.0));

        let img = image::load_from_memory(include_bytes!("../assets/font1.png"))
            .unwrap()
            .to_rgba8();

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

        let mut text = BitmapRenderer::new(&gfx, &font);
        text.set_text(&gfx, &font, "AA");

        self.last_frame = Instant::now();
        window.request_redraw();
        self.state = Some(State { window, gfx, font, text });
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
                    state.gfx.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;
                self.angle += 1.0 * dt;

                let proj = glam::camera::rh::proj::directx::perspective(
                    45f32.to_radians(), state.gfx.aspect(), 0.1, 100.0);
                let view = glam::camera::rh::view::look_at_mat4(
                    glam::Vec3::new(0.0, 0.0, 3.0), glam::Vec3::ZERO, glam::Vec3::Y);
                let mvp = proj * view * glam::Mat4::from_rotation_y(self.angle);

                state.text.set_transform(&state.gfx, mvp, [1.0, 0.4, 0.1, 1.0]);
                state.gfx.render(|pass| state.text.draw(pass));
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
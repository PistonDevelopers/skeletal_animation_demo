extern crate piston;
extern crate shader_version;
extern crate sdl2;
extern crate sdl2_window;
extern crate gfx;
extern crate camera_controllers;
extern crate vecmath;
extern crate env_logger;
extern crate "gfx_gl" as gl;
extern crate gfx_debug_draw;
extern crate gfx_device_gl;

use gfx_debug_draw::{DebugRendererBuilder};

use gl::Gl;

use std::cell::RefCell;
use piston::window::WindowSettings;
use piston::event::{
    events,
    RenderEvent,
    ResizeEvent,
};

use vecmath::mat4_id;

use sdl2_window::Sdl2Window;

use camera_controllers::{
    OrbitZoomCamera,
    OrbitZoomCameraSettings,
    CameraPerspective,
    model_view_projection
};

fn main() {

    env_logger::init().unwrap();

    let (win_width, win_height) = (640, 480);

    let window = Sdl2Window::new(
        shader_version::OpenGL::_3_2,
        WindowSettings {
            title: "Animation Viewer".to_string(),
            size: [640, 480],
            fullscreen: false,
            exit_on_esc: true,
            samples: 4
        }
    );

    let device = gfx_device_gl::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });

    let mut frame = gfx::Frame::new(win_width as u16, win_height as u16);

    let window = RefCell::new(window);

    let clear = gfx::ClearData {
        color: [0.3, 0.3, 0.3, 1.0],
        depth: 1.0,
        stencil: 0
    };

    let mut graphics = gfx::Graphics::new(device);

    let mut debug_renderer = DebugRendererBuilder::new(&mut graphics, [frame.width as u32, frame.height as u32]).build().ok().unwrap();

    let model = mat4_id();
    let mut projection = CameraPerspective {
        fov: 90.0f32,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: (win_width as f32) / (win_height as f32)
    }.projection();

    let mut orbit_zoom_camera: OrbitZoomCamera<f32> = OrbitZoomCamera::new(
        [0.0, 0.0, 0.0],
        OrbitZoomCameraSettings::default()
    );

    // Start event loop

    Gl::load_with(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });

    for e in events(&window) {

        e.resize(|width, height| {
            debug_renderer.resize(width, height);

            // Update frame
            frame.width = width as u16;
            frame.height = height as u16;

            // Update projection matrix
            projection = CameraPerspective {
                fov: 90.0f32,
                near_clip: 0.1,
                far_clip: 1000.0,
                aspect_ratio: (width as f32) / (height as f32)
            }.projection();
        });

        orbit_zoom_camera.event(&e);

        if let Some(args) = e.render_args() {
            graphics.clear(clear, gfx::COLOR | gfx::DEPTH, &frame);

            let camera_projection = model_view_projection(
                model,
                orbit_zoom_camera.camera(args.ext_dt).orthogonal(),
                projection
            );

            // Draw axes
            debug_renderer.draw_line([0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [0.0, 0.0, 1.0, 1.0]);

            // FIXME - this doesn't actually calculate FPS at all
            debug_renderer.draw_text_on_screen(&format!("FPS: {}", 1.0 / args.ext_dt)[..], [10, 10], [1.0, 0.4, 0.4, 0.7]);

            debug_renderer.draw_text_at_position(
                "X",
                [6.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0],
            );

            debug_renderer.draw_text_at_position(
                "Y",
                [0.0, 6.0, 0.0],
                [0.0, 1.0, 0.0, 1.0],
            );

            debug_renderer.draw_text_at_position(
                "Z",
                [0.0, 0.0, 6.0],
                [0.0, 0.0, 1.0, 1.0],
            );

            debug_renderer.render(&mut graphics, &frame, camera_projection);

            graphics.end_frame();
        }
    }
}

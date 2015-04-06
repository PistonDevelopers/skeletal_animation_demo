extern crate gfx_gl as gl;
extern crate camera_controllers;
extern crate collada;
extern crate env_logger;
extern crate gfx;
extern crate gfx_debug_draw;
extern crate gfx_device_gl;
extern crate piston;
extern crate sdl2;
extern crate sdl2_window;
extern crate shader_version;
extern crate skeletal_animation;
extern crate vecmath;

use gfx::traits::*;
use gfx_debug_draw::DebugRenderer;

use gl::Gl;

use std::path::Path;
use std::rc::Rc;
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

use skeletal_animation::SkinnedRenderer;
use collada::document::ColladaDocument;

fn main() {

    env_logger::init().unwrap();

    let (win_width, win_height) = (640, 480);

    let window = Sdl2Window::new(
        shader_version::OpenGL::_3_2,
        WindowSettings::new(
            "Animation Viewer".to_string(),
            piston::window::Size { width: 640, height: 480 }
        ).exit_on_esc(true)
    );

    let device = gfx_device_gl::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });

    let mut frame = gfx::Frame::new(win_width as u16, win_height as u16);

    let window = Rc::new(RefCell::new(window));

    let clear = gfx::ClearData {
        color: [0.3, 0.3, 0.3, 1.0],
        depth: 1.0,
        stencil: 0
    };

    let mut graphics = device.into_graphics();

    let mut debug_renderer = DebugRenderer::new(*graphics.device.get_capabilities(), &mut graphics.device, [frame.width as u32, frame.height as u32], 64, None, None).ok().unwrap();

    // TODO - these are (usually) available in the COLLADA file, associated with a <mesh> element in a somewhat convoluted way
    let texture_paths = vec![
        "assets/young_lightskinned_male_diffuse.png",
        "assets/suit01lres_diffuse.png",
        "assets/male02_diffuse_black.png",
        "assets/brown_eye.png",
        "assets/eyebrow009.png",
        "assets/eyelashes01.png",
    ];

    let collada_document = ColladaDocument::from_path(&Path::new("assets/suit_guy.dae")).unwrap();
    let mut skinned_renderer = SkinnedRenderer::from_collada(&mut graphics, collada_document, texture_paths).unwrap();

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

    let mut elapsed_time = 0f64;

    for e in events(window) {

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

            let camera_view = orbit_zoom_camera.camera(args.ext_dt).orthogonal();

            let camera_projection = model_view_projection(
                model,
                camera_view,
                projection
            );

            // Draw axes
            debug_renderer.draw_line([0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [0.0, 0.0, 1.0, 1.0]);

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

            debug_renderer.update(&mut graphics.device);
            debug_renderer.render(&mut graphics, &frame, camera_projection);

            elapsed_time = elapsed_time + 0.02f64; // TODO use actual time
            skinned_renderer.render(&mut graphics, &frame, camera_view, camera_projection, elapsed_time as f32);

            graphics.end_frame();
        }
    }
}

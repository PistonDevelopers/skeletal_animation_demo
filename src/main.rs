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

use piston::window::{
    WindowSettings,
    OpenGLWindow,
};

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

use skeletal_animation::*;
use collada::document::ColladaDocument;

use piston::input::keyboard::Key;
use piston::input::Button::Keyboard;

fn main() {

    env_logger::init().unwrap();

    let (win_width, win_height) = (640, 480);

    let mut window = Sdl2Window::new(
        shader_version::OpenGL::_3_2,
        WindowSettings::new(
            "Animation Viewer".to_string(),
            piston::window::Size { width: 640, height: 480 }
        ).exit_on_esc(true)
    );

    let mut graphics = gfx_device_gl::create(|s| window.get_proc_address(s)).into_graphics();

    let mut frame = gfx::Frame::new(win_width as u16, win_height as u16);

    let window = Rc::new(RefCell::new(window));

    let clear = gfx::ClearData {
        color: [0.3, 0.3, 0.3, 1.0],
        depth: 1.0,
        stencil: 0
    };

    let mut debug_renderer = DebugRenderer::new(&mut graphics, [frame.width as u32, frame.height as u32], 64, None, None).ok().unwrap();

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

    let collada_document = ColladaDocument::from_path(&Path::new("assets/walk.dae")).unwrap();
    let animations = collada_document.get_animations();
    let mut skeleton_set = collada_document.get_skeletons().unwrap();
    let skeleton = &skeleton_set[0];
    let mut walk_clip = AnimationClip::from_collada(skeleton, &animations);
    walk_clip.set_duration(1.0);

    let collada_document = ColladaDocument::from_path(&Path::new("assets/run.dae")).unwrap();
    let animations = collada_document.get_animations();
    let mut skeleton_set = collada_document.get_skeletons().unwrap();
    let skeleton = &skeleton_set[0];
    let mut run_clip = AnimationClip::from_collada(skeleton, &animations);
    run_clip.set_duration(1.0);

    let walk_node = ClipNode { start_time: 0.0, clip: &walk_clip };
    let run_node = ClipNode { start_time: 0.0, clip: &run_clip };
    let mut lerp_node = LerpNode { blend_parameter: 0.0, inputs: [&walk_node, &run_node] };

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

    let mut skeleton_toggle = false;
    let mut mesh_toggle = true;

    for e in events(window) {

        use piston::event::PressEvent;

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

        e.press(|button| {
            match button {
                Keyboard(Key::W) => { lerp_node.blend_parameter -= 0.1; },
                Keyboard(Key::R) => { lerp_node.blend_parameter += 0.1; },
                Keyboard(Key::M) => { mesh_toggle = !mesh_toggle; },
                Keyboard(Key::S) => { skeleton_toggle = !skeleton_toggle },
                _ => {},
            }
        });

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

            elapsed_time = elapsed_time + 0.01 + 0.02 * lerp_node.blend_parameter as f64;

            debug_renderer.draw_text_on_screen(
                &format!("Blend Factor: {}", lerp_node.blend_parameter)[..],
                [10, 10],
                [1.0, 0.0, 0.0, 1.0],
            );

            let mut local_poses = [ SQT { translation: [0.0, 0.0, 0.0], scale: 0.0, rotation: (0.0, [0.0, 0.0, 0.0]) }; 64 ];
            lerp_node.get_output_pose(elapsed_time as f32, &mut local_poses[0 .. skeleton.joints.len()]);
            let global_poses = calculate_global_poses(skeleton, &local_poses);

            if mesh_toggle {
                skinned_renderer.render(&mut graphics, &frame, camera_view, camera_projection, &global_poses);
            }

            if skeleton_toggle {
                draw_skeleton(&skeleton, &global_poses, &mut debug_renderer, false);
            }

            debug_renderer.render(&mut graphics, &frame, camera_projection);

            graphics.end_frame();
        }
    }
}

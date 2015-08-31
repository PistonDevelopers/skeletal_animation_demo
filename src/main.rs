extern crate camera_controllers;
extern crate collada;
extern crate dev_menu;
extern crate env_logger;
extern crate gfx;
extern crate gfx_text;
extern crate gfx_debug_draw;
extern crate gfx_device_gl;
extern crate gfx_gl as gl;
extern crate piston;
extern crate piston_window;
extern crate sdl2;
extern crate sdl2_window;
extern crate shader_version;
extern crate skeletal_animation;
extern crate vecmath;

use std::collections::HashMap;
use piston_window::PistonWindow;

use gfx_debug_draw::DebugRenderer;

use piston::window::{
    WindowSettings,
};

use piston::input::*;
use piston::event_loop::*;

use vecmath::{mat4_id};

use sdl2_window::Sdl2Window;

use camera_controllers::{
    OrbitZoomCamera,
    OrbitZoomCameraSettings,
    CameraPerspective,
    model_view_projection
};

mod demo;
use demo::Settings;

fn main() {

    env_logger::init().unwrap();

    let (win_width, win_height) = (640, 480);
    let piston_window: PistonWindow<(), Sdl2Window> =
        WindowSettings::new("Skeletal Animation Demo", [win_width, win_height])
            .exit_on_esc(true)
            .opengl(shader_version::OpenGL::V3_2)
            .build()
            .unwrap();

    let mut debug_renderer = {
        let text_renderer = {
            gfx_text::new(piston_window.factory.borrow().clone()).unwrap()
        };
        DebugRenderer::new(piston_window.factory.borrow().clone(), text_renderer, 64).ok().unwrap()
    };

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

    let mut settings = Settings {

        use_dlb: true,
        draw_skeleton: true,
        draw_labels: false,
        draw_mesh: true,
        playback_speed: 1.0,

        params: HashMap::new(),
    };

    let mut menu = dev_menu::Menu::<Settings>::new();

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle DLB/LBS Skinning",
        Box::new( |ref mut settings| {
            settings.use_dlb = !settings.use_dlb;
        })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Skeleton",
        Box::new( |ref mut settings| { settings.draw_skeleton = !settings.draw_skeleton; })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Joint Labels",
        Box::new( |ref mut settings| { settings.draw_labels = !settings.draw_labels; })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Mesh",
        Box::new( |ref mut settings| { settings.draw_mesh = !settings.draw_mesh; })
    ));

    menu.add_item(dev_menu::MenuItem::slider_item(
        "Playback Speed = ",
        [-5.0, 5.0],
        0.01,
        Box::new( |ref settings| { settings.playback_speed }),
        Box::new( |ref mut settings, value| { settings.playback_speed = value }),
    ));

    let mut lbs_demo = {
        demo::lbs_demo(piston_window.factory.borrow().clone())
    };

    let mut dlb_demo = {
        demo::dlb_demo(piston_window.factory.borrow().clone())
    };

    for (param, &value) in dlb_demo.controller.get_parameters().iter() {
        settings.params.insert(param.clone(), value);

        // Apparently need to make our own string copies to move into each closure..
        let param_copy_1 = param.clone();
        let param_copy_2 = param.clone();

        let range = if param == "target-x" || param == "target-y" || param == "target-z" {
            [-100.0, 100.0]
        } else {
            [0.0, 1.0]
        };

        let rate = if param == "target-x" || param == "target-y" || param == "target-z" {
            0.1
        } else {
            0.01
        };

        menu.add_item(dev_menu::MenuItem::slider_item(
            &format!("Param[{}] = ", param)[..],
            range,
            rate,
            Box::new( move |ref settings| {
                settings.params[&param_copy_1[..]]
            }),
            Box::new( move |ref mut settings, value| {
                settings.params.insert(param_copy_2.clone(), value);
            }),
        ));
    }

    // set head look controller params to nice initial values..
    settings.params.insert("head-look-level".to_string(), 1.0);
    settings.params.insert("head-look-sideways-level".to_string(), 1.0);
    settings.params.insert("head-down-to-up".to_string(), 0.5);
    settings.params.insert("head-left-to-right".to_string(), 0.5);

    for e in piston_window {

        orbit_zoom_camera.event(&e);
        menu.event(&e, &mut settings);

        e.resize(|width, height| {
            // Update projection matrix
            projection = CameraPerspective {
                fov: 90.0f32,
                near_clip: 0.1,
                far_clip: 1000.0,
                aspect_ratio: (width as f32) / (height as f32)
            }.projection();
        });

        e.update(|args| {
            dlb_demo.update(&settings, args.dt);
            lbs_demo.update(&settings, args.dt);
        });

        e.draw_3d(|stream| {

            use gfx::traits::Stream;

            let args = e.render_args().unwrap();

            stream.clear(gfx::ClearData {
                color: [0.3, 0.3, 0.3, 1.0],
                depth: 1.0,
                stencil: 0,
            });

            let camera_view = orbit_zoom_camera.camera(args.ext_dt).orthogonal();

            let camera_projection = model_view_projection(
                model,
                camera_view,
                projection
            );

            // Draw IK target...
            let target = [settings.params["target-x"],
                          settings.params["target-y"],
                          settings.params["target-z"]];

            debug_renderer.draw_line(vecmath::vec3_sub(target, [1.0, 0.0, 0.0]), vecmath::vec3_add(target, [1.0, 0.0, 0.0]), [1.0, 1.0, 1.0, 1.0]);
            debug_renderer.draw_line(vecmath::vec3_sub(target, [0.0, 1.0, 0.0]), vecmath::vec3_add(target, [0.0, 1.0, 0.0]), [1.0, 1.0, 1.0, 1.0]);
            debug_renderer.draw_line(vecmath::vec3_sub(target, [0.0, 0.0, 1.0]), vecmath::vec3_add(target, [0.0, 0.0, 1.0]), [1.0, 1.0, 1.0, 1.0]);

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

            dlb_demo.render(&settings, &mut debug_renderer, stream, camera_view, camera_projection, args.ext_dt, settings.use_dlb);

            lbs_demo.render(&settings, &mut debug_renderer, stream, camera_view, camera_projection, args.ext_dt, !settings.use_dlb);

            menu.draw(&settings, &mut debug_renderer);

            if let Err(e) = debug_renderer.render(stream, camera_projection) {
                println!("{:?}", e);
            }
        });
    }
}

use std::path::Path;

use std::rc::Rc;
use std::collections::HashMap;

use collada::document::ColladaDocument;
use gfx;
use gfx_debug_draw;
use vecmath::Matrix4;
use gfx_texture::TextureContext;

use skeletal_animation::math::DualQuaternion;
use skeletal_animation::*;

pub fn lbs_demo<F: gfx::Factory<R>, R: gfx::Resources, C: gfx::CommandBuffer<R>>(
    tcx: &mut TextureContext<F, R, C>
) -> Demo<R, QVTransform, Matrix4<f32>>
    where R: gfx::Resources {
    Demo::new(tcx)
}

pub fn dlb_demo<F: gfx::Factory<R>, R: gfx::Resources, C: gfx::CommandBuffer<R>>(
    tcx: &mut TextureContext<F, R, C>
) -> Demo<R, DualQuaternion<f32>, DualQuaternion<f32>>
    where R: gfx::Resources {
    Demo::new(tcx)
}
pub struct Settings {
    pub use_dlb: bool,
    pub draw_skeleton: bool,
    pub draw_labels: bool,
    pub draw_mesh: bool,
    pub playback_speed: f32,

    pub params: HashMap<String, f32>,
}

pub struct Demo<R: gfx::Resources, TAnim: Transform, TSkinning: Transform + FromTransform<TAnim> + HasShaderSources<'static>> {
    pub asset_manager: AssetManager<TAnim>,
    pub controller: AnimationController<TAnim>,
    pub skinned_renderer: SkinnedRenderer<R, TSkinning>,
    pub skeleton: Rc<Skeleton>,
}

impl<R: gfx::Resources, TAnim: Transform, TSkinning: Transform + FromTransform<TAnim> + HasShaderSources<'static>> Demo<R, TAnim, TSkinning> {

    pub fn new<F: gfx::Factory<R>, C: gfx::CommandBuffer<R>>(tcx: &mut TextureContext<F, R, C>) -> Demo<R, TAnim, TSkinning> {

        let collada_document = ColladaDocument::from_path(&Path::new("assets/suit_guy.dae")).unwrap();

        let texture_paths = vec![
            "assets/young_lightskinned_male_diffuse.png",
            "assets/suit01lres_diffuse.png",
            "assets/male02_diffuse_black.png",
            "assets/brown_eye.png",
            "assets/eyebrow009.png",
            "assets/eyelashes01.png",
        ];

        // TODO better.. we keep reloading the same document over and over for different things...
        let skeleton = {
            let skeleton_set = collada_document.get_skeletons().unwrap();
            Skeleton::from_collada(&skeleton_set[0])
        };

        let skeleton = Rc::new(skeleton);

        let mut asset_manager = AssetManager::<TAnim>::new();

        asset_manager.load_assets("assets/assets.json");

        let controller_def = asset_manager.controller_defs["human-controller"].clone();

        let controller = AnimationController::new(controller_def, skeleton.clone(), &asset_manager.animation_clips);

        let skinned_renderer = SkinnedRenderer::<R, TSkinning>::from_collada(tcx, collada_document, texture_paths).unwrap();

        Demo {
            asset_manager: asset_manager,
            controller: controller,
            skinned_renderer: skinned_renderer,
            skeleton: skeleton,
        }
    }

    pub fn update(&mut self, settings: &Settings, dt: f64) {
        self.controller.set_playback_speed(settings.playback_speed as f64);

        for (param, &value) in settings.params.iter() {
            self.controller.set_param_value(param, value);
        }

        self.controller.update(dt);
    }

    pub fn render<F: gfx::Factory<R>, C: gfx::CommandBuffer<R>, Rf: gfx::format::RenderFormat>(
        &mut self,
        settings: &Settings,
        debug_renderer: &mut gfx_debug_draw::DebugRenderer<R, F>,
        encoder: &mut gfx::Encoder<R, C>,
        out_color: &gfx::handle::RenderTargetView<R, Rf>,
        out_depth: &gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
        camera_view: [[f32; 4]; 4],
        camera_projection: [[f32; 4]; 4],
        ext_dt: f64,
        should_draw: bool
    )
        where TSkinning: gfx::traits::Pod
    {
        let mut global_poses = [ TSkinning::identity(); 64 ];

        self.controller.get_output_pose(ext_dt, &mut global_poses[0 .. self.skeleton.joints.len()]);
        if should_draw {
            if settings.draw_mesh {
                self.skinned_renderer.render(encoder, out_color, out_depth, camera_view, camera_projection, &global_poses);
            }

            if settings.draw_skeleton {
                self.skeleton.draw(&global_poses, debug_renderer, settings.draw_labels);
            }
        }
    }
}

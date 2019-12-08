//! Shadow mapping.
//!
//! Heavily inspired by:
//! https://github.com/glium/glium/blob/master/examples/shadow_mapping.rs

mod shaders;

use log::info;

use nalgebra as na;

use glium::Surface;

use crate::pipeline::{RenderPassComponent, ScenePassComponent};
use crate::shader::{self, ToUniforms};
use crate::{Camera, Context, DrawError, Drawable};

pub use crate::CreationError;

#[derive(Debug, Clone)]
pub struct Config {
    pub shadow_map_size: na::Vector2<u32>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            shadow_map_size: na::Vector2::new(4096, 4096),
        }
    }
}

pub struct ShadowMapping {
    shadow_texture: glium::texture::DepthTexture2d,
}

impl RenderPassComponent for ShadowMapping {
    fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        let mut shadow_target =
            glium::framebuffer::SimpleFrameBuffer::depth_only(facade, &self.shadow_texture)?;

        shadow_target.clear_depth(1.0);

        Ok(())
    }
}

/*struct ScenePassUniforms<'a> {
    mat_light_view_projection: na::Matrix4<f32>,
    shadow_map: glium::uniforms::Sampler<'a, glium::texture::Texture2d>,
}

impl_uniform_input_with_lifetime!(
    ScenePassUniforms,
    'a,
    self => {
        mat_light_view_projection: [[f32; 4]; 4] => self.mat_light_view_projection.into(),
        shadow_map: glium::uniforms::Sampler<'a, glium::texture::Texture2d> => self.shadow_map,
    },
);*/

impl ScenePassComponent for ShadowMapping {
    /// Transforms a shader so that it shadows the scene.
    ///
    /// Note that the transformed shader will require additional uniforms,
    /// which are given by `ShadowMapping::scene_pass_uniforms`.
    fn core_transform<P, I, V>(
        &self,
        core: shader::Core<(Context, P), I, V>,
    ) -> shader::Core<(Context, P), I, V> {
        shaders::render_shadowed_core_transform(core)
    }
}

impl ShadowMapping {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
    ) -> Result<ShadowMapping, CreationError> {
        info!("Creating shadow texture");
        let shadow_texture = glium::texture::DepthTexture2d::empty(
            facade,
            config.shadow_map_size.x,
            config.shadow_map_size.y,
        )?;

        info!("Shadow mapping initialized");

        Ok(ShadowMapping { shadow_texture })
    }

    fn light_projection(&self) -> na::Matrix4<f32> {
        let w = 20.0;
        na::Matrix4::new_orthographic(-w, w, -w, w, 10.0, 50.0)
    }

    fn light_view(&self, context: &Context) -> na::Matrix4<f32> {
        na::Matrix4::look_at_rh(
            &context.main_light_pos,
            &context.main_light_center,
            &na::Vector3::new(0.0, 0.0, 1.0),
        )
    }

    pub fn shadow_pass_core_transform<P, I, V>(
        &self,
        core: shader::Core<P, I, V>,
    ) -> shader::Core<P, I, V> {
        shaders::depth_map_core_transform(core)
    }

    /// Render scene from the light's point of view into depth buffer.
    pub fn shadow_pass<F, I, V, P>(
        &self,
        facade: &F,
        drawable: &impl Drawable<I, V>,
        program: &glium::Program,
        params: (&Context, P),
    ) -> Result<(), DrawError>
    where
        F: glium::backend::Facade,
        V: glium::vertex::Vertex,
        P: ToUniforms,
    {
        let mut shadow_target =
            glium::framebuffer::SimpleFrameBuffer::depth_only(facade, &self.shadow_texture)?;

        let light_projection = self.light_projection();
        let light_view = self.light_view(params.0);

        let camera = Camera {
            viewport: params.0.camera.viewport,
            projection: light_projection,
            view: light_view,
        };

        let light_context = Context {
            camera,
            ..*params.0
        };

        let draw_params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        drawable.draw(
            program,
            &(light_context, params.1),
            &draw_params,
            &mut shadow_target,
        )
    }

    /// Returns uniforms for binding the shadow map during scene pass.
    pub fn scene_pass_uniforms(&self, context: &Context) -> impl ToUniforms + '_ {
        let light_projection = self.light_projection();
        let light_view = self.light_view(context);
        let mat_light_view_projection: [[f32; 4]; 4] = (light_projection * light_view).into();
        let shadow_map = glium::uniforms::Sampler::new(&self.shadow_texture)
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest);

        plain_uniforms! {
            mat_light_view_projection: mat_light_view_projection,
            shadow_map: shadow_map,
        }
    }
}

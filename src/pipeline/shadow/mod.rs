//! Shadow mapping.
//!
//! Heavily inspired by:
//! https://github.com/glium/glium/blob/master/examples/shadow_mapping.rs

mod shaders;

use log::info;

use nalgebra as na;

use glium::texture::DepthTexture2d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler};
use glium::Surface;

use crate::pipeline::render_pass::{HasScenePassParams, RenderPassComponent, ScenePassComponent};
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
    shadow_texture: DepthTexture2d,
}

impl RenderPassComponent for ShadowMapping {
    fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        let mut shadow_target =
            glium::framebuffer::SimpleFrameBuffer::depth_only(facade, &self.shadow_texture)?;

        shadow_target.clear_depth(1.0);

        Ok(())
    }
}

pub struct ScenePassParams<'a> {
    mat_light_view_projection: na::Matrix4<f32>,
    shadow_map: Sampler<'a, DepthTexture2d>,
}

impl_uniform_input_with_lifetime!(
    ScenePassParams<'a>,
    self => {
        mat_light_view_projection: [[f32; 4]; 4] => self.mat_light_view_projection.into(),
        shadow_map: Sampler<'a, DepthTexture2d> => self.shadow_map,
    },
);

impl<'u> HasScenePassParams<'u> for ShadowMapping {
    type Params = ScenePassParams<'u>;
}

impl ScenePassComponent for ShadowMapping {
    /// Transforms a shader so that it shadows the scene.
    ///
    /// Note that the transformed shader will require additional uniforms,
    /// which are given by `params`.
    fn core_transform<P, I, V>(
        &self,
        core: shader::Core<(Context, P), I, V>,
    ) -> shader::Core<(Context, P), I, V> {
        shaders::render_shadowed_core_transform(core)
    }

    fn params(&self, context: &Context) -> ScenePassParams {
        ScenePassParams {
            mat_light_view_projection: self.light_projection() * self.light_view(context),
            shadow_map: Sampler::new(&self.shadow_texture)
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .minify_filter(MinifySamplerFilter::Nearest),
        }
    }
}

impl ShadowMapping {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
    ) -> Result<ShadowMapping, CreationError> {
        info!("Creating shadow texture");
        let shadow_texture =
            DepthTexture2d::empty(facade, config.shadow_map_size.x, config.shadow_map_size.y)?;

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
        draw_params: &glium::DrawParameters,
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
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            ..draw_params.clone()
        };

        drawable.draw(
            program,
            &(light_context, params.1),
            &draw_params,
            &mut shadow_target,
        )
    }
}

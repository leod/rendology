//! Deferred shading.
//!
//! Heavily inspired by:
//! https://github.com/glium/glium/blob/master/examples/deferred.rs

pub mod shader;

use log::info;

use nalgebra as na;

use glium::{glutin, uniform, Surface};

use crate::render::pipeline::instance::{UniformsOption, UniformsPair};
use crate::render::pipeline::{
    CompositionPassComponent, Context, InstanceParams, Light, RenderPass, ScenePassComponent,
};
use crate::render::{self, screen_quad, Camera, DrawError, Object, Resources, ScreenQuad};

pub use crate::render::CreationError;

#[derive(Debug, Clone, Default)]
pub struct Config;

const NUM_TEXTURES: usize = 2;

pub struct DeferredShading {
    config: Config,

    scene_textures: [glium::texture::Texture2d; NUM_TEXTURES],
    shadow_texture: Option<glium::texture::Texture2d>,

    light_texture: glium::texture::Texture2d,

    light_screen_quad_program: glium::Program,
    light_object_program: glium::Program,

    screen_quad: ScreenQuad,
}

impl RenderPass for DeferredShading {
    fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(
            facade,
            self.output_textures().iter().cloned(),
        )?;
        framebuffer.clear_color(0.0, 0.0, 0.0, 1.0);

        Ok(())
    }
}

impl ScenePassComponent for DeferredShading {
    fn core_transform<P: InstanceParams, V: glium::vertex::Vertex>(
        &self,
        core: render::shader::Core<(Context, P), V>,
    ) -> render::shader::Core<(Context, P), V> {
        // Write scene to separate buffers
        shader::scene_buffers_core_transform(self.shadow_texture.is_some(), core)
    }

    fn output_textures(&self) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        let mut result = vec![
            ("f_world_pos", &self.scene_textures[0]),
            ("f_world_normal", &self.scene_textures[1]),
        ];

        if let Some(shadow_texture) = self.shadow_texture.as_ref() {
            result.push(("f_shadow", shadow_texture));
        }

        result
    }
}

impl CompositionPassComponent for DeferredShading {
    fn core_transform(
        &self,
        core: render::shader::Core<(), screen_quad::Vertex>,
    ) -> render::shader::Core<(), screen_quad::Vertex> {
        shader::composition_core_transform(core)
    }
}

impl DeferredShading {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        have_shadows: bool,
        window_size: glutin::dpi::LogicalSize,
    ) -> Result<DeferredShading, CreationError> {
        info!("Creating deferred buffer textures");
        let rounded_size: (u32, u32) = window_size.into();
        let scene_textures = [
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
        ];
        let shadow_texture = if have_shadows {
            Some(Self::create_shadow_texture(facade, rounded_size)?)
        } else {
            None
        };
        let light_texture = Self::create_texture(facade, rounded_size)?;

        info!("Creating deferred light programs");
        let light_screen_quad_core = shader::light_screen_quad_core(have_shadows);
        let light_screen_quad_program = light_screen_quad_core.build_program(facade)?;
        let light_object_core = shader::light_object_core(have_shadows);
        let light_object_program = light_object_core.build_program(facade)?;

        info!("Creating screen quad");
        let screen_quad = ScreenQuad::create(facade)?;

        info!("Deferred shading initialized");

        Ok(DeferredShading {
            config: config.clone(),
            scene_textures,
            shadow_texture,
            light_texture,
            light_screen_quad_program,
            light_object_program,
            screen_quad,
        })
    }

    pub fn on_window_resize<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        new_window_size: glutin::dpi::LogicalSize,
    ) -> Result<(), CreationError> {
        info!(
            "Recreating textures for deferred shading with size {:?}",
            new_window_size
        );

        let rounded_size: (u32, u32) = new_window_size.into();
        self.scene_textures = [
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
        ];

        if let Some(shadow_texture) = self.shadow_texture.as_mut() {
            *shadow_texture = Self::create_shadow_texture(facade, rounded_size)?;
        }

        self.light_texture = Self::create_texture(facade, rounded_size)?;

        Ok(())
    }

    pub fn light_pass<F: glium::backend::Facade>(
        &self,
        facade: &F,
        resources: &Resources,
        camera: &Camera,
        lights: &[Light],
    ) -> Result<(), DrawError> {
        let draw_params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            blend: glium::Blend {
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::One,
                },
                alpha: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::One,
                },
                constant_value: (1.0, 1.0, 1.0, 1.0),
            },
            ..Default::default()
        };

        let mut light_buffer =
            glium::framebuffer::SimpleFrameBuffer::new(facade, &self.light_texture)?;

        light_buffer.clear_color(0.0, 0.0, 0.0, 1.0);

        for light in lights.iter() {
            let textures = UniformsPair(
                uniform! {
                    position_texture: &self.scene_textures[0],
                    normal_texture: &self.scene_textures[1],
                },
                UniformsOption(self.shadow_texture.as_ref().map(|shadow_texture| {
                    uniform! {
                        shadow_texture: shadow_texture,
                    }
                })),
            );

            if light.is_main {
                let no_camera = Camera {
                    view: na::Matrix4::identity(),
                    projection: na::Matrix4::identity(),
                    viewport: camera.viewport,
                };

                let uniforms = UniformsPair(light.uniforms(), textures);
                let uniforms = UniformsPair(uniforms, no_camera.uniforms());

                light_buffer.draw(
                    &self.screen_quad.vertex_buffer,
                    &self.screen_quad.index_buffer,
                    &self.light_screen_quad_program,
                    &uniforms,
                    &draw_params,
                )?;
            } else {
                let radius = 3.0;
                let light = Light {
                    radius,
                    ..light.clone()
                };

                let uniforms = UniformsPair(light.uniforms(), textures);
                let uniforms = UniformsPair(uniforms, camera.uniforms());

                let object = resources.get_object_buffers(Object::Sphere);
                object.index_buffer.draw(
                    &object.vertex_buffer,
                    &self.light_object_program,
                    &uniforms,
                    &draw_params,
                    &mut light_buffer,
                )?;
            }
        }

        Ok(())
    }

    pub fn composition_pass_uniforms(&self) -> impl glium::uniforms::Uniforms + '_ {
        uniform! {
            light_texture: &self.light_texture,
        }
    }

    fn create_texture<F: glium::backend::Facade>(
        facade: &F,
        size: (u32, u32),
    ) -> Result<glium::texture::Texture2d, CreationError> {
        Ok(glium::texture::Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            size.0,
            size.1,
        )?)
    }

    fn create_shadow_texture<F: glium::backend::Facade>(
        facade: &F,
        size: (u32, u32),
    ) -> Result<glium::texture::Texture2d, CreationError> {
        Ok(glium::texture::Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            size.0,
            size.1,
        )?)
    }
}

//! Deferred shading.
//!
//! Heavily inspired by:
//! https://github.com/glium/glium/blob/master/examples/deferred.rs

pub mod shader;

use log::info;

use nalgebra as na;

use glium::{glutin, uniform, Surface};

use crate::render::pipeline::instance::UniformsPair;
use crate::render::pipeline::{self, shadow, Context, InstanceParams, Light, RenderLists};
use crate::render::{Resources, ScreenQuad};

pub use crate::render::CreationError;

#[derive(Debug, Clone, Default)]
pub struct Config;

const NUM_TEXTURES: usize = 3;

pub struct DeferredShading {
    config: Config,
    window_size: glutin::dpi::LogicalSize,

    scene_textures: [glium::texture::Texture2d; NUM_TEXTURES],
    depth_texture: glium::texture::DepthTexture2d,
    light_texture: glium::texture::Texture2d,

    scene_program: glium::Program,
    light_program: glium::Program,
    composition_program: glium::Program,

    screen_quad: ScreenQuad,

    shadow_mapping: Option<shadow::ShadowMapping>,
}

impl DeferredShading {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        window_size: glutin::dpi::LogicalSize,
        shadow_mapping_config: &Option<shadow::Config>,
    ) -> Result<DeferredShading, CreationError> {
        let rounded_size: (u32, u32) = window_size.into();

        let scene_textures = [
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
        ];

        let depth_texture = glium::texture::DepthTexture2d::empty_with_format(
            facade,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            rounded_size.0,
            rounded_size.1,
        )?;

        let light_texture = Self::create_texture(facade, rounded_size)?;

        info!("Creating deferred scene program");
        let scene_core = shader::scene_buffers_core_transform(pipeline::simple::plain_core());
        let scene_program = scene_core.build_program(facade)?;

        info!("Creating deferred light program");
        let light_core = shader::light_core();
        let light_program = light_core.build_program(facade)?;

        info!("Creating deferred composition program");
        let composition_core = shader::composition_core();
        let composition_program = composition_core.build_program(facade)?;

        let shadow_mapping = if let Some(config) = shadow_mapping_config {
            Some(shadow::ShadowMapping::create(facade, config, true, false)?)
        } else {
            None
        };

        info!("Creating screen quad");
        let screen_quad = ScreenQuad::create(facade)?;

        info!("Deferred shading initialized");

        Ok(DeferredShading {
            config: config.clone(),
            window_size,
            scene_textures,
            depth_texture,
            light_texture,
            scene_program,
            light_program,
            composition_program,
            screen_quad,
            shadow_mapping,
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

        self.window_size = new_window_size;
        self.scene_textures = [
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
            Self::create_texture(facade, rounded_size)?,
        ];
        self.depth_texture = glium::texture::DepthTexture2d::empty_with_format(
            facade,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            rounded_size.0,
            rounded_size.1,
        )?;
        self.light_texture = Self::create_texture(facade, rounded_size)?;

        Ok(())
    }

    pub fn render_frame<F: glium::backend::Facade, S: glium::Surface>(
        &mut self,
        facade: &F,
        resources: &Resources,
        context: &Context,
        render_lists: &RenderLists,
        target: &mut S,
    ) -> Result<(), glium::DrawError> {
        self.scene_pass(facade, resources, context, render_lists)?;
        self.light_pass(facade, &render_lists.lights)?;
        self.composition_pass(target)?;

        // Before rendering plain non-deferred objects, copy depth buffer to
        // main surface
        profile!("plain");
        let framebuffer =
            glium::framebuffer::SimpleFrameBuffer::depth_only(facade, &self.depth_texture).unwrap(); // TODO: unwrap
        let rounded_size: (u32, u32) = self.window_size.into();
        target.blit_from_simple_framebuffer(
            &framebuffer,
            &glium::Rect {
                left: 0,
                bottom: 0,
                width: rounded_size.1,
                height: rounded_size.1,
            },
            &glium::BlitTarget {
                left: 0,
                bottom: 0,
                width: rounded_size.1 as i32,
                height: rounded_size.1 as i32,
            },
            glium::uniforms::MagnifySamplerFilter::Nearest,
        );
        //target.clear_depth(1.0);

        render_lists
            .plain
            .render(resources, context, &Default::default(), target)?;

        Ok(())
    }

    fn scene_pass<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        resources: &Resources,
        context: &Context,
        render_lists: &RenderLists,
    ) -> Result<(), glium::DrawError> {
        profile!("scene");

        let output = &[
            ("f_color", &self.scene_textures[0]),
            ("f_world_pos", &self.scene_textures[1]),
            ("f_world_normal", &self.scene_textures[2]),
        ];

        // TODO: How can we avoid having to create framebuffers in every frame?
        let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            facade,
            output.iter().cloned(),
            &self.depth_texture,
        )
        .unwrap(); // TODO: unwrap
        framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        if let Some(shadow_mapping) = self.shadow_mapping.as_mut() {
            shadow_mapping.render_shadowed(
                facade,
                resources,
                context,
                render_lists,
                &mut framebuffer,
            )?;
        } else {
            render_lists.solid.render_with_program(
                resources,
                context,
                &Default::default(),
                &self.scene_program,
                &mut framebuffer,
            )?;
        }

        Ok(())
    }

    fn light_pass<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        lights: &[Light],
    ) -> Result<(), glium::DrawError> {
        profile!("light");

        let draw_params = glium::DrawParameters {
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

        let mut light_buffer = glium::framebuffer::SimpleFrameBuffer::new(
            facade,
            &self.light_texture,
            //&self.depth_texture,
        )
        .unwrap(); // TODO: unwrap
        light_buffer.clear_color(0.1, 0.1, 0.1, 1.0);

        for light in lights.iter() {
            let uniforms = UniformsPair(
                light.uniforms(),
                uniform! {
                    position_texture: &self.scene_textures[1],
                    normal_texture: &self.scene_textures[2],
                },
            );

            light_buffer.draw(
                &self.screen_quad.vertex_buffer,
                &self.screen_quad.index_buffer,
                &self.light_program,
                &uniforms,
                &draw_params,
            )?;
        }

        Ok(())
    }

    fn composition_pass<S: glium::Surface>(
        &mut self,
        target: &mut S,
    ) -> Result<(), glium::DrawError> {
        profile!("composition");

        let uniforms = uniform! {
            color_texture: &self.scene_textures[0],
            light_texture: &self.light_texture,
        };

        target.draw(
            &self.screen_quad.vertex_buffer,
            &self.screen_quad.index_buffer,
            &self.composition_program,
            &uniforms,
            &Default::default(),
        )
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
}

pub mod components;
pub mod config;
pub mod deferred;
pub mod glow;
pub mod render_pass;
pub mod shaders;
pub mod shadow;

pub use render_pass::{CompositionPassComponent, RenderPassComponent, ScenePassComponent};

use coarse_prof::profile;
use log::info;

use glium::{uniform, Surface};

use crate::fxaa::{self, FXAA};
use crate::shader::ToUniforms;
use crate::{
    object, scene, shader, Context, DrawError, Instancing, Light, RenderLists, Resources,
    ScreenQuad,
};

use components::{Components, ScenePass, ScenePassSetup};
use config::Config;

pub struct Pipeline {
    components: Components,

    scene_pass_solid: ScenePass<scene::model::Params, object::Vertex>,
    scene_pass_solid_glow: ScenePass<scene::model::Params, object::Vertex>,
    scene_pass_wind: ScenePass<scene::wind::Params, object::Vertex>,

    scene_pass_plain: ScenePass<scene::model::Params, object::Vertex>,

    scene_color_texture: glium::texture::Texture2d,
    scene_depth_texture: glium::texture::DepthTexture2d,

    composition_program: glium::Program,
    composition_texture: glium::texture::Texture2d,

    fxaa: Option<FXAA>,
    copy_texture_program: glium::Program,

    screen_quad: ScreenQuad,
}

impl Pipeline {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        target_size: (u32, u32),
    ) -> Result<Pipeline, CreationError> {
        let components = Components::create(facade, config, target_size)?;

        let scene_pass_solid = components.create_scene_pass(
            facade,
            ScenePassSetup {
                shadow: true,
                glow: false,
            },
            scene::model::scene_core(),
        )?;
        let scene_pass_solid_glow = components.create_scene_pass(
            facade,
            ScenePassSetup {
                shadow: true,
                glow: true,
            },
            scene::model::scene_core(),
        )?;
        let scene_pass_wind = components.create_scene_pass(
            facade,
            ScenePassSetup {
                shadow: false,
                glow: true,
            },
            scene::wind::scene_core(),
        )?;

        let plain_core = scene::model::scene_core();
        let plain_program = plain_core
            .build_program(facade, shader::InstancingMode::Vertex)
            .map_err(crate::CreationError::from)?;
        let plain_instancing = Instancing::create(facade)?;
        let scene_pass_plain = ScenePass {
            setup: ScenePassSetup {
                shadow: false,
                glow: false,
            },
            shader_core: plain_core,
            program: plain_program,
            instancing: plain_instancing,
        };

        let scene_color_texture = Self::create_color_texture(facade, target_size)?;
        let scene_depth_texture = Self::create_depth_texture(facade, target_size)?;

        let composition_core = components.composition_core(config);
        let composition_program = composition_core
            .build_program(facade, shader::InstancingMode::Uniforms)
            .map_err(crate::CreationError::from)?;
        let composition_texture = Self::create_color_texture(facade, target_size)?;

        let fxaa = config
            .fxaa
            .as_ref()
            .map(|config| fxaa::FXAA::create(facade, config))
            .transpose()
            .map_err(CreationError::FXAA)?;
        let copy_texture_program = shaders::composition_core()
            .build_program(facade, shader::InstancingMode::Uniforms)
            .map_err(crate::CreationError::from)?;

        info!("Creating screen quad");
        let screen_quad = ScreenQuad::create(facade)?;

        info!("Pipeline initialized");

        Ok(Pipeline {
            components,
            scene_pass_solid,
            scene_pass_solid_glow,
            scene_pass_plain,
            scene_pass_wind,
            scene_color_texture,
            scene_depth_texture,
            composition_program,
            composition_texture,
            fxaa,
            copy_texture_program,
            screen_quad,
        })
    }

    pub fn draw_frame<F: glium::backend::Facade, S: glium::Surface>(
        &mut self,
        facade: &F,
        resources: &Resources,
        context: &Context,
        render_lists: &RenderLists,
        target: &mut S,
    ) -> Result<(), DrawError> {
        profile!("pipeline");

        // Send instance data to GPU
        {
            profile!("send_data");

            self.scene_pass_solid
                .instancing
                .update(facade, &render_lists.solid.instances)?;
            self.scene_pass_solid_glow
                .instancing
                .update(facade, &render_lists.solid_glow.instances)?;
            self.scene_pass_plain
                .instancing
                .update(facade, &render_lists.plain.instances)?;
            self.scene_pass_wind
                .instancing
                .update(facade, &render_lists.wind.instances)?;
        }

        // Clear buffers
        {
            profile!("clear");

            let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                facade,
                &self.scene_color_texture,
                &self.scene_depth_texture,
            )?;
            framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            self.components.clear_buffers(facade)?;
        }

        // Create shadow map from the main light's point of view
        if let Some(shadow_mapping) = self.components.shadow_mapping.as_ref() {
            profile!("shadow_pass");

            shadow_mapping.shadow_pass(
                facade,
                resources,
                context,
                &self.scene_pass_solid.instancing,
                &self.scene_pass_solid_glow.instancing,
            )?;
        }

        // Render scene into buffers
        {
            profile!("scene_pass");

            self.components.scene_pass(
                facade,
                resources,
                context,
                &self.scene_pass_solid,
                &render_lists.solid,
                &self.scene_color_texture,
                &self.scene_depth_texture,
            )?;
            self.components.scene_pass(
                facade,
                resources,
                context,
                &self.scene_pass_solid_glow,
                &render_lists.solid_glow,
                &self.scene_color_texture,
                &self.scene_depth_texture,
            )?;
            self.components.scene_pass(
                facade,
                resources,
                context,
                &self.scene_pass_wind,
                &render_lists.wind,
                &self.scene_color_texture,
                &self.scene_depth_texture,
            )?;
        }

        // Render light sources into a buffer
        if let Some(deferred_shading) = self.components.deferred_shading.as_mut() {
            profile!("light_pass");

            deferred_shading.light_pass(
                facade,
                resources,
                &context.camera,
                &render_lists.lights,
            )?;
        }

        // Blur the glow texture
        if let Some(glow) = self.components.glow.as_ref() {
            profile!("blur_glow_pass");

            glow.blur_pass(facade)?;
        }

        // Combine buffers
        {
            profile!("composition_pass");

            let mut target_buffer =
                glium::framebuffer::SimpleFrameBuffer::new(facade, &self.composition_texture)?;

            let color_uniform = uniform! {
                color_texture: &self.scene_color_texture,
            };
            let deferred_shading_uniforms = self
                .components
                .deferred_shading
                .as_ref()
                .map(|c| c.composition_pass_uniforms());
            let glow_uniforms = self
                .components
                .glow
                .as_ref()
                .map(|c| c.composition_pass_uniforms());

            let uniforms = (&color_uniform, &deferred_shading_uniforms, &glow_uniforms);

            target_buffer.draw(
                &self.screen_quad.vertex_buffer,
                &self.screen_quad.index_buffer,
                &self.composition_program,
                &uniforms.to_uniforms(),
                &Default::default(),
            )?;
        }

        // Draw plain stuff on top
        {
            profile!("plain");

            let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                facade,
                &self.composition_texture,
                &self.scene_depth_texture,
            )?;

            self.components.scene_pass_to_surface(
                resources,
                context,
                &self.scene_pass_plain,
                &render_lists.plain,
                &mut framebuffer,
            )?;
        }

        // Postprocessing
        if let Some(fxaa) = self.fxaa.as_ref() {
            profile!("fxaa");

            fxaa.draw(&self.composition_texture, target)?;
        } else {
            profile!("copy_to_target");

            target.draw(
                &self.screen_quad.vertex_buffer,
                &self.screen_quad.index_buffer,
                &self.copy_texture_program,
                &uniform! {
                    color_texture: &self.composition_texture,
                },
                &Default::default(),
            )?;
        }

        Ok(())
    }

    pub fn on_target_resize<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        target_size: (u32, u32),
    ) -> Result<(), CreationError> {
        self.components.on_target_resize(facade, target_size)?;

        self.scene_color_texture = Self::create_color_texture(facade, target_size)?;
        self.scene_depth_texture = Self::create_depth_texture(facade, target_size)?;
        self.composition_texture = Self::create_color_texture(facade, target_size)?;

        Ok(())
    }

    fn create_color_texture<F: glium::backend::Facade>(
        facade: &F,
        size: (u32, u32),
    ) -> Result<glium::texture::Texture2d, CreationError> {
        Ok(glium::texture::Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            size.0,
            size.1,
        )
        .map_err(crate::CreationError::from)?)
    }

    fn create_depth_texture<F: glium::backend::Facade>(
        facade: &F,
        size: (u32, u32),
    ) -> Result<glium::texture::DepthTexture2d, crate::CreationError> {
        Ok(glium::texture::DepthTexture2d::empty_with_format(
            facade,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            size.0,
            size.1,
        )
        .map_err(crate::CreationError::from)?)
    }
}

#[derive(Debug)]
pub enum CreationError {
    FXAA(fxaa::CreationError),
    Components(components::CreationError),
    CreationError(crate::CreationError),
}

impl From<crate::CreationError> for CreationError {
    fn from(err: crate::CreationError) -> CreationError {
        CreationError::CreationError(err)
    }
}

impl From<components::CreationError> for CreationError {
    fn from(err: components::CreationError) -> CreationError {
        CreationError::Components(err)
    }
}

pub mod config;
pub mod deferred;
pub mod glow;
pub mod shaders;
pub mod shadow;

mod components;
mod render_pass;

use coarse_prof::profile;
use log::info;

use glium::{uniform, Surface};

use crate::fxaa::{self, FXAA};
use crate::scene::SceneCore;
use crate::shader::ToUniforms;
use crate::{shader, Context, DrawError, Drawable, Light, ScreenQuad};

use components::Components;

pub use config::Config;
pub use render_pass::{
    CompositionPassComponent, PlainScenePass, RenderPassComponent, ScenePassComponent,
    ShadedScenePass, ShadedScenePassSetup, ShadowPass,
};

pub struct Pipeline {
    components: Components,

    target_size: (u32, u32),
    scene_color_texture: glium::texture::Texture2d,
    scene_depth_texture: glium::texture::DepthTexture2d,
    composition_texture: glium::texture::Texture2d,

    composition_program: glium::Program,
    copy_texture_program: glium::Program,

    fxaa: Option<FXAA>,

    screen_quad: ScreenQuad,
}

struct StepContext<'a, F, S> {
    pipeline: &'a mut Pipeline,
    facade: &'a F,
    context: Context,
    target: &'a mut S,
}

#[must_use]
pub struct StartFrameStep<'a, F, S>(StepContext<'a, F, S>);

#[must_use]
pub struct ShadowPassStep<'a, F, S>(StepContext<'a, F, S>);

#[must_use]
pub struct ShadedScenePassStep<'a, F, S>(StepContext<'a, F, S>);

#[must_use]
pub struct PlainScenePassStep<'a, F, S>(StepContext<'a, F, S>);

impl Pipeline {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        target_size: (u32, u32),
    ) -> Result<Pipeline, CreationError> {
        let components = Components::create(facade, config, target_size)?;

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
            target_size,
            scene_color_texture,
            scene_depth_texture,
            composition_texture,
            composition_program,
            copy_texture_program,
            fxaa,
            screen_quad,
        })
    }

    pub fn create_shadow_pass<F, C>(
        &self,
        facade: &F,
        scene_core: C,
    ) -> Result<Option<ShadowPass<C>>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        self.components.create_shadow_pass(facade, scene_core)
    }

    pub fn create_shaded_scene_pass<F, C>(
        &self,
        facade: &F,
        scene_core: C,
        setup: ShadedScenePassSetup,
    ) -> Result<ShadedScenePass<C>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        self.components
            .create_shaded_scene_pass(facade, scene_core, setup)
    }

    pub fn start_frame<'a, F: glium::backend::Facade, S: glium::Surface>(
        &'a mut self,
        facade: &'a F,
        context: Context,
        target: &'a mut S,
    ) -> Result<StartFrameStep<'a, F, S>, DrawError> {
        profile!("start_frame");

        if target.get_dimensions() != self.target_size {
            info!(
                "Target size has changed to {:?}, resizing",
                target.get_dimensions()
            );

            self.on_target_resize(facade, target.get_dimensions())?;
            self.target_size = target.get_dimensions();
        }

        let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            facade,
            &self.scene_color_texture,
            &self.scene_depth_texture,
        )?;
        framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        self.components.clear_buffers(facade)?;

        Ok(StartFrameStep(StepContext {
            pipeline: self,
            facade,
            context,
            target,
        }))
    }

    fn on_target_resize<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        target_size: (u32, u32),
    ) -> Result<(), crate::CreationError> {
        self.components.on_target_resize(facade, target_size)?;

        self.scene_color_texture = Self::create_color_texture(facade, target_size)?;
        self.scene_depth_texture = Self::create_depth_texture(facade, target_size)?;
        self.composition_texture = Self::create_color_texture(facade, target_size)?;

        Ok(())
    }

    fn create_color_texture<F: glium::backend::Facade>(
        facade: &F,
        size: (u32, u32),
    ) -> Result<glium::texture::Texture2d, crate::CreationError> {
        Ok(glium::texture::Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            size.0,
            size.1,
        )?)
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
        )?)
    }
}

impl<'a, F, S> StartFrameStep<'a, F, S> {
    pub fn shadow_pass(self) -> ShadowPassStep<'a, F, S> {
        ShadowPassStep(self.0)
    }

    pub fn shaded_scene_pass(self) -> ShadedScenePassStep<'a, F, S> {
        ShadedScenePassStep(self.0)
    }

    pub fn plain_scene_pass(self) -> PlainScenePassStep<'a, F, S> {
        PlainScenePassStep(self.0)
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> ShadowPassStep<'a, F, S> {
    pub fn draw<C: SceneCore>(
        self,
        pass: &Option<ShadowPass<C>>,
        drawable: &impl Drawable<C::Instance, C::Vertex>,
        params: &C::Params,
    ) -> Result<Self, DrawError> {
        if let (Some(pass), Some(shadow_mapping)) = (
            pass.as_ref(),
            self.0.pipeline.components.shadow_mapping.as_ref(),
        ) {
            shadow_mapping.shadow_pass(
                self.0.facade,
                drawable,
                &pass.program,
                (&self.0.context, params),
            )?;
        }

        Ok(self)
    }

    pub fn shaded_scene_pass(self) -> ShadedScenePassStep<'a, F, S> {
        ShadedScenePassStep(self.0)
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> ShadedScenePassStep<'a, F, S> {
    pub fn draw<C: SceneCore>(
        self,
        pass: &ShadedScenePass<C>,
        drawable: &impl Drawable<C::Instance, C::Vertex>,
        params: &C::Params,
        draw_params: &glium::DrawParameters,
    ) -> Result<Self, DrawError> {
        let pipeline = &self.0.pipeline;

        let mut output_textures = pipeline
            .components
            .shaded_scene_pass_output_textures(&pass.setup);
        output_textures.push((shader::defs::F_COLOR, &pipeline.scene_color_texture));

        let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            self.0.facade,
            output_textures.into_iter(),
            &pipeline.scene_depth_texture,
        )?;

        pipeline.components.scene_pass::<C, _, _>(
            drawable,
            &pass.program,
            (&self.0.context, params),
            draw_params,
            &mut framebuffer,
        )?;

        Ok(self)
    }

    pub fn compose(mut self, lights: &[Light]) -> Result<PlainScenePassStep<'a, F, S>, DrawError> {
        let pipeline = &mut self.0.pipeline;
        let components = &mut pipeline.components;

        // Render light sources into a buffer
        if let Some(deferred_shading) = components.deferred_shading.as_mut() {
            profile!("light_pass");

            deferred_shading.light_pass(self.0.facade, &self.0.context.camera, lights)?;
        }

        // Blur the glow texture
        if let Some(glow) = components.glow.as_ref() {
            profile!("blur_glow_pass");

            glow.blur_pass(self.0.facade)?;
        }

        // Combine buffers
        {
            profile!("composition_pass");

            let mut target_buffer = glium::framebuffer::SimpleFrameBuffer::new(
                self.0.facade,
                &pipeline.composition_texture,
            )?;

            let color_uniform = uniform! {
                color_texture: &pipeline.scene_color_texture,
            };
            let deferred_shading_uniforms = components
                .deferred_shading
                .as_ref()
                .map(|c| c.composition_pass_uniforms());
            let glow_uniforms = components
                .glow
                .as_ref()
                .map(|c| c.composition_pass_uniforms());

            let uniforms = (&color_uniform, &deferred_shading_uniforms, &glow_uniforms);

            target_buffer.draw(
                &pipeline.screen_quad.vertex_buffer,
                &pipeline.screen_quad.index_buffer,
                &pipeline.composition_program,
                &uniforms.to_uniforms(),
                &Default::default(),
            )?;
        }

        Ok(PlainScenePassStep(self.0))
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> PlainScenePassStep<'a, F, S> {
    pub fn present(self) -> Result<(), DrawError> {
        let pipeline = self.0.pipeline;
        let target = self.0.target;

        // Postprocessing
        if let Some(fxaa) = pipeline.fxaa.as_ref() {
            profile!("fxaa");

            fxaa.draw(&pipeline.composition_texture, target)?;
        } else {
            profile!("copy_to_target");

            // TODO: Use blitting instead
            target.draw(
                &pipeline.screen_quad.vertex_buffer,
                &pipeline.screen_quad.index_buffer,
                &pipeline.copy_texture_program,
                &uniform! {
                    color_texture: &pipeline.composition_texture,
                },
                &Default::default(),
            )?;
        }

        Ok(())
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

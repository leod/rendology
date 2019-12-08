mod config;
pub mod deferred;
pub mod glow;
pub mod render_pass;
pub mod shaders;
pub mod shadow;

mod components;

use coarse_prof::profile;
use log::info;

use glium::{uniform, Surface};

use crate::fxaa::{self, FXAA};
use crate::scene::SceneCore;
use crate::shader::{InstancingMode, ToUniforms};
use crate::{shader, Context, DrawError, Drawable, Light, ScreenQuad};

use components::Components;
use render_pass::CompositionPassComponent;

pub use config::Config;
pub use render_pass::{PlainScenePass, ShadedScenePass, ShadedScenePassSetup, ShadowPass};

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
    _prof_guard: coarse_prof::Guard,
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
pub struct AfterComposeStep<'a, F, S>(StepContext<'a, F, S>);

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
        instancing_mode: InstancingMode,
    ) -> Result<Option<ShadowPass<C>>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        self.components
            .create_shadow_pass(facade, scene_core, instancing_mode)
    }

    pub fn create_shaded_scene_pass<F, C>(
        &self,
        facade: &F,
        scene_core: C,
        instancing_mode: InstancingMode,
        setup: ShadedScenePassSetup,
    ) -> Result<ShadedScenePass<C>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        self.components
            .create_shaded_scene_pass(facade, scene_core, instancing_mode, setup)
    }

    pub fn create_plain_scene_pass<F, C>(
        &self,
        facade: &F,
        scene_core: C,
        instancing_mode: InstancingMode,
    ) -> Result<PlainScenePass<C>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        let shader_core = scene_core.scene_core();
        let program = shader_core.build_program(facade, instancing_mode)?;

        Ok(PlainScenePass {
            instancing_mode,
            program,
            shader_core,
        })
    }

    pub fn start_frame<'a, F: glium::backend::Facade, S: glium::Surface>(
        &'a mut self,
        facade: &'a F,
        context: Context,
        target: &'a mut S,
    ) -> Result<StartFrameStep<'a, F, S>, DrawError> {
        let prof_guard = coarse_prof::enter("pipeline");
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
            _prof_guard: prof_guard,
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
    pub fn draw<C, D>(
        self,
        pass: &Option<ShadowPass<C>>,
        drawable: &D,
        params: &C::Params,
        draw_params: &glium::DrawParameters,
    ) -> Result<Self, DrawError>
    where
        C: SceneCore,
        D: Drawable<C::Instance, C::Vertex>,
    {
        if let (Some(pass), Some(shadow_mapping)) = (
            pass.as_ref(),
            self.0.pipeline.components.shadow_mapping.as_ref(),
        ) {
            assert_eq!(pass.instancing_mode, D::INSTANCING_MODE);

            shadow_mapping.shadow_pass(
                self.0.facade,
                drawable,
                &pass.program,
                (&self.0.context, params),
                draw_params,
            )?;
        }

        Ok(self)
    }

    pub fn shaded_scene_pass(self) -> ShadedScenePassStep<'a, F, S> {
        ShadedScenePassStep(self.0)
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> ShadedScenePassStep<'a, F, S> {
    pub fn draw<C, D>(
        self,
        pass: &ShadedScenePass<C>,
        drawable: &D,
        params: &C::Params,
        draw_params: &glium::DrawParameters,
    ) -> Result<Self, DrawError>
    where
        C: SceneCore,
        D: Drawable<C::Instance, C::Vertex>,
    {
        assert_eq!(pass.instancing_mode, D::INSTANCING_MODE);

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

        let draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            ..draw_params.clone()
        };

        pipeline.components.scene_pass::<C, _, _>(
            drawable,
            &pass.program,
            (&self.0.context, params),
            &draw_params,
            &mut framebuffer,
        )?;

        Ok(self)
    }

    pub fn compose(mut self, lights: &[Light]) -> Result<AfterComposeStep<'a, F, S>, DrawError> {
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
                .map(|c| CompositionPassComponent::params(c));
            let glow_uniforms = components
                .glow
                .as_ref()
                .map(|c| CompositionPassComponent::params(c));

            let uniforms = (&color_uniform, &deferred_shading_uniforms, &glow_uniforms);

            target_buffer.draw(
                &pipeline.screen_quad.vertex_buffer,
                &pipeline.screen_quad.index_buffer,
                &pipeline.composition_program,
                &uniforms.to_uniforms(),
                &Default::default(),
            )?;
        }

        Ok(AfterComposeStep(self.0))
    }
}

impl<'a, F, S: Surface> StepContext<'a, F, S> {
    fn present(self) -> Result<(), DrawError> {
        // Postprocessing
        if let Some(fxaa) = self.pipeline.fxaa.as_ref() {
            profile!("fxaa");

            fxaa.draw(&self.pipeline.composition_texture, self.target)?;
        } else {
            profile!("copy_to_target");

            // TODO: Use blitting instead
            self.target.draw(
                &self.pipeline.screen_quad.vertex_buffer,
                &self.pipeline.screen_quad.index_buffer,
                &self.pipeline.copy_texture_program,
                &uniform! {
                    color_texture: &self.pipeline.composition_texture,
                },
                &Default::default(),
            )?;
        }

        Ok(())
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> AfterComposeStep<'a, F, S> {
    pub fn plain_scene_pass(self) -> PlainScenePassStep<'a, F, S> {
        PlainScenePassStep(self.0)
    }

    pub fn present(self) -> Result<(), DrawError> {
        self.0.present()
    }
}

impl<'a, F: glium::backend::Facade, S: Surface> PlainScenePassStep<'a, F, S> {
    pub fn draw<C, D>(
        self,
        pass: &PlainScenePass<C>,
        drawable: &D,
        params: &C::Params,
        draw_params: &glium::DrawParameters,
    ) -> Result<Self, DrawError>
    where
        C: SceneCore,
        D: Drawable<C::Instance, C::Vertex>,
    {
        assert_eq!(pass.instancing_mode, D::INSTANCING_MODE);

        let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            self.0.facade,
            &self.0.pipeline.composition_texture,
            &self.0.pipeline.scene_depth_texture,
        )?;

        drawable.draw(
            &pass.program,
            &(&self.0.context, params),
            &draw_params,
            &mut framebuffer,
        )?;

        Ok(self)
    }

    pub fn present(self) -> Result<(), DrawError> {
        self.0.present()
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

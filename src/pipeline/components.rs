use log::info;

use crate::shader::{self, ToUniforms, ToVertex, UniformInput};
use crate::{fxaa, screen_quad};
use crate::{Context, DrawError, Instancing, RenderList, Resources};

use crate::pipeline::config::Config;
use crate::pipeline::deferred::{self, DeferredShading};
use crate::pipeline::glow::{self, Glow};
use crate::pipeline::render_pass::{
    CompositionPassComponent, RenderPassComponent, ScenePassComponent,
};
use crate::pipeline::shaders;
use crate::pipeline::shadow::{self, ShadowMapping};

pub struct Components {
    pub shadow_mapping: Option<ShadowMapping>,
    pub deferred_shading: Option<DeferredShading>,
    pub glow: Option<Glow>,
}

#[derive(Debug, Clone)]
pub struct ScenePassSetup {
    pub shadow: bool,
    pub glow: bool,
}

pub struct ScenePass<I: ToVertex, V> {
    pub setup: ScenePassSetup,

    /// Currently just used as a phantom.
    #[allow(dead_code)]
    pub shader_core: shader::Core<Context, I, V>,

    pub program: glium::Program,

    pub instancing: Instancing<I>,
}

impl Components {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        target_size: (u32, u32),
    ) -> Result<Self, CreationError> {
        let shadow_mapping = config
            .shadow_mapping
            .as_ref()
            .map(|config| ShadowMapping::create(facade, config))
            .transpose()
            .map_err(CreationError::ShadowMapping)?;

        let deferred_shading = config
            .deferred_shading
            .as_ref()
            .map(|config| {
                DeferredShading::create(facade, &config, shadow_mapping.is_some(), target_size)
            })
            .transpose()
            .map_err(CreationError::DeferredShading)?;

        let glow = config
            .glow
            .as_ref()
            .map(|config| Glow::create(facade, config, target_size))
            .transpose()
            .map_err(CreationError::Glow)?;

        Ok(Self {
            shadow_mapping,
            deferred_shading,
            glow,
        })
    }

    pub fn create_scene_pass<F, I: ToVertex, V>(
        &self,
        facade: &F,
        setup: ScenePassSetup,
        mut shader_core: shader::Core<Context, I, V>,
    ) -> Result<ScenePass<I, V>, crate::CreationError>
    where
        F: glium::backend::Facade,
        I: UniformInput + Clone,
        V: glium::vertex::Vertex,
    {
        info!(
            "Creating scene pass for I={}, V={}",
            std::any::type_name::<I>(),
            std::any::type_name::<V>(),
        );

        if let Some(glow) = self.glow.as_ref() {
            if setup.glow {
                shader_core = ScenePassComponent::core_transform(glow, shader_core);
            } else {
                // Whoopsie there goes the abstraction, heh. All good though.
                shader_core = glow::shaders::no_glow_map_core_transform(shader_core);
            }
        }

        if let Some(shadow_mapping) = self.shadow_mapping.as_ref() {
            if setup.shadow {
                shader_core = ScenePassComponent::core_transform(shadow_mapping, shader_core);
            }
        }

        if let Some(deferred_shading) = self.deferred_shading.as_ref() {
            shader_core = ScenePassComponent::core_transform(deferred_shading, shader_core);
        } else {
            shader_core = shaders::diffuse_scene_core_transform(shader_core);
        }

        let program = shader_core.build_program(facade, shader::InstancingMode::Vertex)?;

        let instancing = Instancing::create(facade)?;

        Ok(ScenePass {
            setup,
            shader_core,
            program,
            instancing,
        })
    }

    pub fn composition_core(&self, config: &Config) -> shader::Core<(), (), screen_quad::Vertex> {
        let mut shader_core = shaders::composition_core();

        if let Some(deferred_shading) = self.deferred_shading.as_ref() {
            shader_core = CompositionPassComponent::core_transform(deferred_shading, shader_core);
        }

        if let Some(glow) = self.glow.as_ref() {
            shader_core = CompositionPassComponent::core_transform(glow, shader_core);
        }

        if let Some(_) = config.hdr {
            // TODO: Use factor
            shader_core = shaders::hdr_composition_core_transform(shader_core);
        }

        if let Some(gamma) = config.gamma_correction {
            shader_core = shaders::gamma_correction_composition_core_transform(shader_core, gamma);
        }

        shader_core
    }

    pub fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        self.shadow_mapping
            .as_ref()
            .map(|c| c.clear_buffers(facade))
            .transpose()?;
        self.deferred_shading
            .as_ref()
            .map(|c| c.clear_buffers(facade))
            .transpose()?;
        self.glow
            .as_ref()
            .map(|c| c.clear_buffers(facade))
            .transpose()?;

        Ok(())
    }

    pub fn scene_output_textures(
        &self,
        setup: &ScenePassSetup,
    ) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        let mut textures = Vec::new();

        textures.extend(
            self.deferred_shading
                .as_ref()
                .map_or(Vec::new(), ScenePassComponent::output_textures),
        );

        if setup.glow {
            textures.extend(
                self.glow
                    .as_ref()
                    .map_or(Vec::new(), ScenePassComponent::output_textures),
            );
        }

        textures
    }

    pub fn scene_pass_to_surface<I, V, S>(
        &self,
        resources: &Resources,
        context: &Context,
        pass: &ScenePass<I, V>,
        _render_list: &RenderList<I>,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        I: ToUniforms + ToVertex,
        V: glium::vertex::Vertex,
        S: glium::Surface,
    {
        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            line_width: Some(2.0),
            ..Default::default()
        };

        let uniforms = (
            context,
            self.shadow_mapping
                .as_ref()
                .map(|c| c.scene_pass_uniforms(context)),
        );

        pass.instancing.draw(
            resources,
            &pass.program,
            &uniforms.to_uniforms(),
            &params,
            target,
        )?;

        Ok(())
    }

    pub fn scene_pass<F, I, V>(
        &self,
        facade: &F,
        resources: &Resources,
        context: &Context,
        pass: &ScenePass<I, V>,
        render_list: &RenderList<I>,
        color_texture: &glium::texture::Texture2d,
        depth_texture: &glium::texture::DepthTexture2d,
    ) -> Result<(), DrawError>
    where
        F: glium::backend::Facade,
        I: ToUniforms + ToVertex,
        V: glium::vertex::Vertex,
    {
        let mut output_textures = self.scene_output_textures(&pass.setup);
        output_textures.push((shader::defs::F_COLOR, color_texture));

        let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            facade,
            output_textures.into_iter(),
            depth_texture,
        )?;

        self.scene_pass_to_surface(resources, context, pass, render_list, &mut framebuffer)
    }

    pub fn on_target_resize<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        target_size: (u32, u32),
    ) -> Result<(), CreationError> {
        if let Some(deferred_shading) = self.deferred_shading.as_mut() {
            deferred_shading
                .on_target_resize(facade, target_size)
                .map_err(CreationError::DeferredShading)?;
        }

        if let Some(glow) = self.glow.as_mut() {
            glow.on_target_resize(facade, target_size)
                .map_err(CreationError::Glow)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum CreationError {
    ShadowMapping(shadow::CreationError),
    DeferredShading(deferred::CreationError),
    Glow(glow::CreationError),
    FXAA(fxaa::CreationError),
    CreationError(crate::CreationError),
}

impl From<crate::CreationError> for CreationError {
    fn from(err: crate::CreationError) -> CreationError {
        CreationError::CreationError(err)
    }
}

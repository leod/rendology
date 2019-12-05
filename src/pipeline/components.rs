use log::info;

use crate::object::ObjectBuffers;
use crate::scene::SceneCore;
use crate::shader::{self, ToUniforms};
use crate::{fxaa, screen_quad, Context, DrawError, Instancing};

use crate::pipeline::config::Config;
use crate::pipeline::deferred::{self, DeferredShading};
use crate::pipeline::glow::{self, Glow};
use crate::pipeline::render_pass::{
    CompositionPassComponent, RenderPassComponent, ScenePassComponent, ShadedScenePass,
    ShadedScenePassSetup, ShadowPass,
};
use crate::pipeline::shaders;
use crate::pipeline::shadow::{self, ShadowMapping};

pub struct Components {
    pub shadow_mapping: Option<ShadowMapping>,
    pub deferred_shading: Option<DeferredShading>,
    pub glow: Option<Glow>,
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

    pub fn create_shadow_pass<F, C>(
        &self,
        facade: &F,
        scene_core: C,
    ) -> Result<Option<ShadowPass<C>>, crate::CreationError>
    where
        F: glium::backend::Facade,
        C: SceneCore,
    {
        self.shadow_mapping
            .as_ref()
            .map(|shadow_mapping| {
                info!("Creating shadow pass for C={}", std::any::type_name::<C>());

                let shader_core =
                    shadow_mapping.shadow_pass_core_transform(scene_core.scene_core());
                let program = shader_core.build_program(facade, shader::InstancingMode::Vertex)?;

                Ok(ShadowPass {
                    program,
                    shader_core,
                })
            })
            .transpose()
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
        info!("Creating scene pass for C={}", std::any::type_name::<C>());

        let mut shader_core = scene_core.scene_core();

        if let Some(glow) = self.glow.as_ref() {
            if setup.draw_glowing {
                shader_core = ScenePassComponent::core_transform(glow, shader_core);
            } else {
                // Whoopsie there goes the "abstraction", heh. All good though.
                shader_core = glow::shaders::no_glow_map_core_transform(shader_core);
            }
        }

        if let Some(shadow_mapping) = self.shadow_mapping.as_ref() {
            if setup.draw_shadowed {
                shader_core = ScenePassComponent::core_transform(shadow_mapping, shader_core);
            }
        }

        if let Some(deferred_shading) = self.deferred_shading.as_ref() {
            shader_core = ScenePassComponent::core_transform(deferred_shading, shader_core);
        } else {
            shader_core = shaders::diffuse_scene_core_transform(shader_core);
        }

        let program = shader_core.build_program(facade, shader::InstancingMode::Vertex)?;

        Ok(ShadedScenePass {
            setup,
            program,
            shader_core,
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

    pub fn scene_pass<C, S>(
        &self,
        object: &ObjectBuffers<C::Vertex>,
        instancing: &Instancing<C::Instance>,
        program: &glium::Program,
        params: (&Context, &C::Params),
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        C: SceneCore,
        S: glium::Surface,
    {
        let uniforms = (
            params,
            self.shadow_mapping
                .as_ref()
                .map(|c| c.scene_pass_uniforms(params.0)),
        );

        instancing.draw(
            object,
            program,
            &uniforms.to_uniforms(),
            &draw_params,
            target,
        )
    }

    pub fn on_target_resize<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        target_size: (u32, u32),
    ) -> Result<(), crate::CreationError> {
        if let Some(deferred_shading) = self.deferred_shading.as_mut() {
            deferred_shading.on_target_resize(facade, target_size)?;
        }

        if let Some(glow) = self.glow.as_mut() {
            glow.on_target_resize(facade, target_size)?;
        }

        Ok(())
    }

    pub fn shaded_scene_pass_output_textures(
        &self,
        setup: &ShadedScenePassSetup,
    ) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        let mut textures = Vec::new();

        textures.extend(
            self.deferred_shading
                .as_ref()
                .map_or(Vec::new(), ScenePassComponent::output_textures),
        );

        if setup.draw_glowing {
            textures.extend(
                self.glow
                    .as_ref()
                    .map_or(Vec::new(), ScenePassComponent::output_textures),
            );
        }

        textures
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

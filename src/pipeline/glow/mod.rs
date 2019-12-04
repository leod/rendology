pub mod shaders;

use log::info;

use glium::framebuffer::SimpleFrameBuffer;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerWrapFunction};
use glium::{uniform, Surface};

use crate::pipeline::{CompositionPassComponent, Context, RenderPassComponent, ScenePassComponent};
use crate::shader::{self, ToUniforms};
use crate::{screen_quad, DrawError, ScreenQuad};

pub use crate::CreationError;

#[derive(Debug, Clone)]
pub struct Config {
    num_blur_passes: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { num_blur_passes: 2 }
    }
}

pub struct Glow {
    config: Config,
    glow_texture: glium::texture::Texture2d,
    glow_texture_back: glium::texture::Texture2d,
    blur_program: glium::Program,
    screen_quad: ScreenQuad,
}

impl RenderPassComponent for Glow {
    fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        let mut framebuffer =
            glium::framebuffer::SimpleFrameBuffer::new(facade, &self.glow_texture)?;
        framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

        Ok(())
    }
}

impl ScenePassComponent for Glow {
    fn core_transform<P, I, V>(
        &self,
        core: shader::Core<(Context, P), I, V>,
    ) -> shader::Core<(Context, P), I, V> {
        shaders::glow_map_core_transform(core)
    }

    fn output_textures(&self) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        vec![("f_glow_color", &self.glow_texture)]
    }
}

impl CompositionPassComponent for Glow {
    fn core_transform(
        &self,
        core: shader::Core<(), (), screen_quad::Vertex>,
    ) -> shader::Core<(), (), screen_quad::Vertex> {
        shaders::composition_core_transform(core)
    }
}

impl Glow {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
        target_size: (u32, u32),
    ) -> Result<Self, CreationError> {
        let glow_texture = Self::create_texture(facade, target_size)?;
        let glow_texture_back = Self::create_texture(facade, target_size)?;

        info!("Creating blur program");
        let blur_program =
            shaders::blur_core().build_program(facade, shader::InstancingMode::Uniforms)?;

        info!("Creating screen quad");
        let screen_quad = ScreenQuad::create(facade)?;

        Ok(Glow {
            config: config.clone(),
            glow_texture,
            glow_texture_back,
            blur_program,
            screen_quad,
        })
    }

    pub fn blur_pass<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError> {
        let glow_map = Sampler::new(&self.glow_texture)
            .magnify_filter(MagnifySamplerFilter::Linear)
            .minify_filter(MinifySamplerFilter::Linear)
            .wrap_function(SamplerWrapFunction::Clamp);
        let glow_map_back = Sampler::new(&self.glow_texture_back)
            .magnify_filter(MagnifySamplerFilter::Linear)
            .minify_filter(MinifySamplerFilter::Linear)
            .wrap_function(SamplerWrapFunction::Clamp);

        let mut glow_buffer = SimpleFrameBuffer::new(facade, &self.glow_texture)?;
        let mut glow_buffer_back = SimpleFrameBuffer::new(facade, &self.glow_texture_back)?;

        for _ in 0..self.config.num_blur_passes {
            glow_buffer_back.draw(
                &self.screen_quad.vertex_buffer,
                &self.screen_quad.index_buffer,
                &self.blur_program,
                &uniform! {
                    horizontal: false,
                    glow_texture: glow_map,
                },
                &Default::default(),
            )?;

            glow_buffer.draw(
                &self.screen_quad.vertex_buffer,
                &self.screen_quad.index_buffer,
                &self.blur_program,
                &uniform! {
                    horizontal: true,
                    glow_texture: glow_map_back,
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
        self.glow_texture = Self::create_texture(facade, target_size)?;
        self.glow_texture_back = Self::create_texture(facade, target_size)?;

        Ok(())
    }

    pub fn composition_pass_uniforms(&self) -> impl ToUniforms + '_ {
        uniform! {
            glow_texture: &self.glow_texture,
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
}

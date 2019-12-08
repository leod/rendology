use crate::pipeline::Context;
use crate::scene::SceneCore;
use crate::shader::{InstancingMode, ToUniforms};
use crate::{screen_quad, shader, DrawError};

pub struct ShadowPass<C: SceneCore> {
    pub instancing_mode: InstancingMode,
    pub program: glium::Program,

    /// The transformed shader core that was used for building the `program`.
    /// Currently this is basically just phantom data.
    #[allow(dead_code)]
    pub shader_core: shader::Core<(Context, C::Params), C::Instance, C::Vertex>,
}

#[derive(Debug, Clone)]
pub struct ShadedScenePassSetup {
    pub draw_shadowed: bool,
    pub draw_glowing: bool,
}

pub struct ShadedScenePass<C: SceneCore> {
    pub instancing_mode: InstancingMode,
    pub setup: ShadedScenePassSetup,

    pub program: glium::Program,

    /// The transformed shader core that was used for building the `program`.
    /// Currently this is basically just phantom data.
    #[allow(dead_code)]
    pub shader_core: shader::Core<(Context, C::Params), C::Instance, C::Vertex>,
}

pub struct PlainScenePass<C: SceneCore> {
    pub instancing_mode: InstancingMode,
    pub program: glium::Program,

    /// The transformed shader core that was used for building the `program`.
    /// Currently this is basically just phantom data.
    #[allow(dead_code)]
    pub shader_core: shader::Core<(Context, C::Params), C::Instance, C::Vertex>,
}

pub trait RenderPassComponent {
    fn clear_buffers<F: glium::backend::Facade>(&self, facade: &F) -> Result<(), DrawError>;
}

pub trait HasScenePassParams<'u> {
    type Params: ToUniforms;
}

pub trait ScenePassComponent: RenderPassComponent + for<'u> HasScenePassParams<'u> {
    fn core_transform<P, I, V>(
        &self,
        core: shader::Core<(Context, P), I, V>,
    ) -> shader::Core<(Context, P), I, V>;

    fn output_textures(&self) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        Vec::new()
    }

    fn params<'u>(&'u self, context: &Context) -> <Self as HasScenePassParams<'u>>::Params;
}

pub trait HasCompositionPassParams<'u> {
    type Params: ToUniforms;
}

pub trait CompositionPassComponent:
    RenderPassComponent + for<'u> HasCompositionPassParams<'u>
{
    fn core_transform(
        &self,
        core: shader::Core<(), (), screen_quad::Vertex>,
    ) -> shader::Core<(), (), screen_quad::Vertex>;

    fn params<'u>(&'u self) -> <Self as HasCompositionPassParams<'u>>::Params;
}

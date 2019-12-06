use crate::pipeline::Context;
use crate::scene::SceneCore;
use crate::shader::InstancingMode;
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

pub trait ScenePassComponent {
    fn core_transform<P, I, V>(
        &self,
        core: shader::Core<(Context, P), I, V>,
    ) -> shader::Core<(Context, P), I, V>;

    fn output_textures(&self) -> Vec<(&'static str, &glium::texture::Texture2d)> {
        Vec::new()
    }

    // Ideally, we would define a method here which returns uniforms that
    // need to be passed into the transformed shaders for the pass to work.
    // Unfortunately, that is not easily possible, since glium's `uniform!`
    // macro returns a long nested type. We could use "impl trait in trait"
    // again (as in `ToUniforms`), but this is blocked by the fact that,
    // for texture uniforms, the returned type borrows `self`, so it is
    // actually a generic type!
    //
    // Thus, for now, the methods returning uniforms are defined in the
    // individual passes separately.
    //fn uniforms<'a>(&'a self) -> impl Uniforms<'a>;
}

pub trait CompositionPassComponent {
    fn core_transform(
        &self,
        core: shader::Core<(), (), screen_quad::Vertex>,
    ) -> shader::Core<(), (), screen_quad::Vertex>;

    // Due to the same reason as described in `ScenePassComponent`, the uniforms
    // are returned in pass-specific methods.
    //fn uniforms<'a>(&'a self) -> impl Uniforms<'a>;
}

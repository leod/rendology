pub mod model;
pub mod wind;

use crate::shader::{self, ToVertex, UniformInput};
use crate::Context;

pub trait SceneCore {
    type Params: UniformInput + Clone;
    type Instance: UniformInput + ToVertex + Clone;
    type Vertex: glium::vertex::Vertex;

    fn scene_core() -> shader::Core<(Context, Self::Params), Self::Instance, Self::Vertex>;
}

pub struct ShadowPass<C: SceneCore> {
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
    pub setup: ShadedScenePassSetup,

    pub program: glium::Program,

    /// The transformed shader core that was used for building the `program`.
    /// Currently this is basically just phantom data.
    #[allow(dead_code)]
    pub shader_core: shader::Core<(Context, C::Params), C::Instance, C::Vertex>,
}

pub struct PlainScenePass<C: SceneCore> {
    pub program: glium::Program,

    /// The transformed shader core that was used for building the `program`.
    /// Currently this is basically just phantom data.
    #[allow(dead_code)]
    pub shader_core: shader::Core<(Context, C::Params), C::Instance, C::Vertex>,
}

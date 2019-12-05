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

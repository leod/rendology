use crate::shader::{self, InstanceInput, UniformInput};
use crate::Context;

pub trait SceneCore {
    type Params: UniformInput + Clone;
    type Instance: InstanceInput + Clone;
    type Vertex: glium::vertex::Vertex;

    fn scene_core(&self) -> shader::Core<(Context, Self::Params), Self::Instance, Self::Vertex>;
}

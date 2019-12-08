use nalgebra as na;

use rendology::{basic_obj, shader, Context, SceneCore};

#[derive(Clone)]
struct Params<'a> {
    texture: &'a glium::Texture2d,
}

#[derive(Clone)]
struct Instance {
    transform: na::Matrix4<f32>,
}

rendology::impl_uniform_input_with_lifetime!(
    Params<'a>,
    self => {
        texture: &'a glium::Texture2d => self.texture,
    },
);

rendology::impl_instance_input!(
    Instance,
    self => {
        transform: [[f32; 4]; 4] => self.transform.into(),
    },
);

struct Core;

impl SceneCore for Core {
    type Params = &'static Params<'static>;
    type Instance = Instance;
    type Vertex = basic_obj::Vertex;

    fn scene_core(&self) -> shader::Core<(Context, Self::Params), Self::Instance, Self::Vertex> {
        shader::Core {
            vertex: shader::VertexCore::empty(),
            fragment: shader::FragmentCore::empty(),
        }
    }
}

fn main() {}

use nalgebra as na;

use crate::scene::SceneCore;
use crate::{basic_obj, shader, Context};

#[derive(Clone, Debug)]
pub struct Instance {
    pub transform: na::Matrix4<f32>,
    pub color: na::Vector4<f32>,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            transform: na::Matrix4::identity(),
            color: na::Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl_instance_input!(
    Instance,
    self => {
        instance_transform: [[f32; 4]; 4] => self.transform,
        instance_color: [f32; 4] => self.color,
    },
);

pub struct Core;

impl SceneCore for Core {
    type Params = ();
    type Instance = Instance;
    type Vertex = basic_obj::Vertex;

    fn scene_core(&self) -> shader::Core<(Context, ()), Instance, basic_obj::Vertex> {
        let vertex = shader::VertexCore::empty()
            .with_out(
                // TODO: Precompute inverse of mat_model if we ever have lots of vertices
                shader::defs::v_world_normal(),
                "normalize(transpose(inverse(mat3(instance_transform))) * normal)",
            )
            .with_out(
                shader::defs::v_world_pos(),
                "instance_transform * vec4(position, 1.0)",
            )
            .with_out(shader::defs::v_color(), "instance_color")
            .with_out_expr(
                shader::defs::V_POSITION,
                "context_camera_projection * context_camera_view * v_world_pos",
            );

        let fragment = shader::FragmentCore::empty()
            .with_in_def(shader::defs::v_color())
            .with_out(shader::defs::f_color(), "v_color");

        shader::Core { vertex, fragment }
    }
}

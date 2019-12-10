//! Smooth screen-space line rendering.
//!
//! Heavily inspired by:
//!     https://mattdesl.svbtle.com/drawing-lines-is-hard
//! More specifically:
//!     https://github.com/mattdesl/webgl-lines/blob/master/projected/vert.glsl

use nalgebra as na;

use glium::implement_vertex;

use crate::{shader, Context, SceneCore};

#[derive(Clone, Debug)]
pub struct Instance {
    pub transform: na::Matrix4<f32>,
    pub color: na::Vector4<f32>,
}

impl_instance_input!(
    Instance,
    self => {
        instance_transform: [[f32; 4]; 4] = self.transform,
        color: [f32; 4] = self.color,
    },
);

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub pos: [f32; 3],
    pub next_pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub orientation: f32,
}

implement_vertex!(Point, pos, next_pos, prev_pos, orientation);

pub struct Core;

impl SceneCore for Core {
    type Params = ();
    type Instance = Instance;
    type Vertex = Point;

    fn scene_core(&self) -> shader::Core<(Context, ()), Instance, Point> {
        let vertex = shader::VertexCore::empty();

        let fragment = shader::FragmentCore::empty();

        shader::Core { vertex, fragment }
    }
}

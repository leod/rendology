//! Smooth screen-space line rendering.
//!
//! Heavily inspired by:
//!     https://mattdesl.svbtle.com/drawing-lines-is-hard
//!
//! More specifically:
//!     https://github.com/mattdesl/webgl-lines/blob/master/projected/vert.glsl
//!
//! I've integrated the anti-aliasing technique from:
//!     https://blog.mapbox.com/drawing-antialiased-lines-with-opengl-8766f34192dc
//!
//! Additional resource:
//!     https://github.com/spite/THREE.MeshLine/blob/master/src/THREE.MeshLine.js

use nalgebra as na;

use glium::implement_vertex;

use crate::{shader, Context, CoreInput, CreationError, Mesh, SceneCore};

#[derive(Clone, Debug)]
pub struct Params {
    pub feather: f32,
}

impl_uniform_input!(
    Params,
    self => {
        params_feather: f32 = self.feather,
    },
);

#[derive(Clone, Debug)]
pub struct Instance {
    pub transform: na::Matrix4<f32>,
    pub color: na::Vector4<f32>,
    pub thickness: f32,
}

impl_instance_input!(
    Instance,
    self => {
        instance_transform: [[f32; 4]; 4] = self.transform,
        instance_color: [f32; 4] = self.color,
        instance_thickness: f32 = self.thickness,
    },
);

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub prev_pos: [f32; 3],
    pub curr_pos: [f32; 3],
    pub next_pos: [f32; 3],
    pub orientation: f32,
}

implement_vertex!(Point, prev_pos, curr_pos, next_pos, orientation);

impl Instance {
    pub fn from_start_end(
        start: &na::Point3<f32>,
        end: &na::Point3<f32>,
        color: &na::Vector4<f32>,
        thickness: f32,
    ) -> Self {
        let delta = end - start;
        let transform = na::Matrix4::from_columns(&[
            na::Vector4::new(delta.x, delta.y, delta.z, 0.0),
            na::Vector4::zeros(),
            na::Vector4::zeros(),
            na::Vector4::new(start.x, start.y, start.z, 1.0),
        ]);

        Self {
            transform,
            color: *color,
            thickness,
        }
    }
}

pub fn create_mesh<F: glium::backend::Facade>(facade: &F) -> Result<Mesh<Point>, CreationError> {
    let points = vec![
        Point {
            prev_pos: [0.0, 0.0, 0.0],
            curr_pos: [0.0, 0.0, 0.0],
            next_pos: [1.0, 0.0, 0.0],
            orientation: 1.0,
        },
        Point {
            prev_pos: [0.0, 0.0, 0.0],
            curr_pos: [0.0, 0.0, 0.0],
            next_pos: [1.0, 0.0, 0.0],
            orientation: -1.0,
        },
        Point {
            prev_pos: [0.0, 0.0, 0.0],
            curr_pos: [1.0, 0.0, 0.0],
            next_pos: [1.0, 0.0, 0.0],
            orientation: -1.0,
        },
        Point {
            prev_pos: [0.0, 0.0, 0.0],
            curr_pos: [1.0, 0.0, 0.0],
            next_pos: [1.0, 0.0, 0.0],
            orientation: 1.0,
        },
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    Mesh::create_with_indices(
        facade,
        glium::index::PrimitiveType::TrianglesList,
        &points,
        &indices,
    )
}

const V_NORMAL: (&str, shader::VertexOutDef) = (
    "v_normal",
    shader::VertexOutDef(shader::Type::FloatVec2, shader::VertexOutQualifier::Smooth),
);

const VERTEX_BODY: &str = "
    vec2 aspect_vec = vec2(context_camera_viewport_size.x / context_camera_viewport_size.y, 1.0);
    mat4 transform = context_camera_projection * context_camera_view * instance_transform;

    vec4 prev_projected = transform * vec4(prev_pos, 1.0);
    vec4 curr_projected = transform * vec4(curr_pos, 1.0);
    vec4 next_projected = transform * vec4(next_pos, 1.0);

    vec2 prev_screen = prev_projected.xy / prev_projected.w * aspect_vec;
    vec2 curr_screen = curr_projected.xy / curr_projected.w * aspect_vec;
    vec2 next_screen = next_projected.xy / next_projected.w * aspect_vec;
    
    vec2 line_direction;

    if (curr_pos == prev_pos) {
        // Start of the line
        line_direction = normalize(next_screen - curr_screen);
    } else if (curr_pos  == next_pos) {
        // End of the line
        line_direction = normalize(curr_screen - prev_screen);
    } else {
        // Middle of the line
        line_direction = normalize(curr_screen - prev_screen);
    }

    float w = instance_thickness
        * curr_projected.w
        / (context_camera_viewport_size.x * context_camera_projection[0][0]);

    vec2 line_normal = vec2(-line_direction.y, line_direction.x) * orientation;
    vec2 line_offset = line_normal * w / 2.0;
    line_offset.x /= aspect_vec.x;
";

pub struct Core;

impl CoreInput for Core {
    type Params = Params;
    type Instance = Instance;
    type Vertex = Point;
}

impl SceneCore for Core {
    fn scene_core(&self) -> shader::Core<(Context, Params), Instance, Point> {
        let vertex = shader::VertexCore::empty()
            .with_body(VERTEX_BODY)
            .with_out(shader::defs::V_COLOR, "instance_color")
            .with_out(shader::defs::V_WORLD_POS, "vec4(0.0, 0.0, 0.0, 0.0)")
            .with_out(shader::defs::V_WORLD_NORMAL, "vec3(0.0, 0.0, 0.0)")
            .with_out(V_NORMAL, "line_normal")
            .with_out(
                shader::defs::V_POS,
                "curr_projected + vec4(line_offset, 0.0, 0.0)",
            );

        let fragment = shader::FragmentCore::empty()
            .with_in_def(shader::defs::V_COLOR)
            .with_in_def(V_NORMAL)
            .with_body(
                "
                float distance_from_center = length(v_normal);
                float alpha = 1.0 - 
                    step(1.0 - params_feather, distance_from_center)
                    * (distance_from_center - (1.0 - params_feather))
                    / params_feather;
                ",
            )
            .with_out(
                shader::defs::F_COLOR,
                "vec4(v_color.rgb, v_color.a * alpha)",
            );

        shader::Core { vertex, fragment }
    }
}

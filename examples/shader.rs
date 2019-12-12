use nalgebra as na;

use rendology::shader;

#[derive(Clone)]
struct Params {
    projection_matrix: na::Matrix4<f32>,
    view_matrix: na::Matrix4<f32>,
    light_pos: na::Vector3<f32>,
}

rendology::impl_uniform_input!(
    Params,
    self => {
        projection_matrix: [[f32; 4]; 4] = self.projection_matrix,
        view_matrix: [[f32; 4]; 4] = self.view_matrix,
        light_pos: [f32; 3] = self.light_pos,
    },
);

#[derive(Clone)]
struct Instance {
    instance_matrix: na::Matrix4<f32>,
}

rendology::impl_instance_input!(
    Instance,
    self => {
        instance_matrix: [[f32; 4]; 4] = self.instance_matrix,
    },
);

#[derive(Copy, Clone)]
struct Vertex {
    vertex_pos: [f32; 3],
    vertex_normal: [f32; 3],
}

glium::implement_vertex!(Vertex, vertex_pos, vertex_normal);

fn scene_core() -> shader::Core<Params, Instance, Vertex> {
    let vertex = shader::VertexCore::empty()
        .with_body("mat3 normal_matrix = transpose(inverse(mat3(instance_matrix)));")
        .with_out(
            shader::defs::V_WORLD_POS,
            "instance_matrix * vec4(vertex_pos, 1.0)",
        )
        .with_out(
            shader::defs::V_WORLD_NORMAL,
            "normalize(normal_matrix * vertex_normal)",
        )
        .with_out(
            shader::defs::V_POS,
            "projection_matrix * view_matrix * v_world_pos",
        );

    let fragment =
        shader::FragmentCore::empty().with_out(shader::defs::F_COLOR, "vec4(1, 0, 0, 1)");

    shader::Core { vertex, fragment }
}

fn diffuse_transform<I, V>(core: shader::Core<Params, I, V>) -> shader::Core<Params, I, V> {
    let fragment = core
        .fragment
        .with_in_def(shader::defs::V_WORLD_POS)
        .with_in_def(shader::defs::V_WORLD_NORMAL)
        .with_body(
            "
            float diffuse = max(
                0.0,
                dot(v_world_normal, normalize(light_pos - v_world_pos.xyz))
            );
            ",
        )
        .with_out_expr("f_color", "diffuse * f_color");

    shader::Core {
        vertex: core.vertex,
        fragment,
    }
}

fn main() {
    let linked_core = scene_core().link();

    println!(
        "PLAIN SHADER:\n=====\n{}\n=====\n{}",
        linked_core.vertex.compile(shader::InstancingMode::Uniforms),
        linked_core.fragment.compile(),
    );

    let transformed_core = diffuse_transform(scene_core());
    let linked_core = transformed_core.link();

    println!(
        "TRANSFORMED SHADER:\n=====\n{}\n=====\n{}",
        linked_core.vertex.compile(shader::InstancingMode::Uniforms),
        linked_core.fragment.compile(),
    );
}

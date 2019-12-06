use glium::uniforms::UniformType;

use crate::pipeline::Light;
use crate::{basic_obj, screen_quad, shader, Camera};

pub const F_WORLD_POS: &str = "f_world_pos";
pub const F_WORLD_NORMAL: &str = "f_world_normal";

pub fn f_world_pos() -> shader::FragmentOutDef {
    (
        (F_WORLD_POS.into(), UniformType::FloatVec4),
        shader::FragmentOutQualifier::Yield,
    )
}

pub fn f_world_normal() -> shader::FragmentOutDef {
    (
        (F_WORLD_NORMAL.into(), UniformType::FloatVec4),
        shader::FragmentOutQualifier::Yield,
    )
}

/// Shader core transform for writing position/normal/color into separate
/// buffers, so that they may be combined in a subsequent pass.
pub fn scene_buffers_core_transform<P, I, V>(
    always_include_shadow_out: bool,
    core: shader::Core<P, I, V>,
) -> shader::Core<P, I, V> {
    assert!(
        core.vertex.has_out(shader::defs::V_WORLD_POS),
        "VertexCore needs V_WORLD_POS output for deferred shading scene pass"
    );
    assert!(
        core.vertex.has_out(shader::defs::V_WORLD_NORMAL),
        "VertexCore needs V_WORLD_NORMAL output for deferred shading scene pass"
    );
    assert!(
        core.fragment.has_out(shader::defs::F_COLOR),
        "FragmentCore needs F_COLOR output for deferred shading scene pass"
    );

    let mut fragment = core
        .fragment
        .with_in_def(shader::defs::v_world_pos())
        .with_in_def(shader::defs::v_world_normal())
        .with_out(f_world_pos(), "v_world_pos")
        .with_out(f_world_normal(), "vec4(v_world_normal, 0.0)");

    // We may have the case that we want to attach an `f_shadow` output, but
    // the given `core` does not provide any shadow values (i.e. it wants to
    // be unshadowed). In that case, we still need to provide a shadow value.
    if always_include_shadow_out && !fragment.has_out(shader::defs::F_SHADOW) {
        fragment = fragment.with_out(shader::defs::f_shadow(), "1.0");
    }

    // This is a bit sneaky: we turn `f_shadow` from a local variable into
    // something that is output by the fragment shader.
    fragment.out_defs = fragment
        .out_defs
        .iter()
        .map(|(def, qualifier)| {
            if def.0 == shader::defs::F_SHADOW {
                (def.clone(), shader::FragmentOutQualifier::Yield)
            } else {
                (def.clone(), *qualifier)
            }
        })
        .collect();

    shader::Core {
        vertex: core.vertex,
        fragment,
    }
}

pub const V_LIGHT_POS: &str = "v_light_pos";
pub const V_LIGHT_COLOR: &str = "v_light_color";
pub const V_LIGHT_ATTENUATION: &str = "v_light_attenuation";

pub fn v_light_pos() -> shader::VertexOutDef {
    (
        (V_LIGHT_POS.into(), UniformType::FloatVec3),
        shader::VertexOutQualifier::Flat,
    )
}

pub fn v_light_color() -> shader::VertexOutDef {
    (
        (V_LIGHT_COLOR.into(), UniformType::FloatVec3),
        shader::VertexOutQualifier::Flat,
    )
}

pub fn v_light_attenuation() -> shader::VertexOutDef {
    (
        (V_LIGHT_ATTENUATION.into(), UniformType::FloatVec3),
        shader::VertexOutQualifier::Flat,
    )
}

fn light_fragment_core() -> shader::FragmentCore<Camera> {
    shader::FragmentCore::empty()
        .with_extra_uniform("position_texture", UniformType::Sampler2d)
        .with_extra_uniform("normal_texture", UniformType::Sampler2d)
        .with_in_def(v_light_pos())
        .with_in_def(v_light_color())
        .with_in_def(v_light_attenuation())
        .with_body(
            "
            vec2 tex_coord = gl_FragCoord.xy / viewport.zw;
            vec3 position = texture(position_texture, tex_coord).xyz;
            vec3 normal = texture(normal_texture, tex_coord).xyz;

            vec3 light_vector = v_light_pos - position;
            float light_distance_sq = dot(light_vector, light_vector);
            float light_distance = sqrt(light_distance_sq);

            float diffuse = max(dot(normal, light_vector / light_distance), 0.0);
            float attenuation = 1.0 / dot(
                v_light_attenuation,
                vec3(1, light_distance, light_distance_sq)
            );
            diffuse *= max(attenuation, 0.0);

            float radiance = diffuse;
            ",
        )
        .with_out(
            shader::defs::f_color(),
            "vec4(v_light_color * radiance, 1.0)",
        )
}

/// Shader core for rendering a light source, given the position/normal buffers
/// from the scene pass.
pub fn main_light_screen_quad_core(
    have_shadows: bool,
) -> shader::Core<Camera, Light, screen_quad::Vertex> {
    let vertex = shader::VertexCore::default()
        .with_out(v_light_pos(), "light_position")
        .with_out(v_light_color(), "light_color")
        .with_out(v_light_attenuation(), "light_attenuation")
        .with_out_expr(shader::defs::V_POSITION, "position");

    let mut fragment = light_fragment_core();
    if have_shadows {
        fragment = fragment
            .with_extra_uniform("shadow_texture", UniformType::Sampler2d)
            .with_body(
                "
                radiance *= texture(shadow_texture, tex_coord).r;
                ",
            );
    }

    shader::Core { vertex, fragment }
}

pub fn light_object_core() -> shader::Core<Camera, Light, basic_obj::Vertex> {
    let vertex = shader::VertexCore::default()
        .with_out(v_light_pos(), "light_position")
        .with_out(v_light_color(), "light_color")
        .with_out(v_light_attenuation(), "light_attenuation")
        .with_out_expr(
            shader::defs::V_POSITION,
            "
                mat_projection
                * mat_view
                * (vec4(position * light_radius, 1.0) + vec4(light_position, 0))
            ",
        );

    shader::Core {
        vertex,
        fragment: light_fragment_core(),
    }
}

/// Composition shader core transform for composing our buffers.
pub fn composition_core_transform(
    core: shader::Core<(), (), screen_quad::Vertex>,
) -> shader::Core<(), (), screen_quad::Vertex> {
    assert!(
        core.fragment.has_in(shader::defs::V_TEX_COORD),
        "FragmentCore needs V_TEX_COORD input for deferred shading composition pass"
    );
    assert!(
        core.fragment.has_out(shader::defs::F_COLOR),
        "FragmentCore needs F_COLOR output for deferred shading composition pass"
    );

    let light_expr = "texture(light_texture, v_tex_coord).rgb";
    let ambient_expr = "vec3(0.3, 0.3, 0.3)";

    let fragment = core
        .fragment
        .with_extra_uniform("light_texture", UniformType::Sampler2d)
        .with_out_expr(
            shader::defs::F_COLOR,
            &format!("f_color * vec4({} + {}, 1.0)", light_expr, ambient_expr),
        );

    shader::Core {
        vertex: core.vertex,
        fragment,
    }
}

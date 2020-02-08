use glium::uniforms::UniformType;

use crate::pipeline::Light;
use crate::{basic_obj, screen_quad, shader, Camera, Context};

pub const F_WORLD_POS: (&str, shader::FragmentOutDef) = (
    "f_world_pos",
    shader::FragmentOutDef(shader::Type::FloatVec4, shader::FragmentOutQualifier::Yield),
);

pub const F_WORLD_NORMAL: (&str, shader::FragmentOutDef) = (
    "f_world_normal",
    shader::FragmentOutDef(shader::Type::FloatVec4, shader::FragmentOutQualifier::Yield),
);

/// Shader core transform for writing position/normal/color into separate
/// buffers, so that they may be combined in a subsequent pass.
pub fn scene_buffers_core_transform<P, I, V>(
    always_include_shadow_out: bool,
    core: shader::Core<P, I, V>,
) -> shader::Core<P, I, V> {
    assert!(
        core.vertex.has_out_def(shader::defs::V_WORLD_POS),
        "VertexCore needs V_WORLD_POS output for deferred shading scene pass"
    );
    assert!(
        core.vertex.has_out_def(shader::defs::V_WORLD_NORMAL),
        "VertexCore needs V_WORLD_NORMAL output for deferred shading scene pass"
    );
    assert!(
        core.fragment.has_out_def(shader::defs::F_COLOR),
        "FragmentCore needs F_COLOR output for deferred shading scene pass"
    );

    let mut fragment = core
        .fragment
        .with_in_def(shader::defs::V_WORLD_POS)
        .with_in_def(shader::defs::V_WORLD_NORMAL)
        .with_out(F_WORLD_POS, "v_world_pos")
        .with_out(F_WORLD_NORMAL, "vec4(v_world_normal, 0.0)");

    // We may have the case that we want to attach an `f_shadow` output, but
    // the given `core` does not provide any shadow values (i.e. it wants to
    // be unshadowed). In that case, we still need to provide a shadow value.
    if always_include_shadow_out && !fragment.has_out("f_shadow") {
        fragment = fragment.with_out(shader::defs::F_SHADOW, "1.0");
    }

    // This is a bit sneaky: we turn `f_shadow` from a local variable into
    // something that is output by the fragment shader.
    fragment = fragment.with_out_def((
        "f_shadow",
        shader::FragmentOutDef(shader::Type::Float, shader::FragmentOutQualifier::Yield),
    ));

    shader::Core {
        vertex: core.vertex,
        fragment,
    }
}

const V_LIGHT_POS: (&str, shader::VertexOutDef) = (
    "v_light_pos",
    shader::VertexOutDef(shader::Type::FloatVec3, shader::VertexOutQualifier::Flat),
);

const V_LIGHT_COLOR: (&str, shader::VertexOutDef) = (
    "v_light_color",
    shader::VertexOutDef(shader::Type::FloatVec3, shader::VertexOutQualifier::Flat),
);

const V_LIGHT_ATTENUATION: (&str, shader::VertexOutDef) = (
    "v_light_attenuation",
    shader::VertexOutDef(shader::Type::FloatVec4, shader::VertexOutQualifier::Flat),
);

fn light_fragment_core() -> shader::FragmentCore<Camera> {
    shader::FragmentCore::empty()
        .with_extra_uniform("position_texture", UniformType::Sampler2d)
        .with_extra_uniform("normal_texture", UniformType::Sampler2d)
        .with_in_def(V_LIGHT_POS)
        .with_in_def(V_LIGHT_COLOR)
        .with_in_def(V_LIGHT_ATTENUATION)
        .with_body(
            "
            vec2 tex_coord = gl_FragCoord.xy / camera_viewport_size;
            vec3 position = texture(position_texture, tex_coord).xyz;
            vec3 normal = texture(normal_texture, tex_coord).xyz;

            vec3 light_vector = v_light_pos - position;
            float light_distance_sq = dot(light_vector, light_vector);
            float light_distance = sqrt(light_distance_sq);

            float diffuse = max(dot(normal, light_vector / light_distance), 0.0);
            float attenuation = dot(
                v_light_attenuation.xyz,
                vec3(1, light_distance, light_distance_sq)
            ) * exp(v_light_attenuation.w * light_distance_sq);
            diffuse /= attenuation;

            // Discarding here means that additive blending does not need to
            // be performed. This got me a speed-up in scenes with many lights.
            if (diffuse < 0.0001)
                discard;

            float radiance = diffuse;
            //float radiance = 1.0;
            ",
        )
        .with_out(shader::defs::F_COLOR, "vec4(v_light_color * radiance, 1.0)")
}

/// Shader core for rendering a light source, given the position/normal buffers
/// from the scene pass.
pub fn main_light_screen_quad_core(
    have_shadows: bool,
) -> shader::Core<Camera, Light, screen_quad::Vertex> {
    let vertex = shader::VertexCore::default()
        .with_out(V_LIGHT_POS, "light_position")
        .with_out(V_LIGHT_COLOR, "light_color")
        .with_out(V_LIGHT_ATTENUATION, "light_attenuation")
        .with_out(shader::defs::V_POS, "position");

    let mut fragment = light_fragment_core();
    if have_shadows {
        fragment = fragment
            .with_extra_uniform("shadow_texture", UniformType::Sampler2d)
            .with_out_expr(
                "f_color",
                "vec4(f_color.rgb * texture(shadow_texture, tex_coord).r, 1.0)",
            );
    }

    shader::Core { vertex, fragment }
}

pub fn light_object_core() -> shader::Core<Camera, Light, basic_obj::Vertex> {
    let vertex = shader::VertexCore::default()
        .with_out(V_LIGHT_POS, "light_position")
        .with_out(V_LIGHT_COLOR, "light_color")
        .with_out(V_LIGHT_ATTENUATION, "light_attenuation")
        .with_out(
            shader::defs::V_POS,
            "
                camera_projection
                * camera_view
                * (vec4(position * light_radius, 1.0) + vec4(light_position, 0))
            ",
        );

    let fragment = light_fragment_core();

    // Uncomment the following line to debug light volumes:
    //let fragment = fragment.with_out_expr("f_color", "vec4(1, 1, 1, 1)");

    shader::Core { vertex, fragment }
}

/// Composition shader core transform for composing our buffers.
pub fn composition_core_transform(
    core: shader::Core<Context, (), screen_quad::Vertex>,
) -> shader::Core<Context, (), screen_quad::Vertex> {
    assert!(
        core.fragment.has_in_def(shader::defs::V_TEX_COORD),
        "FragmentCore needs V_TEX_COORD input for deferred shading composition pass"
    );
    assert!(
        core.fragment.has_out_def(shader::defs::F_COLOR),
        "FragmentCore needs F_COLOR output for deferred shading composition pass"
    );

    let fragment = core
        .fragment
        .with_extra_uniform("light_texture", UniformType::Sampler2d)
        .with_extra_uniform("normal_texture", UniformType::Sampler2d)
        .with_body(
            "
            vec4 light_value = texture(light_texture, v_tex_coord);
            vec4 normal_value = texture(normal_texture, v_tex_coord);

            vec4 lighting = vec4(light_value.rgb + context_ambient_light, 1.0);

            // Keep background color as-is.
            // TODO: There are definitely more efficient ways to do this,
            // without having to read the normal texture.
            lighting += step(0.001, 1.0 - length(normal_value.rgb)) * vec4(1.0, 1.0, 1.0, 0.0);
            ",
        )
        .with_out_expr("f_color", "f_color * lighting");

    shader::Core {
        vertex: core.vertex,
        fragment,
    }
}

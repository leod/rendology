use nalgebra as na;

use crate::scene::{BuildProgram, CoreInput};
use crate::shader::{InstanceInput, InstancingMode};

#[derive(Debug, Clone)]
pub struct Params {
    pub time: f32,
}

impl_uniform_input!(
    Params,
    self => {
        params_time: f32 = self.time,
    }
);

#[derive(Debug, Clone)]
pub struct Particle {
    pub spawn_time: f32,
    pub life_duration: f32,
    pub start_pos: na::Point3<f32>,
    pub velocity: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
    pub size: f32,
    pub friction: f32,
}

impl Particle {
    pub fn dead() -> Self {
        Self {
            spawn_time: -1.0,
            life_duration: 0.0,
            start_pos: na::Point3::origin(),
            velocity: na::Vector3::zeros(),
            color: na::Vector3::zeros(),
            size: 0.0,
            friction: 0.0,
        }
    }
}

impl_instance_input!(
    Particle,
    self => {
        particle_spawn_time: f32 = self.spawn_time,
        particle_life_duration: f32 = self.life_duration,
        particle_start_pos: [f32; 3] = self.start_pos.coords,
        particle_velocity: [f32; 3] = self.velocity,
        particle_color: [f32; 3] = self.color,
        particle_size: f32 = self.size,
        particle_friction: f32 = self.friction,
    },
);

#[derive(Debug, Clone)]
pub struct Shader;

impl CoreInput for Shader {
    type Params = Params;
    type Instance = ();
    type Vertex = <Particle as InstanceInput>::Vertex;
}

impl BuildProgram for Shader {
    fn build_program<F: glium::backend::Facade>(
        &self,
        facade: &F,
        _: InstancingMode,
    ) -> Result<glium::Program, glium::program::ProgramCreationError> {
        // We use the long form of `glium::Program` here to set `outputs_rgb`
        // to true. See `shader::LinkedCore::build_program` for more background.
        // Here, we additionally set `uses_point_size` to true.
        glium::Program::new(
            facade,
            glium::program::ProgramCreationInput::SourceCode {
                /*vertex_shader: VERTEX_SHADER_2,
                fragment_shader: FRAGMENT_SHADER_2,
                geometry_shader: None,*/
                vertex_shader: VERTEX_SHADER,
                fragment_shader: FRAGMENT_SHADER,
                geometry_shader: Some(GEOMETRY_SHADER),
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )
    }
}

const VERTEX_SHADER: &str = "
    #version 330
    uniform mat4 context_camera_view;
    uniform float params_time;

    in float particle_spawn_time;
    in float particle_life_duration;
    in vec3 particle_start_pos;
    in vec3 particle_velocity;
    in vec3 particle_color;
    in float particle_size;
    in float particle_friction;

    out VertexData {
        vec4 color;
        float size;
    } vertex_out;

    void main() {
        float delta_time = params_time - particle_spawn_time;

        // Integrate velocity.
        vec3 current_pos = particle_start_pos
            + particle_velocity * delta_time
            - 0.5 * particle_friction * delta_time * delta_time * normalize(particle_velocity);
        gl_Position = context_camera_view * vec4(current_pos, 1);

        // Forward particle properties to geometry shader.
        vertex_out.color = vec4(particle_color, 1.0 - pow(delta_time / particle_life_duration, 3.0));
        vertex_out.size = particle_size;
    }
";

const GEOMETRY_SHADER: &str = "
    #version 330

    uniform mat4 context_camera_projection;

    layout (points) in;
    layout (triangle_strip, max_vertices=4) out;

    in VertexData {
        vec4 color;
        float size;
    } vertex_in[];

    out VertexData {
        vec4 color;
        vec2 uv;
    } vertex_out;

    void main() {
        // If the particle is alive, generate a camera-aligned quad.
        if (vertex_in[0].color.a > 0.0 && vertex_in[0].color.a <= 1.0) {
            vec4 center = gl_in[0].gl_Position;
            float size = vertex_in[0].size * vertex_in[0].color.a;

            gl_Position = context_camera_projection * (center + vec4(-size, -size, 0, 0));
            vertex_out.color = vertex_in[0].color;
            vertex_out.uv = vec2(-1, -1);
            EmitVertex();

            gl_Position = context_camera_projection * (center + vec4(size, -size, 0, 0));
            vertex_out.color = vertex_in[0].color;
            vertex_out.uv = vec2(1, -1);
            EmitVertex();

            gl_Position = context_camera_projection * (center + vec4(-size, size, 0, 0));
            vertex_out.color = vertex_in[0].color;
            vertex_out.uv = vec2(-1, 1);
            EmitVertex();

            gl_Position = context_camera_projection * (center + vec4(size, size, 0, 0));
            vertex_out.color = vertex_in[0].color;
            vertex_out.uv = vec2(1, 1);
            EmitVertex();

            EndPrimitive();
        }
    }
";

const FRAGMENT_SHADER: &str = "
    #version 330

    in VertexData {
        vec4 color;
        vec2 uv;
    } vertex_in;

    out vec4 target;

    void main() {
        float circle = max(1 - dot(vertex_in.uv, vertex_in.uv), 0);
        target = vertex_in.color;
        target.a *= circle;
    }
";

// The below version does not use a geometry shader, instead relying on
// `GL_POINTS`. It is not finished, since it does not correct the
// `gl_PointSize` for perspective.
//
// Unfortunately, it is not significantly faster than using the geometry shader
// on my GPU (NVIDIA GTX 1070).
/*
const VERTEX_SHADER_2: &str = "
    #version 330

    uniform mat4 context_camera_view;
    uniform mat4 context_camera_projection;
    uniform float params_time;

    in float particle_spawn_time;
    in float particle_life_duration;
    in vec3 particle_start_pos;
    in vec3 particle_velocity;
    in vec3 particle_color;
    in float particle_size;
    in float particle_friction;

    out vec4 v_color;

    void main() {
        float delta_time = params_time - particle_spawn_time;

        if (delta_time >= 0.0 && delta_time <= particle_life_duration) {
            v_color = vec4(particle_color, 1.0 - pow(delta_time / particle_life_duration, 3.0));

            // Integrate velocity together with friction.
            vec3 current_pos = particle_start_pos
                + particle_velocity * delta_time
                - 0.5 * particle_friction * delta_time * delta_time * normalize(particle_velocity);
            gl_Position = context_camera_projection * context_camera_view * vec4(current_pos, 1);
            //gl_Position.z -= 100.0 * float(delta_time < 0.0 || delta_time > particle_life_duration);

            //gl_PointSize = particle_size * 3.0 * (1.0 - v_color.a) / gl_Position.w;
            gl_PointSize = 0.1;
        } else {
            gl_Position = vec4(100.0, 100.0, 100.0, -100.0);
        }
    }
";

const FRAGMENT_SHADER_2: &str = "
    #version 330

    in vec4 v_color;
    out vec4 f_color;

    void main() {
        vec2 uv = gl_PointCoord;
        float circle = max(1 - dot(uv, uv), 0);

        f_color = v_color;
        f_color.a *= circle;
    }
";
*/

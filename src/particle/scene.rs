use nalgebra as na;

use crate::scene::{CoreInput, BuildProgram};
use crate::shader::{InstancingMode, InstanceInput};

const VERTEX_SHADER: &str = "
";

const GEOMETRY_SHADER: &str = "
";

const FRAGMENT_SHADER: &str = "
";

#[derive(Debug, Clone)]
pub struct Params {
    pub time: f32,
}

impl_uniform_input!(
    Params,
    self => {
        time: f32 = self.time,
    }
);

#[derive(Debug, Clone)]
pub struct Particle {
    pub spawn_time: f32,
    pub life_duration: f32,
    pub start_pos: na::Vector3<f32>,
    pub velocity: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
}

impl_instance_input!(
    Particle,
    self => {
        spawn_time: f32 = self.spawn_time,
        life_duration: f32 = self.life_duration,
        start_pos: [f32; 3] = self.start_pos,
        velocity: [f32; 3] = self.velocity,
        color: [f32; 3] = self.color,
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
        &self, facade: &F, _: InstancingMode,
    ) -> Result<glium::Program, glium::program::ProgramCreationError> {
        // We use the long form of `glium::Program` here to set `outputs_rgb`
        // to true. See `shader::LinkedCore::build_program` for more background.
        glium::Program::new(
            facade,
            glium::program::ProgramCreationInput::SourceCode {
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


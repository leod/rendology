use crate::shader::{self, InstanceInput, UniformInput};
use crate::{Context, InstancingMode};

pub trait BuildProgram {
    fn build_program<F: glium::backend::Facade>(
        &self,
        facade: &F,
        instancing_mode: InstancingMode,
    ) -> Result<glium::Program, glium::program::ProgramCreationError>;
}

pub trait CoreInput {
    type Params: UniformInput + Clone;
    type Instance: InstanceInput + Clone;
    type Vertex: glium::vertex::Vertex;
}

pub trait SceneCore: CoreInput {
    fn scene_core(
        &self,
    ) -> shader::Core<
        (Context, <Self as CoreInput>::Params),
        <Self as CoreInput>::Instance,
        <Self as CoreInput>::Vertex,
    >;
}

impl<T: SceneCore> BuildProgram for T {
    fn build_program<F: glium::backend::Facade>(
        &self,
        facade: &F,
        instancing_mode: InstancingMode,
    ) -> Result<glium::Program, glium::program::ProgramCreationError> {
        // TODO: Dropping error detail here...
        self.scene_core()
            .build_program(facade, instancing_mode)
            .map_err(|e| e.error)
    }
}

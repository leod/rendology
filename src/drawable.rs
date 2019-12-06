use crate::shader::{InstancingMode, ToUniforms};
use crate::DrawError;

pub trait Drawable<I, V>
where
    V: glium::vertex::Vertex,
{
    /// The instancing mode supported by this `Drawable`.
    ///
    /// We use this to check that the supplied `glium::Program` is compatible
    /// with the `Drawable`.
    const INSTANCING_MODE: InstancingMode;

    fn draw<U, S>(
        &self,
        program: &glium::Program,
        uniforms: &U,
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        U: ToUniforms,
        S: glium::Surface;
}

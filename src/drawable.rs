use crate::{DrawError, Mesh};

use crate::shader::ToVertex;

pub trait Drawable<I, V>
where
    V: glium::vertex::Vertex,
{
    fn draw<U, S>(
        &self,
        program: &glium::Program,
        uniforms: &U,
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        U: glium::uniforms::Uniforms,
        S: glium::Surface;
}

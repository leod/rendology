pub use crate::CreationError;

pub enum IndexBuffer {
    IndexBuffer(glium::index::IndexBuffer<u32>),
    NoIndices(glium::index::NoIndices),
}

impl IndexBuffer {
    pub fn draw<'a, V, U, S>(
        &self,
        vertex_buffer: V,
        program: &glium::Program,
        uniforms: &U,
        draw_parameters: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), glium::DrawError>
    where
        V: glium::vertex::MultiVerticesSource<'a>,
        U: glium::uniforms::Uniforms,
        S: glium::Surface,
    {
        match &self {
            IndexBuffer::IndexBuffer(buffer) => {
                target.draw(vertex_buffer, buffer, program, uniforms, draw_parameters)
            }
            IndexBuffer::NoIndices(buffer) => {
                target.draw(vertex_buffer, buffer, program, uniforms, draw_parameters)
            }
        }
    }
}

pub struct Mesh<V: Copy> {
    pub vertex_buffer: glium::VertexBuffer<V>,
    pub index_buffer: IndexBuffer,
}

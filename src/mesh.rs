pub use crate::CreationError;

pub enum IndexBuffer {
    IndexBuffer(glium::index::IndexBuffer<u32>),
    NoIndices(glium::index::NoIndices),
}

impl<'a> From<&'a IndexBuffer> for glium::index::IndicesSource<'a> {
    fn from(buffer: &'a IndexBuffer) -> Self {
        match buffer {
            IndexBuffer::IndexBuffer(buffer) => buffer.into(),
            IndexBuffer::NoIndices(buffer) => buffer.into(),
        }
    }
}

pub struct Mesh<V: Copy> {
    pub vertex_buffer: glium::VertexBuffer<V>,
    pub index_buffer: IndexBuffer,
}

impl<V: glium::vertex::Vertex> Mesh<V> {
    pub fn create_with_indices<F: glium::backend::Facade>(
        facade: &F,
        primitive_type: glium::index::PrimitiveType,
        vertices: &[V],
        indices: &[u32],
    ) -> Result<Self, CreationError> {
        Ok(Mesh {
            vertex_buffer: glium::VertexBuffer::new(facade, vertices)?,
            index_buffer: IndexBuffer::IndexBuffer(glium::IndexBuffer::new(
                facade,
                primitive_type,
                indices,
            )?),
        })
    }
}

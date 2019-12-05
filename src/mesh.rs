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

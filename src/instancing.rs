use log::info;

use crate::shader::{InstanceInput, InstancingMode, ToUniforms};
use crate::{Drawable, Mesh};

pub use crate::error::{CreationError, DrawError};

pub const INSTANCES_PER_BUFFER: usize = 1000;

struct Buffer<V: Copy> {
    buffer: glium::VertexBuffer<V>,
    num_used: usize,
}

impl<V> Buffer<V>
where
    V: glium::vertex::Vertex,
{
    fn create<F: glium::backend::Facade>(facade: &F) -> Result<Self, CreationError> {
        let buffer = glium::VertexBuffer::empty_dynamic(facade, INSTANCES_PER_BUFFER)?;

        Ok(Self {
            buffer,
            num_used: 0,
        })
    }

    fn clear(&mut self) {
        self.num_used = 0;
    }

    fn remaining_capacity(&self) -> usize {
        assert!(self.num_used <= self.buffer.len());

        self.buffer.len() - self.num_used
    }

    fn append(&mut self, vertices: &[V]) -> usize {
        let capacity = self.remaining_capacity();

        if capacity == 0 {
            return 0;
        }

        let start_index = self.buffer.len() - capacity;
        let num_to_write = vertices.len().min(capacity);
        let range = start_index..start_index + num_to_write;

        // Safe to unwrap since we have bounded the range to our capacity.
        let slice = self.buffer.slice_mut(range).unwrap();
        slice.write(&vertices[0..num_to_write]);

        self.num_used += num_to_write;

        num_to_write
    }
}

pub struct Instancing<V: InstanceInput> {
    buffers: Vec<Buffer<V::Vertex>>,
}

impl<I: InstanceInput> Instancing<I> {
    pub fn create<F: glium::backend::Facade>(facade: &F) -> Result<Self, CreationError> {
        let buffers = vec![Buffer::create(facade)?];

        Ok(Self { buffers })
    }

    pub fn update<F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        mut instances: &[I::Vertex],
    ) -> Result<(), CreationError> {
        // Write instance data into vertex buffers. We move through the buffers
        // that we have, filling them up sequentially.
        for buffer in &mut self.buffers {
            buffer.clear();
        }

        let mut cur_buffer = 0;

        while !instances.is_empty() {
            // Write as much as possible into the current buffer.
            let num_written = self.buffers[cur_buffer].append(instances);

            if num_written == 0 {
                // We had instance data to write, but nothing was written
                // Move on to the next buffer.
                cur_buffer += 1;

                if cur_buffer == self.buffers.len() {
                    // We have reached past the last buffer. Create a new
                    // buffer to write into.
                    self.buffers.push(Buffer::create(facade)?);

                    info!(
                        "Created new vertex buffer for `I={}`",
                        std::any::type_name::<I>()
                    );
                }
            } else {
                // We have written something into the buffer, reduce slice
                // accordingly.
                instances = &instances[num_written..];
            }
        }

        Ok(())
    }

    pub fn as_drawable<'a, V: glium::vertex::Vertex>(
        &'a self,
        mesh: &'a Mesh<V>,
    ) -> impl Drawable<I, V> + 'a {
        DrawableImpl(self, mesh)
    }
}

struct DrawableImpl<'a, I: InstanceInput, V: Copy>(&'a Instancing<I>, &'a Mesh<V>);

impl<'a, I, V> Drawable<I, V> for DrawableImpl<'a, I, V>
where
    I: InstanceInput,
    V: glium::vertex::Vertex,
{
    const INSTANCING_MODE: InstancingMode = InstancingMode::Vertex;

    fn draw<U, S>(
        &self,
        program: &glium::Program,
        uniforms: &U,
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError>
    where
        U: ToUniforms,
        S: glium::Surface,
    {
        let uniforms = uniforms.to_uniforms();

        for buffer in self.0.buffers.iter() {
            if buffer.num_used == 0 {
                // Buffers are filled sequentially, so we can exit early here.
                return Ok(());
            }

            // Safe to unwrap here, since we assure that `num_used < buffer.len()`.
            let instances = buffer.buffer.slice(0..buffer.num_used).unwrap();

            // TODO: Fall back to non-instanced rendering if not supported?
            let per_instance = instances
                .per_instance()
                .map_err(|_| DrawError::InstancingNotSupported)?;

            target.draw(
                (&self.1.vertex_buffer, per_instance),
                &self.1.index_buffer,
                program,
                &uniforms,
                draw_params,
            )?;
        }

        Ok(())
    }
}

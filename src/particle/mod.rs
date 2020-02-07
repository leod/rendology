//! A first attempt to get particles into rendology.
//!
//! For now I'll just try to reimplement particle-frenzy:
//! https://github.com/leod/particle-frenzy
//!
//! I'm not sure if it makes sense to include particles in deferred shading,
//! but it seems overkill for now. Unfortunately, that would be somewhat
//! difficult as of now anyway, since geometry shaders are not part of the
//! design of `shader::Core`.
//!
//! As a result of lacking `shader::Core` functionality, the particles are
//! currently almost completely non-configurable. We'll figure this out and
//! extend it as we go.

mod scene;

use crate::error::{CreationError, DrawError};
use crate::shader::{InstanceInput, InstancingMode, ToUniforms};
use crate::Drawable;

pub use scene::{Params, Particle, Shader};

/// Keeps a buffer of particle vertices.
struct Buffer {
    buffer: glium::VertexBuffer<<Particle as InstanceInput>::Vertex>,

    /// Time at which our most long-living particle dies.
    max_death_time: f32,
}

impl Buffer {
    fn create<F: glium::backend::Facade>(facade: &F, size: usize) -> Result<Self, CreationError> {
        let buffer = glium::VertexBuffer::empty_dynamic(facade, size)?;

        Ok(Self {
            buffer,
            max_death_time: 0.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub particles_per_buffer: usize,
    pub num_buffers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            particles_per_buffer: 20000,
            num_buffers: 30,
        }
    }
}

/// A particle system manages multiple buffers to store particles and allows
/// for rendering them.
pub struct System {
    /// Our configuration.
    config: Config,

    /// A ring buffer of particles.
    buffers: Vec<Buffer>,

    /// The position at which the next particle will be inserted in our buffers.
    ///
    /// The first element is an index into `buffers`, the second element is an
    /// index into that buffer.
    ///
    /// `next_index` must always be valid.
    next_index: (usize, usize),

    /// Current time as supplied by the user. Used to render only buffers that
    /// contain at least one alive particle.
    current_time: f32,
}

impl System {
    pub fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &Config,
    ) -> Result<Self, CreationError> {
        assert!(config.particles_per_buffer > 0);
        assert!(config.num_buffers > 0);

        let buffers = (0..config.num_buffers)
            .map(|_| Buffer::create(facade, config.particles_per_buffer))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            config: config.clone(),
            buffers,
            next_index: (0, 0),
            current_time: 0.0,
        })
    }

    pub fn shader(&self) -> Shader {
        Shader
    }

    pub fn set_current_time(&mut self, current_time: f32) {
        self.current_time = current_time;
    }

    pub fn spawn(&mut self, mut particles: &[<Particle as InstanceInput>::Vertex]) {
        // Copy new particles, filling up the ring buffer.
        while !particles.is_empty() {
            // By our invariant, `self.next_index` always is valid.
            assert!(self.next_index.0 < self.buffers.len());
            assert!(self.next_index.1 < self.config.particles_per_buffer);

            // Contiguously fill up the current buffer as much as possible.
            let capacity = self.config.particles_per_buffer - self.next_index.1;
            let num_to_write = particles.len().min(capacity);
            assert!(num_to_write > 0);

            let slice_to_write = &particles[0..num_to_write];
            let target_buffer = &mut self.buffers[self.next_index.0];

            target_buffer
                .buffer
                .slice_mut(self.next_index.1..self.next_index.1 + num_to_write)
                .unwrap() // safe to unwrap, since range bounded to capacity.
                .write(slice_to_write);

            // Keep track of how alive the buffer is. This allows us to ignore
            // buffers containing only dead particles when rendering.
            let new_max_death_time = slice_to_write
                .iter()
                .map(|particle| particle.particle_spawn_time + particle.particle_life_duration)
                .fold(0.0, f32::max);
            assert!(!new_max_death_time.is_nan());

            target_buffer.max_death_time = target_buffer.max_death_time.max(new_max_death_time);

            // Determine the next particle writing point.
            self.next_index = if num_to_write == capacity {
                // The current buffer is now fully written. Move on to the
                // next buffer in the ring.
                let buffer_index = (self.next_index.0 + 1) % self.buffers.len();
                (buffer_index, 0)
            } else {
                // Stay in the current buffer, advancing the inner index.
                (self.next_index.0, self.next_index.1 + num_to_write)
            };

            // Reduce the slice of particles to write for the next iteration.
            particles = &particles[num_to_write..];
        }
    }

    pub fn clear(&mut self) {
        for buffer in self.buffers.iter_mut() {
            buffer.max_death_time = 0.0;

            let mut mapping = buffer.buffer.map_write();
            for i in 0..mapping.len() {
                mapping.set(i, Particle::dead().to_vertex());
            }
        }
    }
}

impl Drawable<(), <Particle as InstanceInput>::Vertex> for System {
    const INSTANCING_MODE: InstancingMode = InstancingMode::Uniforms;

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
        for buffer in self.buffers.iter() {
            if buffer.max_death_time <= self.current_time {
                // This buffer does not contain any alive particles, so we can
                // completely skip it.
                continue;
            }

            // TODO: If we insert particles in a smart order, we might be able
            // to render slices of the buffer here if it is partially alive.
            // This is probably not worth the effort for now.

            target.draw(
                &buffer.buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                program,
                &uniforms.to_uniforms(),
                draw_params,
            )?;
        }

        Ok(())
    }
}

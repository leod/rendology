//! A first attempt to get particles into rendology.
//!
//! For now I'll just try to reimplement particle-frenzy:
//! https://github.com/leod/particle-frenzy
//!
//! I'm not sure if it makes sense to include particles in deferred shading,
//! but it seems overkill for now. Unfortunately, that would be somewhat
//! difficult as of now anyway, since geometry shaders are not part of the
//! design of `shader::Core`.

mod scene;

use nalgebra as na;

use crate::shader::InstanceInput;

pub use scene::{Params, Particle, Shader};

struct Buffer {
    buffer: glium::VertexBuffer<<Particle as InstanceInput>::Vertex>,
}

pub struct System {
    new_particles: Vec<Particle>
}

impl System {
    pub fn shader(&self) -> Shader {
        Shader
    }

    #[inline]
    pub fn spawn(&mut self, particle: Particle) {
        self.new_particles.push(particle);
    }
}

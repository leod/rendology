use log::info;

use num_traits::{FromPrimitive, ToPrimitive};

use crate::object::{self, Object, ObjectBuffers};
use crate::{scene, shader};

pub use crate::CreationError;

pub struct Resources {
    pub object_buffers: Vec<ObjectBuffers<object::Vertex>>,
}

impl Resources {
    pub fn create<F: glium::backend::Facade>(facade: &F) -> Result<Resources, CreationError> {
        // Unfortunately, it doesn't seem easy to use enum_map here,
        // since we need to check for errors in creating buffers
        let mut object_buffers = Vec::new();

        for i in 0..Object::NumTypes as usize {
            // Safe to unwrap here, since we iterate within the range
            let object: Object = FromPrimitive::from_usize(i).unwrap();

            object_buffers.push(object.create_buffers(facade)?);
        }

        Ok(Resources { object_buffers })
    }

    pub fn object_buffers(&self, object: Object) -> &ObjectBuffers<object::Vertex> {
        // Safe to unwrap array access here, since we have initialized buffers
        // for all objects
        &self.object_buffers[object.to_usize().unwrap()]
    }
}

mod mesh;
mod scene;

use std::ops::{Index, IndexMut};
use std::path::Path;

use glium::{self, implement_vertex};
use log::info;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::{CreationError, DrawError, Mesh};

pub use mesh::{load_wavefront, mesh_from_slices, CUBE_INDICES, CUBE_NORMALS, CUBE_POSITIONS};
pub use scene::{Core, Instance};

#[derive(Copy, Clone, PartialEq, Eq, Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum BasicObject {
    Triangle,
    Quad,
    Cube,
    Sphere,

    LineX,
    LineY,
    LineZ,

    TessellatedCube,
    TessellatedCylinder,
}

pub const NUM_TYPES: usize = 9;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

pub struct Resources {
    pub meshes: Vec<Mesh<Vertex>>,
}

impl Resources {
    pub fn create<F: glium::backend::Facade>(facade: &F) -> Result<Resources, CreationError> {
        // Unfortunately, it doesn't seem easy to use enum_map here,
        // since we need to check for errors in creating buffers
        let mut meshes = Vec::new();

        for i in 0..NUM_TYPES {
            // Safe to unwrap here, since we iterate within the range
            let object: BasicObject = num_traits::FromPrimitive::from_usize(i).unwrap();

            meshes.push(object.create_mesh(facade)?);
        }

        Ok(Resources { meshes })
    }

    pub fn mesh(&self, object: BasicObject) -> &Mesh<Vertex> {
        // Safe to unwrap since `BasicObject::to_usize()` never fails.
        &self.meshes[object.to_usize().unwrap()]
    }
}

impl BasicObject {
    #[rustfmt::skip]
    pub fn create_mesh<F: glium::backend::Facade>(
        self,
        facade: &F,
    ) -> Result<Mesh<Vertex>, CreationError> {
        mesh::create_mesh(self, facade)
    }
}

/*pub struct RenderList<I: ToVertex>(Vec<crate::RenderList<I>>);

impl<I: ToVertex> Default for RenderList<I> {
    fn default() -> Self {
        Self(vec![Default::default(); NUM_TYPES])
    }
}

impl<I: ToVertex> Index<BasicObject> for RenderList<I> {
    fn index(&self, object: BasicObject) -> &crate::RenderList<I> {
        // Safe to unwrap since `BasicObject::to_usize()` never fails.
        &self.0[object.to_usize().unwrap()]
    }
}

impl<I: ToVertex> IndexMut<BasicObject> for RenderList<I> {
    fn index_mut(&mut self, object: BasicObject) -> &mut crate::RenderList<I> {
        // Safe to unwrap since `BasicObject::to_usize()` never fails.
        &mut self.0[object.to_usize().unwrap()]
    }
}

pub struct Instancing<I: ToVertex>(Vec<crate::Instancing<I>>);

impl<I: ToVertex> Instancing<I> {
    pub fn update<'a, F: glium::backend::Facade>(
        &mut self,
        facade: &F,
        render_list: &RenderList<I>,
    ) -> Result<(), CreationError> {
        for i in 0..NUM_TYPES {
            self.0.update(render_list.0[i].to_slice())?;
        }

        Ok(())
    }

    pub fn as_draw_instances(&self, resources: &Resources) -> DrawInstances<'_, I> {
        DrawInstances(&self, resources)
    }
}

pub struct DrawInstances<'a, I: ToVertex>(&'a Instancing<I>, &'a Resources);

impl<'a, I: ToVertex> DrawInstances<I> for DrawInstancing<'a, I> {
    fn draw_instances<U, W, S>(
        &self,
        object: &Mesh<W>,
        program: &glium::Program,
        uniforms: &U,
        draw_params: &glium::DrawParameters,
        target: &mut S,
    ) -> Result<(), DrawError> {
        for i in 0..NUM_TYPES {
            self.0[i].draw_instances(
        }
    }
}*/

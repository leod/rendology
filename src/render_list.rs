use crate::shader::{InstanceInput, InstancingMode, ToUniforms};
use crate::{DrawError, Drawable, Mesh};

#[derive(Clone)]
pub struct RenderList<I: InstanceInput> {
    instances: Vec<I::Vertex>,
}

impl<I: InstanceInput> Default for RenderList<I> {
    fn default() -> Self {
        RenderList {
            instances: Vec::new(),
        }
    }
}

impl<I: InstanceInput> RenderList<I> {
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn as_slice(&self) -> &[I::Vertex] {
        &self.instances
    }

    pub fn add(&mut self, params: I) {
        self.instances.push(params.to_vertex());
    }

    pub fn as_drawable<'a, V: glium::vertex::Vertex>(
        &'a self,
        mesh: &'a Mesh<V>,
    ) -> impl Drawable<I, V> + 'a {
        DrawableImpl(self, mesh)
    }
}

struct DrawableImpl<'a, I: InstanceInput, V: Copy>(&'a RenderList<I>, &'a Mesh<V>);

impl<'a, I, V> Drawable<I, V> for DrawableImpl<'a, I, V>
where
    I: InstanceInput,
    V: glium::vertex::Vertex,
{
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
        for instance in &self.0.instances {
            target.draw(
                &self.1.vertex_buffer,
                &self.1.index_buffer,
                program,
                &(uniforms, instance).to_uniforms(),
                draw_params,
            )?;
        }

        Ok(())
    }
}

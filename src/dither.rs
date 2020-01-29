use crate::scene::{CoreInput, SceneCore};
use crate::{shader, Context};

pub struct Core<C: SceneCore>(pub C);

impl<C: SceneCore> CoreInput for Core<C> {
    type Params = C::Params;
    type Instance = C::Instance;
    type Vertex = C::Vertex;
}

impl<C: SceneCore> SceneCore for Core<C> {
    fn scene_core(&self) -> shader::Core<(Context, C::Params), C::Instance, C::Vertex> {
        let core = self.0.scene_core();

        let fragment = core
            .fragment
            .with_defs(
                "
                bool dither(vec2 p) {
                    return ((int(p.x) ^ int(p.y)) & 1) == 0;
                }
                ",
            )
            .with_body("if (!dither(gl_FragCoord.xy)) { discard; }");

        shader::Core {
            vertex: core.vertex,
            fragment,
        }
    }
}

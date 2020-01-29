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
                mat4 thresh = mat4(
                    1.0, 9.0, 3.0, 11.0,
                    13.0, 5.0, 15.0, 7.0,
                    4.0, 12.0, 2.0, 10.0,
                    16.0, 8.0, 14.0, 6.0
                ) / 17.0;

                bool dither(vec2 p, float alpha) {
                    return thresh[int(p.x) % 4][int(p.y) % 4] >= alpha;
                }
                ",
            )
            .with_body("if (dither(gl_FragCoord.xy, v_color.a)) { discard; }");

        shader::Core {
            vertex: core.vertex,
            fragment,
        }
    }
}

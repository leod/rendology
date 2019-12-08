use std::time::Instant;

use floating_duration::TimeAsFloat;
use glium::{glutin, Surface};
use nalgebra as na;

use rendology::{
    basic_obj, BasicObj, InstancingMode, Light, Mesh, RenderList, ShadedScenePass,
    ShadedScenePassSetup, ShadowPass,
};

const WINDOW_SIZE: (u32, u32) = (1280, 720);

mod my_scene {
    use nalgebra as na;
    use rendology::{basic_obj, shader, Context, SceneCore};

    #[derive(Clone)]
    pub struct Params<'a> {
        pub time: f32,
        pub texture: &'a glium::texture::CompressedSrgbTexture2d,
    }

    #[derive(Clone)]
    pub struct Instance {
        pub transform: na::Matrix4<f32>,
    }

    rendology::impl_uniform_input_with_lifetime!(
        Params<'a>,
        self => {
            time: f32 => self.time,
            texture: &'a glium::texture::CompressedSrgbTexture2d => self.texture,
        },
    );

    rendology::impl_instance_input!(
        Instance,
        self => {
            mat_model: [[f32; 4]; 4] => self.transform.into(),
        },
    );

    pub struct Core;

    impl SceneCore for Core {
        type Params = Params<'static>;
        type Instance = Instance;
        type Vertex = basic_obj::Vertex;

        fn scene_core(
            &self,
        ) -> shader::Core<(Context, Self::Params), Self::Instance, Self::Vertex> {
            let position = "position + 0.2 * sin(time) * sin(time) * normal";

            let vertex = shader::VertexCore::empty()
                .with_out(
                    shader::defs::v_world_normal(),
                    "normalize(transpose(inverse(mat3(mat_model))) * normal)",
                )
                .with_out(
                    shader::defs::v_world_pos(),
                    &format!("mat_model * vec4({}, 1.0)", position),
                )
                .with_out_expr(
                    shader::defs::V_POSITION,
                    "mat_projection * mat_view * v_world_pos",
                );
            let fragment = shader::FragmentCore::empty()
                .with_out(shader::defs::f_color(), "vec4(1.0, 0.0, 0.0, 1.0)");

            shader::Core { vertex, fragment }
        }
    }
}

#[derive(Default)]
struct Scene {
    time: f32,
    cubes: RenderList<basic_obj::Instance>,
    my_cubes: RenderList<my_scene::Instance>,
    lights: Vec<Light>,
}

struct Pipeline {
    rendology: rendology::Pipeline,

    shadow_pass: Option<ShadowPass<basic_obj::Core>>,
    scene_pass: ShadedScenePass<basic_obj::Core>,

    my_shadow_pass: Option<ShadowPass<my_scene::Core>>,
    my_scene_pass: ShadedScenePass<my_scene::Core>,

    cube: Mesh<basic_obj::Vertex>,
    texture: glium::texture::CompressedSrgbTexture2d,
}

impl Pipeline {
    fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &rendology::Config,
    ) -> Result<Self, rendology::pipeline::CreationError> {
        let rendology = rendology::Pipeline::create(facade, config, WINDOW_SIZE)?;

        let shadow_pass =
            rendology.create_shadow_pass(facade, basic_obj::Core, InstancingMode::Uniforms)?;
        let scene_pass = rendology.create_shaded_scene_pass(
            facade,
            basic_obj::Core,
            InstancingMode::Uniforms,
            ShadedScenePassSetup {
                draw_shadowed: true,
                draw_glowing: false,
            },
        )?;

        let my_shadow_pass =
            rendology.create_shadow_pass(facade, my_scene::Core, InstancingMode::Uniforms)?;
        let my_scene_pass = rendology.create_shaded_scene_pass(
            facade,
            my_scene::Core,
            InstancingMode::Uniforms,
            ShadedScenePassSetup {
                draw_shadowed: true,
                draw_glowing: false,
            },
        )?;

        let cube = BasicObj::Cube.create_mesh(facade)?;

        let texture = {
            let image = image::load(
                std::io::Cursor::new(&include_bytes!("texture.png")[..]),
                image::PNG,
            )
            .unwrap()
            .to_rgba();
            let dimensions = image.dimensions();
            let image =
                glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);

            glium::texture::CompressedSrgbTexture2d::new(facade, image).unwrap()
        };

        Ok(Pipeline {
            rendology,
            shadow_pass,
            scene_pass,
            my_shadow_pass,
            my_scene_pass,
            cube,
            texture,
        })
    }

    fn draw_frame<F: glium::backend::Facade, S: glium::Surface>(
        &mut self,
        facade: &F,
        context: &rendology::Context,
        scene: &Scene,
        target: &mut S,
    ) -> Result<(), rendology::DrawError> {
        let draw_params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let my_params = my_scene::Params {
            time: scene.time,
            texture: &self.texture,
        };

        self.rendology
            .start_frame(facade, context.clone(), target)?
            .shadow_pass()
            .draw(
                &self.shadow_pass,
                &scene.cubes.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .draw(
                &self.my_shadow_pass,
                &scene.my_cubes.as_drawable(&self.cube),
                &my_params,
                &Default::default(),
            )?
            .shaded_scene_pass()
            .draw(
                &self.scene_pass,
                &scene.cubes.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .draw(
                &self.my_scene_pass,
                &scene.my_cubes.as_drawable(&self.cube),
                &my_params,
                &Default::default(),
            )?
            .compose(&scene.lights)?
            .present()
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // Initialize glium
    let mut events_loop = glutin::EventsLoop::new();
    let display = {
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(WINDOW_SIZE.into())
            .with_title("Rendology example: Cube");
        let context_builder = glutin::ContextBuilder::new();
        glium::Display::new(window_builder, context_builder, &events_loop).unwrap()
    };

    // Initialize rendology pipeline
    let mut pipeline = Pipeline::create(&display, &Default::default()).unwrap();

    let start_time = Instant::now();
    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent {
                event: glutin::WindowEvent::CloseRequested,
                ..
            } = event
            {
                quit = true;
            }
        });

        let time = start_time.elapsed().as_fractional_secs() as f32;
        let scene = scene(time);

        let mut target = display.draw();
        let render_context = render_context(target.get_dimensions());

        pipeline
            .draw_frame(&display, &render_context, &scene, &mut target)
            .unwrap();

        target.finish().unwrap();
    }
}

fn scene(time: f32) -> Scene {
    let mut scene = Scene::default();

    scene.time = time;

    scene.cubes.add(basic_obj::Instance {
        transform: na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(10.0, 10.0, 0.1)),
        color: na::Vector4::new(0.9, 0.9, 0.9, 1.0),
    });

    scene.my_cubes.add(my_scene::Instance {
        transform: na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 3.0))
            * na::Matrix4::from_euler_angles(time, time, time),
    });

    scene.lights.push(Light {
        position: na::Point3::new(10.0, 10.0, 10.0),
        attenuation: na::Vector3::new(1.0, 0.0, 0.0),
        color: na::Vector3::new(1.0, 1.0, 1.0),
        is_main: true,
        ..Default::default()
    });

    scene
}

fn render_context(target_size: (u32, u32)) -> rendology::Context {
    let camera = rendology::Camera {
        view: na::Matrix4::look_at_rh(
            &na::Point3::new(9.0, -5.0, 7.0),
            &na::Point3::new(0.0, 0.0, 0.0),
            &na::Vector3::new(0.0, 0.0, 1.0),
        ),
        projection: na::Perspective3::new(
            target_size.0 as f32 / target_size.1 as f32,
            60.0f32.to_radians(),
            0.1,
            1000.0,
        )
        .to_homogeneous(),
        viewport: na::Vector4::new(0.0, 0.0, target_size.0 as f32, target_size.1 as f32),
    };

    rendology::Context {
        camera,
        main_light_pos: na::Point3::new(10.0, 10.0, 10.0),
        main_light_center: na::Point3::new(0.0, 0.0, 0.0),
    }
}

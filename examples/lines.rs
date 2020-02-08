use std::time::Instant;

use floating_duration::TimeAsFloat;
use glium::{glutin, Surface};
use nalgebra as na;

use rendology::{
    basic_obj, line, BasicObj, InstancingMode, Light, Mesh, PlainScenePass, RenderList,
    ShadedScenePass, ShadedScenePassSetup, ShadowPass,
};

const WINDOW_SIZE: (u32, u32) = (1280, 720);

#[derive(Default)]
struct Scene {
    cubes: RenderList<basic_obj::Instance>,
    lines: RenderList<line::Instance>,
    lights: Vec<Light>,
}

struct Pipeline {
    rendology: rendology::Pipeline,

    shadow_pass: Option<ShadowPass<basic_obj::Core>>,
    scene_pass: ShadedScenePass<basic_obj::Core>,
    line_pass: PlainScenePass<line::Core>,

    cube: Mesh<basic_obj::Vertex>,
    line: Mesh<line::Point>,
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
        let line_pass =
            rendology.create_plain_scene_pass(facade, line::Core, InstancingMode::Uniforms)?;

        let cube = BasicObj::Cube.create_mesh(facade)?;
        let line = line::create_mesh(facade)?;

        Ok(Pipeline {
            rendology,
            shadow_pass,
            scene_pass,
            line_pass,
            cube,
            line,
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

        let line_draw_params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };

        let line_params = line::Params { feather: 0.1 };

        self.rendology
            .start_frame(facade, (1.0, 1.0, 1.0), context.clone(), target)?
            .shadow_pass()
            .draw(
                &self.shadow_pass,
                &scene.cubes.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .shaded_scene_pass()
            .draw(
                &self.scene_pass,
                &scene.cubes.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .compose(&scene.lights)?
            .postprocess()?
            .plain_scene_pass()
            .draw(
                &self.line_pass,
                &scene.lines.as_drawable(&self.line),
                &line_params,
                &line_draw_params,
            )?
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

    scene.cubes.add(basic_obj::Instance {
        transform: na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 3.0))
            * na::Matrix4::from_euler_angles(time, time, time),
        color: na::Vector4::new(0.9, 0.9, 0.9, 1.0),
    });

    scene.cubes.add(basic_obj::Instance {
        transform: na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(10.0, 10.0, 0.1)),
        color: na::Vector4::new(0.0, 1.0, 0.0, 1.0),
    });

    scene.lights.push(Light {
        position: na::Point3::new(10.0, 10.0, 10.0),
        attenuation: na::Vector4::new(1.0, 0.0, 0.0, 0.0),
        color: na::Vector3::new(1.0, 1.0, 1.0),
        is_main: true,
        ..Default::default()
    });

    scene.lines.add(line::Instance::from_start_end(
        &na::Point3::new(0.0, 0.0, 1.0),
        &na::Point3::new(5.0, 5.0, 3.0),
        &na::Vector4::new(0.0, 0.0, 0.0, 1.0),
        100.0,
    ));

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
        viewport_size: na::Vector2::new(target_size.0 as f32, target_size.1 as f32),
    };

    rendology::Context {
        camera,
        main_light_pos: na::Point3::new(10.0, 10.0, 10.0),
        main_light_center: na::Point3::new(0.0, 0.0, 0.0),
        ambient_light: na::Vector3::new(0.3, 0.3, 0.3),
    }
}

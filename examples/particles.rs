//! Video: https://streamable.com/uglim

use std::time::Instant;

use coarse_prof::profile;
use floating_duration::TimeAsFloat;
use glium::{glutin, Surface};
use nalgebra as na;

use rendology::particle::Particle;
use rendology::{
    basic_obj, particle, BasicObj, Instancing, InstancingMode, Light, Mesh, PlainScenePass,
    RenderList, ShadedScenePass, ShadedScenePassSetup, ShadowPass,
};

const WINDOW_SIZE: (u32, u32) = (1280, 720);

#[derive(Default)]
struct Scene {
    time: f32,
    cubes: RenderList<basic_obj::Instance>,
    new_particles: RenderList<Particle>,
    lights: Vec<Light>,
}

struct Pipeline {
    rendology: rendology::Pipeline,

    shadow_pass: Option<ShadowPass<basic_obj::Core>>,
    scene_pass: ShadedScenePass<basic_obj::Core>,

    particle_system: particle::System,
    particle_pass: PlainScenePass<particle::Shader>,

    cube: Mesh<basic_obj::Vertex>,
    cube_instancing: Instancing<basic_obj::Instance>,
}

impl Pipeline {
    fn create<F: glium::backend::Facade>(
        facade: &F,
        config: &rendology::Config,
    ) -> Result<Self, rendology::pipeline::CreationError> {
        let rendology = rendology::Pipeline::create(facade, config, WINDOW_SIZE)?;

        let shadow_pass =
            rendology.create_shadow_pass(facade, basic_obj::Core, InstancingMode::Vertex)?;
        let scene_pass = rendology.create_shaded_scene_pass(
            facade,
            basic_obj::Core,
            InstancingMode::Vertex,
            ShadedScenePassSetup {
                draw_shadowed: true,
                draw_glowing: false,
            },
        )?;

        let particle_system = particle::System::create(facade, &Default::default())?;
        let particle_pass = rendology.create_plain_scene_pass(
            facade,
            particle_system.shader(),
            InstancingMode::Uniforms,
        )?;

        let cube = BasicObj::Cube.create_mesh(facade)?;
        let cube_instancing = Instancing::create(facade)?;

        Ok(Pipeline {
            rendology,
            shadow_pass,
            scene_pass,
            particle_system,
            particle_pass,
            cube,
            cube_instancing,
        })
    }

    fn draw_frame<F: glium::backend::Facade, S: glium::Surface>(
        &mut self,
        facade: &F,
        context: &rendology::Context,
        scene: &Scene,
        target: &mut S,
    ) -> Result<(), rendology::DrawError> {
        profile!("draw_frame");

        self.cube_instancing
            .update(facade, scene.cubes.as_slice())?;

        self.particle_system.spawn(scene.new_particles.as_slice());

        let draw_params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let particle_params = particle::Params { time: scene.time };
        let particle_draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: false,
                ..Default::default()
            },
            blend: glium::Blend {
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::One,
                },
                ..Default::default()
            },
            ..Default::default()
        };
        self.particle_system.set_current_time(scene.time);

        self.rendology
            .start_frame(facade, (0.0, 0.0, 0.0), context.clone(), target)?
            .shadow_pass()
            .draw(
                &self.shadow_pass,
                &self.cube_instancing.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .shaded_scene_pass()
            .draw(
                &self.scene_pass,
                &self.cube_instancing.as_drawable(&self.cube),
                &(),
                &draw_params,
            )?
            .compose(&scene.lights)?
            .plain_scene_pass()
            .draw(
                &self.particle_pass,
                &self.particle_system,
                &particle_params,
                &particle_draw_params,
            )?
            .postprocess()?
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
    let mut last_time = Instant::now();
    let mut quit = false;
    while !quit {
        profile!("frame");

        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent {
                event: glutin::WindowEvent::CloseRequested,
                ..
            } = event
            {
                quit = true;
            } else if let glutin::Event::WindowEvent {
                event: glutin::WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if input.state == glutin::ElementState::Pressed
                    && input.virtual_keycode == Some(glutin::VirtualKeyCode::P)
                {
                    coarse_prof::write(&mut std::io::stdout()).unwrap();
                    coarse_prof::reset();
                }
            }
        });

        let time = start_time.elapsed().as_fractional_secs() as f32;
        let dt = last_time.elapsed().as_fractional_secs() as f32;
        last_time = Instant::now();

        let scene = scene(time, dt);

        let mut target = display.draw();
        let render_context = render_context(target.get_dimensions());

        pipeline
            .draw_frame(&display, &render_context, &scene, &mut target)
            .unwrap();

        target.finish().unwrap();
    }
}

fn scene(time: f32, dt: f32) -> Scene {
    profile!("scene");

    let mut scene = Scene::default();

    scene.time = time;

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

    // Spawn particles and stuff
    let t = time * 1.5;
    let pos = na::Point3::new(3.0 * t.cos(), 3.0 * t.sin(), 3.0);
    let tangent = na::Vector3::new(t.cos(), t.sin(), 0.0);

    let smallest_unit = if tangent.x <= tangent.y && tangent.x <= tangent.z {
        na::Vector3::x()
    } else if tangent.y <= tangent.x && tangent.y <= tangent.z {
        na::Vector3::y()
    } else {
        na::Vector3::z()
    };
    let x_unit = tangent.cross(&smallest_unit).normalize();
    let y_unit = tangent.cross(&x_unit).normalize();

    let spawn_per_sec = 20000.0 * 30.0;

    for _ in 0..(spawn_per_sec * dt) as usize {
        let radius = rand::random::<f32>() * 1.41;
        let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;

        let velocity =
            angle.cos() * x_unit + angle.sin() * y_unit + 1.41 * radius * tangent.normalize();

        let particle = Particle {
            spawn_time: time,
            life_duration: 3.0,
            start_pos: pos + velocity * 1.0,
            velocity,
            color: na::Vector3::new(radius / 2.0, radius / 8.0, 0.0),
            size: 0.015,
            friction: 0.5,
        };

        scene.new_particles.add(particle);
    }

    scene
}

fn render_context(target_size: (u32, u32)) -> rendology::Context {
    let projection = na::Perspective3::new(
        target_size.0 as f32 / target_size.1 as f32,
        60.0f32.to_radians(),
        0.1,
        1000.0,
    )
    .to_homogeneous();
    let camera = rendology::Camera {
        view: na::Matrix4::look_at_rh(
            &na::Point3::new(9.0, -5.0, 7.0),
            &na::Point3::new(0.0, 0.0, 0.0),
            &na::Vector3::new(0.0, 0.0, 1.0),
        ),
        projection,
        viewport_size: na::Vector2::new(target_size.0 as f32, target_size.1 as f32),
    };

    rendology::Context {
        camera,
        main_light_pos: na::Point3::new(10.0, 10.0, 10.0),
        main_light_center: na::Point3::new(0.0, 0.0, 0.0),
        ambient_light: na::Vector3::new(0.3, 0.3, 0.3),
    }
}

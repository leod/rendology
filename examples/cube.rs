use std::time::Instant;

use floating_duration::TimeAsFloat;
use nalgebra as na;
use glium::glutin;

const WINDOW_SIZE: (u32, u32) = (1280, 720);

fn main() {
    // Initialize glium
    let mut events_loop = glutin::EventsLoop::new();
    let display = {
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(WINDOW_SIZE.into())
            .with_title("Rendology example: Cube");
        let context_builder = glutin::ContextBuilder::new();
        glium::Display::new(window_builder, context_builder, &events_loop).unwrap()
    };

    // Initialize rendology
    let resources = rendology::Resources::create(&display).unwrap();
    let mut pipeline = rendology::Pipeline::create(&display, &Default::default(), WINDOW_SIZE).unwrap();

    let start_time = Instant::now();
    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event: glutin::WindowEvent::CloseRequested, .. } = event {
                quit = true;
            }
        });

        let time_elapsed = start_time.elapsed().as_fractional_secs() as f32;

        let mut render_lists = rendology::RenderLists::default();

        let angle = time_elapsed;
        render_lists.solid.add(rendology::Object::Cube, &rendology::scene::model::Params {
            transform: na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 3.0))
                * na::Matrix4::from_euler_angles(angle, angle, angle),
            color: na::Vector4::new(0.9, 0.9, 0.9, 1.0),
        });
        render_lists.solid.add(rendology::Object::Cube, &rendology::scene::model::Params {
            transform: na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(10.0, 10.0, 0.1)) ,
            color: na::Vector4::new(0.0, 1.0, 0.0, 1.0),
        });
        render_lists.lights.push(rendology::Light {
            position: na::Point3::new(10.0, 10.0, 10.0),
            attenuation: na::Vector3::new(1.0, 0.0, 0.0),
            color: na::Vector3::new(1.0, 1.0, 1.0),
            is_main: true,    
            ..Default::default()
        });

        let camera = rendology::Camera {
            view: na::Matrix4::look_at_rh(
                &na::Point3::new(9.0, -5.0, 7.0),
                &na::Point3::new(0.0, 0.0, 0.0), 
                &na::Vector3::new(0.0, 0.0, 1.0),
            ),
            projection: na::Perspective3::new(
                WINDOW_SIZE.0 as f32 / WINDOW_SIZE.1 as f32,
                60.0f32.to_radians(),
                0.1,
                1000.0,
            ).to_homogeneous(),
            viewport: na::Vector4::new(0.0, 0.0, WINDOW_SIZE.0 as f32, WINDOW_SIZE.1 as f32),
        };
        let context = rendology::Context {
            camera,
            main_light_pos: na::Point3::new(10.0, 10.0, 10.0),
            main_light_center: na::Point3::new(0.0, 0.0, 0.0),
        };

        let mut target = display.draw();
        pipeline.draw_frame(
            &display,
            &resources,
            &context,
            &render_lists,
            &mut target,
        ).unwrap();

        target.finish().unwrap();
    }
}

use nalgebra as na;

use crate::Camera;

#[derive(Debug, Clone)]
pub struct Context {
    pub camera: Camera,
    pub main_light_pos: na::Point3<f32>,
    pub main_light_center: na::Point3<f32>,
    pub ambient_light: na::Vector3<f32>,
}

impl_uniform_input!(
    Context,
    self => {
        context_camera_viewport_size: [f32; 2] = self.camera.viewport_size,
        context_camera_projection: [[f32; 4]; 4] = self.camera.projection,
        context_camera_view: [[f32; 4]; 4] = self.camera.view,
        context_main_light_pos: [f32; 3] = self.main_light_pos.coords,
        context_ambient_light: [f32; 3] = self.ambient_light,
    },
);

#[derive(Debug, Clone)]
pub struct Light {
    pub position: na::Point3<f32>,
    pub attenuation: na::Vector4<f32>,
    pub color: na::Vector3<f32>,
    pub is_main: bool,
    pub radius: f32,
}

impl_instance_input!(
    Light,
    self => {
        light_position: [f32; 3] = self.position.coords,
        light_attenuation: [f32; 4] = self.attenuation,
        light_color: [f32; 3] = self.color,
        light_radius: f32 = self.radius,
    },
);

impl Default for Light {
    fn default() -> Self {
        Self {
            position: na::Point3::origin(),
            attenuation: na::Vector4::zeros(),
            color: na::Vector3::zeros(),
            is_main: false,
            radius: 0.0,
        }
    }
}

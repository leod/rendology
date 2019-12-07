use nalgebra as na;

use crate::Camera;

#[derive(Debug, Clone)]
pub struct Context {
    pub camera: Camera,
    pub main_light_pos: na::Point3<f32>,
    pub main_light_center: na::Point3<f32>,
}

impl_uniform_input!(
    Context,
    context,
    self => {
        viewport: [f32; 4] => self.camera.viewport.into(),
        mat_projection: [[f32; 4]; 4] => self.camera.projection.into(),
        mat_view: [[f32; 4]; 4] => self.camera.view.into(),
        main_light_pos: [f32; 3] => self.main_light_pos.coords.into(),
    },
);

#[derive(Debug, Clone)]
pub struct Light {
    pub position: na::Point3<f32>,
    pub attenuation: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
    pub is_main: bool,
    pub radius: f32,
}

impl_instance_input!(
    Light,
    light,
    self => {
        light_position: [f32; 3] => self.position.coords.into(),
        light_attenuation: [f32; 3] => self.attenuation.into(),
        light_color: [f32; 3] => self.color.into(),
        //light_is_main: Bool => self.is_main,
        light_radius: f32 => self.radius,
    },
);

impl Default for Light {
    fn default() -> Self {
        Self {
            position: na::Point3::origin(),
            attenuation: na::Vector3::zeros(),
            color: na::Vector3::zeros(),
            is_main: false,
            radius: 0.0,
        }
    }
}

use nalgebra as na;

#[derive(Debug, Clone)]
pub struct Camera {
    pub viewport_size: na::Vector2<f32>,
    pub projection: na::Matrix4<f32>,
    pub view: na::Matrix4<f32>,
}

impl_uniform_input!(
    Camera,
    self => {
        camera_viewport_size: [f32; 2] => self.viewport_size,
        camera_projection: [[f32; 4]; 4] => self.projection,
        camera_view: [[f32; 4]; 4] => self.view,
    },
);

impl Camera {
    pub fn new(viewport_size: na::Vector2<f32>, projection: na::Matrix4<f32>) -> Camera {
        Camera {
            viewport_size,
            projection,
            view: na::Matrix4::identity(),
        }
    }

    pub fn project_to_viewport(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
        let q = self.projection * self.view * na::Vector4::new(p.x, p.y, p.z, 1.0);
        let h = q.fixed_rows::<na::U3>(0) / q.w;

        na::Point3::new(
            (h.x + 1.0) / 2.0 * self.viewport_size.x,
            (1.0 - (h.y + 1.0) / 2.0) * self.viewport_size.y,
            h.z,
        )
    }

    pub fn unproject_from_viewport(&self, win: &na::Point3<f32>) -> na::Point3<f32> {
        // As in:
        // https://www.nalgebra.org/rustdoc_glm/src/nalgebra_glm/ext/matrix_projection.rs.html#163

        let transform = (self.projection * self.view)
            .try_inverse()
            .unwrap_or_else(na::Matrix4::zeros);

        let point = na::Vector4::new(
            2.0 * win.x / self.viewport_size.x - 1.0,
            2.0 * (self.viewport_size.y - win.y) / self.viewport_size.y - 1.0,
            2.0 * win.z - 1.0,
            1.0,
        );

        let result = transform * point;
        na::Point3::from(result.fixed_rows::<na::U3>(0) / result.w)
    }
}

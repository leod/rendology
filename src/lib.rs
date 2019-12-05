#[macro_use]
pub mod shader;
pub mod camera;
pub mod draw_instances;
pub mod error;
pub mod fxaa;
pub mod instancing;
pub mod object;
pub mod pipeline;
pub mod resources;
pub mod scene;
pub mod screen_quad;
pub mod stage;

pub use camera::Camera;
pub use error::{CreationError, DrawError};
pub use instancing::Instancing;
pub use object::{Object, ObjectBuffers};
pub use pipeline::{Pipeline, PlainScenePass, ShadedScenePass, ShadedScenePassSetup, ShadowPass};
pub use resources::Resources;
pub use screen_quad::ScreenQuad;
pub use stage::{Context, Light, RenderList};

#[macro_use]
pub mod shader;

mod camera;
mod drawable;
mod error;
mod instancing;
mod line;
mod mesh;
mod render_list;
mod scene;
mod stage;

pub mod basic_obj;
pub mod fxaa;
pub mod pipeline;
pub mod screen_quad;

pub use basic_obj::BasicObj;
pub use camera::Camera;
pub use drawable::Drawable;
pub use error::{CreationError, DrawError};
pub use instancing::Instancing;
pub use mesh::Mesh;
pub use pipeline::{
    Config, Pipeline, PlainScenePass, ShadedScenePass, ShadedScenePassSetup, ShadowPass,
};
pub use render_list::RenderList;
pub use scene::SceneCore;
pub use screen_quad::ScreenQuad;
pub use shader::InstancingMode;
pub use stage::{Context, Light};

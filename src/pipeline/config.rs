use crate::fxaa;
use crate::pipeline::{deferred, glow, shadow};

#[derive(Debug, Clone)]
pub struct Config {
    pub shadow_mapping: Option<shadow::Config>,
    pub deferred_shading: Option<deferred::Config>,
    pub glow: Option<glow::Config>,
    pub hdr: Option<f32>,
    pub gamma_correction: Option<f32>,
    pub fxaa: Option<fxaa::Config>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shadow_mapping: Some(Default::default()),
            deferred_shading: Some(Default::default()),
            glow: Some(Default::default()),
            hdr: None,
            gamma_correction: Some(2.2),
            fxaa: Some(Default::default()),
        }
    }
}

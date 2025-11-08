#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod calculator;
use crate::exports::codeberg::circle_of_confusion::calculator::{Guest, GuestCalculator};

#[cfg(feature = "wasi")]
wit_bindgen::generate!({
    world: "circle-of-confusion",
});

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(target_arch = "wasm32")]
struct Component;

impl GuestCalculator for Calculator {
    fn new(settings: Settings) -> Self {
        Calculator::new(settings)
    }
    fn calculate(&self, value: f32) -> f32 {
        self.calculate(value)
    }
}

#[cfg(target_arch = "wasm32")]
impl Guest for Component {
    type Calculator = Calculator;
}

impl Filmback {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Resolution {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Default for exports::codeberg::circle_of_confusion::settings::Settings {
    fn default() -> Self {
        Self {
            size: 5.0,
            max_size: 10.0,
            math: exports::codeberg::circle_of_confusion::settings::Math::OneDividedByZ,
            focal_plane: 0.0,
            protect: 0.0,
            pixel_aspect: 1.0,
            camera_data: None,
        }
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            focal_length: 50.0,
            f_stop: 16.0,
            filmback: Filmback::new(24.576, 18.672),
            near_field: 0.1,
            far_field: 10000.0,
            world_unit: WorldUnit::M,
            resolution: Resolution::new(1920, 1080),
        }
    }
}

pub use exports::codeberg::circle_of_confusion::settings::*;
pub use calculator::Calculator;


#[cfg(target_arch = "wasm32")]
export!(Component);


mod something {
    use crate::Calculator;
    
}


#[cfg(feature = "python-bindings")]
#[pymodule]
mod circle_of_confusion {
    #[pymodule_export]
    use crate::{Calculator, CameraData, Math, Settings, WorldUnit};
}

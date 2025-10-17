#![cfg_attr(not(feature = "python-bindings"), no_std)]

mod calculator;
mod settings;

pub use crate::calculator::Calculator;
pub use crate::settings::{CameraData, Math, Settings, WorldUnit};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
mod circle_of_confusion {
    #[pymodule_export]
    use crate::{Calculator, CameraData, Math, Settings, WorldUnit};
}

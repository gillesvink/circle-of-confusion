// Data used to set the circle of confusion parameters.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(i32)]
#[cfg_attr(feature = "python-bindings", pyclass)]
/// World unit that specifies the unit for the depth channel
///
/// This enum defines the units that can be used for depth channel measurements.
/// The unit determines how depth values are interpreted in the rendering process.
pub enum WorldUnit {
    /// Millimeters
    MM,
    /// Centimeters
    CM,
    /// Decimeters
    DM,
    /// Meters
    M,
    /// Inches
    INCH,
    /// Feet
    FT,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(i32)]
#[cfg_attr(feature = "python-bindings", pyclass)]
/// Math mode of provided depth channel
///
/// This enum defines how the depth channel's mathematical representation
/// should be interpreted. The mode is specified by the render engine that
/// generates the depth information.
pub enum Math {
    /// Real depth values
    REAL,
    /// One divided by Z (inverse depth)
    OneDividedByZ,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "python-bindings", pyclass)]
/// Camera parameters that are used when using the camera-based circle of confusion
///
/// This struct contains camera-specific parameters that are used to calculate
/// circle of confusion.
pub struct CameraData {
    /// Focal length of the camera lens
    pub focal_length: f32,
    /// F-stop (aperture) of the camera
    pub f_stop: f32,
    /// Filmback size (width and height)
    pub filmback: [f32; 2],
    /// Near field distance for depth of field
    pub near_field: f32,
    /// Far field distance for depth of field
    pub far_field: f32,
    /// World unit for depth measurements
    pub world_unit: WorldUnit,
    /// Resolution camera is recording at
    pub resolution: [u32; 2],
}

#[cfg_attr(feature = "python-bindings", pymethods)]
impl CameraData {
    #[cfg(feature = "python-bindings")]
    #[new]
    pub fn new_py(
        focal_length: f32,
        f_stop: f32,
        filmback: [f32; 2],
        near_field: f32,
        far_field: f32,
        world_unit: WorldUnit,
        resolution: [u32; 2],
    ) -> Self {
        Self {
            focal_length,
            f_stop,
            filmback: filmback.into(),
            near_field,
            far_field,
            world_unit,
            resolution: resolution.into(),
        }
    }
}

impl CameraData {
    pub fn new(
        focal_length: f32,
        f_stop: f32,
        filmback: [f32; 2],
        near_field: f32,
        far_field: f32,
        world_unit: WorldUnit,
        resolution: [u32; 2],
    ) -> Self {
        Self {
            focal_length,
            f_stop,
            filmback: filmback.into(),
            near_field,
            far_field,
            world_unit,
            resolution: resolution.into(),
        }
    }
}

impl Default for CameraData {
    fn default() -> Self {
        CameraData {
            focal_length: 50.0,
            f_stop: 16.0,
            filmback: [24.576, 18.672],
            near_field: 0.1,
            far_field: 10000.0,
            world_unit: WorldUnit::M,
            resolution: [1920, 1080],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "python-bindings", pyclass)]
/// Settings for the circle of confusion calculation.
///
/// This struct contains all the parameters needed to calculate the circle of confusion.
pub struct Settings {
    /// Base size for circle of confusion
    pub size: f32,
    /// Maximum size for circle of confusion (clamped)
    pub max_size: f32,
    /// Mathematical interpretation of depth
    pub math: Math,
    /// Focal plane position
    pub focal_plane: f32,
    /// Protection value for circle of confusion, this adds a region that has a CoC of zero.
    pub protect: f32,
    /// Pixel aspect ratio
    pub pixel_aspect: f32,
    /// Camera parameters
    pub camera_data: Option<CameraData>,
}

#[cfg_attr(feature = "python-bindings", pymethods)]
impl Settings {
    #[cfg(feature = "python-bindings")]
    #[new]
    pub fn py_new(
        size: f32,
        max_size: f32,
        math: Math,
        focal_plane: f32,
        protect: f32,
        pixel_aspect: f32,
        camera_data: Option<CameraData>,
    ) -> Self {
        Self::new(
            size,
            max_size,
            math,
            focal_plane,
            protect,
            pixel_aspect,
            camera_data,
        )
    }
}

impl Settings {
    pub fn new(
        size: f32,
        max_size: f32,
        math: Math,
        focal_plane: f32,
        protect: f32,
        pixel_aspect: f32,
        camera_data: Option<CameraData>,
    ) -> Self {
        Self {
            size,
            max_size,
            math,
            camera_data,
            focal_plane,
            protect,
            pixel_aspect,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            math: Math::OneDividedByZ,
            size: 5.0,
            max_size: 10.0,
            camera_data: None,
            focal_plane: 0.0,
            protect: 0.0,
            pixel_aspect: 1.0,
        }
    }
}

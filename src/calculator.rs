use crate::{Math, Settings, WorldUnit};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python-bindings", pyclass)]
/// Calculator that is able to calculate the Circle of Confusion based on the provided settings.
///
/// The size in px is the radius of the convolution.
/// A CoC of 10 would mean a diameter of 20 pixels.
///
/// ## Output:
///
/// * `+` is for far field pixels
/// * `-` is for near field pixels
///
/// ## Modes
///
/// The calculator supports two modes, one is physically accurate,
/// the other lets you tune your own DoF size.
///
/// ### Manually
/// When no camera data is provided to the Settings struct, the manual mode will be used.
/// This creates a smooth falloff to the focal plane point, based on the size and max size added.
///
/// It will gradually apply the size based on the distance from the focal plane.
/// When using a larger size, the CoC will be increased.
///
/// Protect can be used to apply a safe region to the focal plane.
///
/// ### Camera
/// When the camera data is applied to the settings,
/// the camera values will be used instead. This matches real world CoC values.
///
/// Lowering f-stop will increase the CoC values, just like increasing the focal-length would.
///
/// This calculation is based on the CoC algorithm:
/// [Wikipedia](https://en.wikipedia.org/wiki/Circle_of_confusion)
///
/// ## Usage
/// Use the `Calculator::new(settings)` method to create the instance
/// and let the necessary parameters be pre-computed.
///
/// To update the settings, for example when changing the focal plane,
/// call the `update_settings` method.
///
/// ## Example
/// ```
/// use circle_of_confusion::{Settings, Calculator, CameraData, Math};
/// let settings = Settings {
///     math: Math::REAL,
///     focal_plane: 30.0,
///     max_size: 100.0,
///     camera_data: Some(CameraData {
///         f_stop: 2.0,
///         focal_length: 100.0,
///         ..Default::default()
///     }),
///     ..Default::default()
/// };
/// let calculator = Calculator::new(settings);
/// let result = calculator.calculate(10.0);
/// assert_eq!(result, -11.935329)
/// ```
pub struct Calculator {
    settings: Settings,
    internal_focus: f32,
    world_unit_multiplier: f32,
    depth_of_field: [f32; 2],
    hyperfocal_distance: f32,
}

#[cfg_attr(feature = "python-bindings", pymethods)]
impl Calculator {
    #[cfg(feature = "python-bindings")]
    #[new]
    pub fn new_py(settings: Settings) -> Self {
        Self::new(settings)
    }

    /// Update the instance of the Calculator with the specified settings.
    ///
    /// Automatically calculates all necessary values for calculations.
    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.world_unit_multiplier = Self::get_world_unit_multiplier(&self.settings);
        self.internal_focus = Self::get_internal_focus(&self.settings, self.world_unit_multiplier);
        self.depth_of_field = Self::get_depth_of_field(
            &self.settings,
            self.internal_focus,
            self.world_unit_multiplier,
        );
        let zeiss_formula = Self::calculate_zeiss_formula(&self.settings);
        self.hyperfocal_distance =
            Self::calculate_hyperfocal_distance(&self.settings, zeiss_formula);
    }

    /// Perform calculation based on the input value.
    ///
    /// This can be called upon a depth map for each pixel for example,
    /// to calculate the size of convolution for each pixel.
    pub fn calculate(&self, value: f32) -> f32 {
        let mut converted_value = Self::convert_value_to_distance(value, &self.settings.math);

        if self.settings.camera_data.is_some() {
            converted_value *= self.world_unit_multiplier;
            converted_value = self.calculate_circle_of_confusion(converted_value);
            converted_value *= self.settings.pixel_aspect;
        } else {
            converted_value = self.calculate_direct_map(converted_value);
        }
        -converted_value.clamp(-self.settings.max_size, self.settings.max_size)
    }
}

impl Calculator {
    /// Create a new instance of the Calculator with the specified settings.
    ///
    /// Automatically calculates all necessary values for calculations.
    pub fn new(settings: Settings) -> Self {
        let world_unit_multiplier = Self::get_world_unit_multiplier(&settings);
        let internal_focus = Self::get_internal_focus(&settings, world_unit_multiplier);
        let depth_of_field =
            Self::get_depth_of_field(&settings, internal_focus, world_unit_multiplier);

        let zeiss_formula = Self::calculate_zeiss_formula(&settings);
        let hyperfocal_distance = Self::calculate_hyperfocal_distance(&settings, zeiss_formula);

        Self {
            settings,
            internal_focus,
            world_unit_multiplier,
            depth_of_field,
            hyperfocal_distance,
        }
    }
    /// Just a wrapper for pythagoras (length) calculation.
    fn length(a: f32, b: f32) -> f32 {
        libm::sqrtf(libm::powf(a, 2.0) + libm::powf(b, 2.0))
    }

    /// Calculate the Zeiss formula to get the criterion.
    ///
    /// More information can be found here:
    /// https://resources.wolframcloud.com/FormulaRepository/resources/Zeiss-Formula
    fn calculate_zeiss_formula(settings: &Settings) -> f32 {
        let camera_data = match settings.camera_data {
            Some(data) => data,
            None => return 1.0,
        };
        Self::length(camera_data.filmback[0], camera_data.filmback[1]) / 1730.0
    }

    /// Calculate the distance where it does not matter
    /// anymore if the focal plane changes
    ///
    /// The CoC stays the same after this value.
    ///
    /// More information:
    /// https://www.watchprosite.com/editors-picks/using-the-zeiss-formula-to-understand-the-circle-of-confusion/1278.1127636.8608906/
    fn calculate_hyperfocal_distance(settings: &Settings, zeiss_formula: f32) -> f32 {
        let camera_data = match settings.camera_data {
            Some(data) => data,
            None => return 0.0, // for non-camera-data things we just use zero
        };
        (libm::powf(camera_data.focal_length, 2.0) / (camera_data.f_stop * zeiss_formula))
            + camera_data.focal_length
    }

    /// Map the world unit from the settings to a multiplication value
    fn get_world_unit_multiplier(settings: &Settings) -> f32 {
        let world_unit = match settings.camera_data {
            Some(data) => data.world_unit,
            None => return 1.0,
        };
        match world_unit {
            WorldUnit::MM => 1.0,
            WorldUnit::CM => 10.0,
            WorldUnit::DM => 100.0,
            WorldUnit::M => 1000.0,
            WorldUnit::INCH => 25.4,
            WorldUnit::FT => 304.8,
        }
    }

    /// Convert the distance selected according to the math of the input value and the world unit.
    fn get_internal_focus(settings: &Settings, world_unit_multiplier: f32) -> f32 {
        Self::convert_value_to_distance(settings.focal_plane, &settings.math).max(0.0)
            * world_unit_multiplier
    }

    /// Calculate the depth of field range for a safe region.
    /// This is used to increase the region which is considered to be in focus.
    fn get_depth_of_field(
        settings: &Settings,
        internal_focus: f32,
        world_unit_multiplier: f32,
    ) -> [f32; 2] {
        if settings.protect == 0.0 || internal_focus == 0.0 {
            return [internal_focus, internal_focus];
        }
        if settings.camera_data.is_some() {
            return [
                internal_focus - ((settings.protect * 0.5) * world_unit_multiplier),
                internal_focus + ((settings.protect * 0.5) * world_unit_multiplier),
            ];
        }
        let normalized_focus = 1.0 / internal_focus;
        [
            1.0 / (normalized_focus + (normalized_focus * (settings.protect * 0.5))),
            1.0 / (normalized_focus - (normalized_focus * (settings.protect * 0.5))),
        ]
    }

    /// Convert input value to distance value.
    fn convert_value_to_distance(value: f32, math: &Math) -> f32 {
        if value == 0.0 {
            return 9999.0;
        }
        match math {
            Math::REAL => value,
            Math::OneDividedByZ => 1.0 / value,
        }
    }

    /// Apply the Circle of Confusion algorithm to the distance, to calculate the disc
    /// size of confusion which a real camera would also have.
    fn calculate_circle_of_confusion(&self, distance: f32) -> f32 {
        let camera_data = match self.settings.camera_data {
            Some(data) => data,
            None => {
                return 0.0;
            }
        };
        if distance == 0.0 {
            return distance;
        }
        let mut calculated_focal_distance = self.internal_focus;
        let simple_point = [0.0; 2];
        if self.depth_of_field != simple_point
            && (distance > self.depth_of_field[0] && distance < self.depth_of_field[1])
        {
            return 0.0;
        }
        if distance < self.depth_of_field[0] {
            calculated_focal_distance = self.depth_of_field[0];
        } else if distance > self.internal_focus {
            calculated_focal_distance = self.depth_of_field[1];
        }

        calculated_focal_distance = calculated_focal_distance.min(self.hyperfocal_distance);
        let circle_of_confusion = ((calculated_focal_distance - distance)
            * libm::powf(camera_data.focal_length, 2.0))
            / (camera_data.f_stop
                * distance
                * (calculated_focal_distance - camera_data.focal_length));

        circle_of_confusion / (Self::length(camera_data.filmback[0], camera_data.filmback[1]))
            * (Self::length(
                camera_data.resolution[0] as f32,
                camera_data.resolution[1] as f32,
            ) * 0.5)
    }

    /// Calculate the Circle of Confusion based on the manual values selected.
    /// This is not physically accurate, but gives a nice falloff.
    fn calculate_direct_map(&self, pixel_value: f32) -> f32 {
        if self.internal_focus == pixel_value
            || (pixel_value > self.depth_of_field[0] && pixel_value < self.depth_of_field[1])
        {
            return 0.0;
        }

        let converted_pixel_value = if pixel_value == 0.0 {
            0.0
        } else {
            1.0 / pixel_value
        };

        let mut calculated_value = 0.0;
        if self.internal_focus < pixel_value {
            let calculated_focus_point = if self.depth_of_field[1] == 0.0 {
                0.0
            } else {
                1.0 / self.depth_of_field[1]
            };
            calculated_value =
                -(calculated_focus_point - converted_pixel_value) / calculated_focus_point;
        }
        if self.internal_focus > pixel_value {
            let calculated_focus_point = if self.depth_of_field[0] == 0.0 {
                0.0
            } else {
                1.0 / self.depth_of_field[0]
            };
            let calculated_near_field =
                (converted_pixel_value - calculated_focus_point) / calculated_focus_point;
            calculated_value =
                calculated_near_field.min(self.settings.max_size / self.settings.size)
        }
        calculated_value * self.settings.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::CameraData;
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    #[rstest]
    #[case( // 1 Test simple size
        0.0,
        Settings { size: 20.0, ..Default::default() },
        0.0
    )]
    #[case( // 2 Test another size
        10.0,
        Settings { size: 30.0, ..Default::default() },
        -10.0
    )]
    #[case( // 3 Test in-focus
        50.0,
        Settings { focal_plane: 50.0, max_size: 100.0, ..Default::default() },
        0.0
    )]
    #[case( // 4 Test out of focus
        0.5,
        Settings {
            math: Math::OneDividedByZ,
            focal_plane: 0.0,
            max_size: 100.0,
            ..Default::default()
        },
        -100.0
    )]
    #[case( // 5 Test focus in foreground
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 0.0,
            max_size: 100.0,
            ..Default::default()
        },
        -100.0
    )]
    #[case( // 6 Test z-focus
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 3.0,
            size: 1.0,
            max_size: 100.0,
            ..Default::default()
        },
        0.7
    )]
    #[case( // 7 Test far focus with pixels that are zero.
        0.0,
        Settings {
            math: Math::OneDividedByZ,
            focal_plane: 20.0,
            size: 3.0,
            max_size: 100.0,
            ..Default::default()
        },
        2.9999843
    )]
    #[case( // 8 Test camera focus
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 10.0,
            max_size: 100.0,
            camera_data: Some(CameraData::default()),
            ..Default::default()
        },
        0.075
    )]
    #[case( // 9 Test camera out of focus
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 15.0,
            max_size: 10.0,
            camera_data: Some(
                CameraData {
                    f_stop: 2.0,
                    ..Default::default()
                }
            ),
            ..Default::default()
        },
        -1.4819534
    )]
    #[case( // 10 Test camera change focal plane
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 30.0,
            max_size: 100.0,
            camera_data: Some(
                CameraData {
                    f_stop: 2.0,
                    focal_length: 100.0,
                    ..Default::default()
                }
            ),
            ..Default::default()
        },
        -11.935
    )]
    #[case( // 11 Test camera far zero should return in far max
        10.0,
        Settings {
            math: Math::REAL,
            focal_plane: 2.4,
            max_size: 50.0,
            camera_data: Some(
                CameraData {
                    f_stop: 2.0,
                    focal_length: 100.0,
                    ..Default::default()
                }
            ),
            ..Default::default()
        },
        50.0
    )]
    fn test_map_rendering(#[case] coc: f32, #[case] settings: Settings, #[case] expected: f32) {
        let calculator = Calculator::new(settings);

        assert_approx_eq!(calculator.calculate(coc), expected, 1e-2f32);
    }
}

use crate::{
    Calculator, Settings, DepthOfField, Math, WorldUnit,
};

impl Calculator {
    /// Perform calculation based on the input value.
    ///
    /// This can be called upon a depth map for each pixel for example,
    /// to calculate the size of convolution for each pixel.
    pub fn calculate(&self, value: f32) -> f32 {
        let mut converted_value = Self::convert_value_to_distance(value, &self.settings.math());

        if self.settings.camera_data.is_some() {
            converted_value *= self.world_unit_multiplier;
            converted_value = self.calculate_circle_of_confusion(converted_value);
            converted_value *= self.settings.pixel_aspect;
        } else {
            converted_value = self.calculate_direct_map(converted_value);
        }
        converted_value.clamp(-self.settings.max_size, self.settings.max_size)
    }
}

impl Calculator {
    /// Create a new instance of the Calculator with the specified settings.
    ///
    /// Automatically calculates all necessary values for calculations.
    pub fn new(settings: Settings) -> Self {
        let mut instance = Self::default();
        instance.settings = settings;
        instance.compute();
        instance
    }
    /// Precompute the internal data
    fn compute(&mut self) {
        self.world_unit_multiplier = self.get_world_unit_multiplier();
        self.internal_focus = self.get_internal_focus();
        self.depth_of_field = self.get_depth_of_field();

        self.hyperfocal_distance = Self::calculate_hyperfocal_distance(&self.settings);
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
        let camera_data = match &settings.camera_data {
            Some(data) => data,
            None => return 1.0,
        };
        Self::length(camera_data.filmback.width, camera_data.filmback.height) / 1730.0
    }

    /// Calculate the distance where it does not matter
    /// anymore if the focal plane changes
    ///
    /// The CoC stays the same after this value.
    ///
    /// More information:
    /// https://www.watchprosite.com/editors-picks/using-the-zeiss-formula-to-understand-the-circle-of-confusion/1278.1127636.8608906/
    fn calculate_hyperfocal_distance(settings: &Settings) -> f32 {
        let camera_data = match &settings.camera_data {
            Some(data) => data,
            None => return 0.0, // for non-camera-data things we just use zero
        };
        let zeiss_formula = Self::calculate_zeiss_formula(settings);
        (libm::powf(camera_data.focal_length, 2.0) / (camera_data.f_stop * zeiss_formula))
            + camera_data.focal_length
    }

    /// Map the world unit from the settings to a multiplication value
    fn get_world_unit_multiplier(&self) -> f32 {
        let world_unit = match &self.settings.camera_data {
            Some(data) => data.world_unit(),
            None => return 1.0,
        };
        match world_unit {
            WorldUnit::Mm => 1.0,
            WorldUnit::Cm => 10.0,
            WorldUnit::Dm => 100.0,
            WorldUnit::M => 1000.0,
            WorldUnit::In => 25.4,
            WorldUnit::Ft => 304.8,
        }
    }

    /// Convert the distance selected according to the math of the input value and the world unit.
    fn get_internal_focus(&self) -> f32 {
        Self::convert_value_to_distance(self.settings.focal_plane, &self.settings.math()).max(0.0)
            * self.world_unit_multiplier
    }

    /// Calculate the depth of field range for a safe region.
    /// This is used to increase the region which is considered to be in focus.
    fn get_depth_of_field(&self) -> DepthOfField {
        if self.settings.protect == 0.0 || self.internal_focus == 0.0 {
            return DepthOfField {
                x: self.internal_focus,
                y: self.internal_focus,
            };
        }
        if self.settings.camera_data.is_some() {
            return DepthOfField {
                x: self.internal_focus
                    - ((self.settings.protect * 0.5) * self.world_unit_multiplier),
                y: self.internal_focus
                    + ((self.settings.protect * 0.5) * self.world_unit_multiplier),
            };
        }
        let normalized_focus = 1.0 / self.internal_focus;
        DepthOfField {
            x: 1.0 / (normalized_focus + (normalized_focus * (self.settings.protect * 0.5))),
            y: 1.0 / (normalized_focus - (normalized_focus * (self.settings.protect * 0.5))),
        }
    }

    /// Convert input value to distance value.
    fn convert_value_to_distance(value: f32, math: &Math) -> f32 {
        if value == 0.0 {
            return 9999.0;
        }
        match math {
            Math::Real => value,
            Math::OneDividedByZ => 1.0 / value,
        }
    }

    /// Apply the Circle of Confusion algorithm to the distance, to calculate the disc
    /// size of confusion which a real camera would also have.
    fn calculate_circle_of_confusion(&self, distance: f32) -> f32 {
        let camera_data = match &self.settings.camera_data {
            Some(data) => data,
            None => {
                return 0.0;
            }
        };
        if distance == 0.0 {
            return distance;
        }
        let mut calculated_focal_distance = self.internal_focus;
        let simple_point = DepthOfField { x: 0.0, y: 0.0 };
        if self.depth_of_field != simple_point
            && (distance > self.depth_of_field.x && distance < self.depth_of_field.y)
        {
            return 0.0;
        }
        if distance < self.depth_of_field.x {
            calculated_focal_distance = self.depth_of_field.x;
        } else if distance > self.internal_focus {
            calculated_focal_distance = self.depth_of_field.y;
        }

        calculated_focal_distance = calculated_focal_distance.min(self.hyperfocal_distance);
        let circle_of_confusion = ((calculated_focal_distance - distance)
            * libm::powf(camera_data.focal_length, 2.0))
            / (camera_data.f_stop
                * distance
                * (calculated_focal_distance - camera_data.focal_length));

        -(circle_of_confusion
            / (Self::length(camera_data.filmback.width, camera_data.filmback.height))
            * (Self::length(
                camera_data.resolution.width as f32,
                camera_data.resolution.height as f32,
            ) * 0.5))
    }

    /// Calculate the Circle of Confusion based on the manual values selected.
    /// This is not physically accurate, but gives a nice falloff.
    fn calculate_direct_map(&self, pixel_value: f32) -> f32 {
        if self.internal_focus == pixel_value
            || (pixel_value > self.depth_of_field.x && pixel_value < self.depth_of_field.y)
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
            let calculated_focus_point = if self.depth_of_field.y == 0.0 {
                0.0
            } else {
                1.0 / self.depth_of_field.y
            };
            calculated_value =
                (calculated_focus_point - converted_pixel_value) / calculated_focus_point;
        }
        if self.internal_focus > pixel_value {
            let calculated_focus_point = if self.depth_of_field.x == 0.0 {
                0.0
            } else {
                1.0 / self.depth_of_field.x
            };
            let calculated_near_field =
                (converted_pixel_value - calculated_focus_point) / calculated_focus_point;
            calculated_value =
                -calculated_near_field.min(self.settings.max_size / self.settings.size)
        }
        calculated_value * self.settings.size
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;
    use crate::CameraData;
    use approx::{self, relative_eq};
    use serde_json::Value;

    #[derive(Debug)]
    struct TestResult {
        #[allow(dead_code)]
        pub settings: Settings,
        #[allow(dead_code)]
        pub coc: f32,
        pub result: f32,
        pub expected: f32,
    }

    impl TestResult {
        pub fn is_success(&self) -> bool {
            !relative_eq!(self.result, self.expected, epsilon = 1e-2)
        }
    }

    fn case_to_settings(case: &Value) -> Settings {
        let mut settings = Settings::default();
        if case["size"].is_f64() {
            settings.size = case["size"].as_f64().unwrap() as f32;
        }
        if case["max_size"].is_f64() {
            settings.max_size = case["max_size"].as_f64().unwrap() as f32;
        }
        if case["focal_plane"].is_f64() {
            settings.focal_plane = case["focal_plane"].as_f64().unwrap() as f32;
        }
        if let Some(camera_data) = case.get("camera_data") {
            let mut camera_instance = CameraData::default();
            if camera_data["focal_length"].is_f64() {
                camera_instance.focal_length = camera_data["focal_length"].as_f64().unwrap() as f32;
            }
            if camera_data["f_stop"].is_f64() {
                camera_instance.f_stop = camera_data["f_stop"].as_f64().unwrap() as f32;
            }

            settings.camera_data = Some(camera_instance);
        }
        if case["math"].is_string() {
            settings.math = match case["math"].as_str().unwrap() {
                "ONE_DIVIDED_BY_Z" => Math::OneDividedByZ,
                _ => Math::Real,
            }
            .into()
        }

        settings
    }

    #[test]
    fn test_calculation() {
        let cases = include_str!("../test/cases.json");

        let json: Vec<Value> = serde_json::from_str(cases).unwrap();

        let mut results = Vec::new();
        for case in json.iter() {
            let settings = case_to_settings(&case["settings"]);
            let expected = case["expected"].as_f64().unwrap() as f32;
            let coc = case["coc"].as_f64().unwrap() as f32;
            let calculator = Calculator::new(settings);
            let result = calculator.calculate(coc);
            results.push(TestResult {
                settings,
                coc,
                result,
                expected,
            });
        }

        results.retain(|f| f.is_success());

        if results.is_empty() {
            return;
        }

        for (i, result) in results.iter().enumerate() {
            eprintln!("Test case '{i}' failed with input: {:#?}", result)
        }
        panic!("Test failed")
    }
}

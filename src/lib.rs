#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod calculator;
mod datamodel {
    include!(concat!(env!("OUT_DIR"), "/circle_of_confusion.rs"));
}

pub use crate::datamodel::*;

#[cfg(feature = "wasm-bindings")]
mod wasm {
    /// As wasm memory is linear, the memory starts at null. But this doesn't work well with LTO.
    const PTR_OFFSET: usize = 1;

    use crate::ffi_result::ResultValue;

    use super::*;
    use prost::Message;

    fn get_result_protobuf() -> Result<(&'static mut [u8], FfiResult), usize> {
        let ptr = (get_calculator_size() + PTR_OFFSET) as *mut u8;
        let len = get_result_size();
        let result_buffer = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        let current_result = FfiResult::default();
        Ok((result_buffer, current_result))
    }

    #[unsafe(no_mangle)]
    /// Get the max size of the Calculator object.
    pub extern "C" fn get_settings_size() -> usize {
        size_of::<Settings>()
            + size_of::<CameraData>()
            + size_of::<Filmback>()
            + size_of::<Resolution>()
            + size_of::<WorldUnit>()
            + size_of::<Math>()
    }

    #[unsafe(no_mangle)]
    /// Get the max size of the Calculator object.
    pub extern "C" fn get_calculator_size() -> usize {
        size_of::<Calculator>() + size_of::<DepthOfField>() + get_settings_size()
    }

    #[unsafe(no_mangle)]
    /// Get the max size of the Result object.
    pub extern "C" fn get_result_size() -> usize {
        size_of::<FfiResult>()
    }

    /// Wrapper for the inner calculator to work with results
    unsafe fn initialize_calculator_inner(
        settings_size: usize,
    ) -> Result<Calculator, FfiError> {
        let address = PTR_OFFSET as *mut u8;
        let settings = unsafe {
            let data = core::slice::from_raw_parts(address, settings_size);
            Settings::decode(data)
        }
        .map_err(|_| FfiError::ProtoDecode)?;

        let calculator = Calculator::new(settings);
        let max_calculator_size = get_calculator_size();
        calculator
            .encode(&mut unsafe { core::slice::from_raw_parts_mut(address, max_calculator_size) })
            .map_err(|_| FfiError::ProtoEncode)?;

        Ok(calculator)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn initialize_calculator(settings_size: usize) -> usize {
        let (mut result_buffer, mut current_result) = match get_result_protobuf() {
            Ok(value) => value,
            Err(_) => return 0,
        };

        match unsafe { initialize_calculator_inner(settings_size) } {
            Ok(calculator) => {
                current_result.result_value =
                    Some(ResultValue::UintValue(calculator.encoded_len() as u32))
            }
            Err(error) => {
                current_result.result_value = Some(ResultValue::Error(error.into()));
            }
        }
        let _ = current_result.encode(&mut result_buffer);
        current_result.encoded_len()
    }

    #[allow(invalid_null_arguments)]
    unsafe fn calculate_inner(calculator_size: usize, value: f32) -> Result<f32, FfiError> {
        let ptr = PTR_OFFSET as *const u8;

        let current_calculator = unsafe {
            let data = core::slice::from_raw_parts(ptr, calculator_size);
            Calculator::decode(data)
        }
        .map_err(|_| FfiError::ProtoDecode)?;

        Ok(current_calculator.calculate(value))
    }

    #[unsafe(no_mangle)]
    pub fn calculate(value: f32, calculator_size: usize) -> usize {
        let (mut result_buffer, mut current_result) = match get_result_protobuf() {
            Ok(value) => value,
            Err(value) => return value,
        };
        match unsafe { calculate_inner(calculator_size, value) } {
            Ok(value) => current_result.result_value = Some(ResultValue::FloatValue(value)),
            Err(error) => current_result.result_value = Some(ResultValue::Error(error.into())),
        };

        let _ = current_result.encode(&mut result_buffer);
        current_result.encoded_len()
    }
}

#[cfg(feature = "wasm-bindings")]
pub use wasm::*;

#[cfg(not(feature = "wasm-bindings"))]
pub fn initialize_calculator(settings: Settings) -> Calculator {
    Calculator::new(settings)
}

#[cfg(not(feature = "wasm-bindings"))]
pub fn calculate(calculator: &Calculator, value: f32) -> f32 {
    calculator.calculate(value)
}

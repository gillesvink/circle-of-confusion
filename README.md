[![CI](https://github.com/gillesvink/circle-of-confusion/actions/workflows/CI.yml/badge.svg)](https://github.com/gillesvink/circle-of-confusion/actions/workflows/CI.yml)


# Circle of Confusion

Calculator for Circle of Confusion (CoC) to calculate the size in pixels of an area, used for depth of field processing.

It's built in Rust. To use the library in no-std: enable the `no-std` feature.
For the Python package it exposes its functions via [PyO3](https://pyo3.rs/latest/).

Add the project to your Cargo.toml by using
```bash
cargo add circle-of-confusion

# or for no-std:
cargo add circle-of-confusion --features no-std
```

Or in your Python project (managed by uv, but it's just available in pip as well):
```bash
uv add circle-of-confusion
```

## Usage
The calculator is able to calculate the Circle of Confusion based on the provided settings.
The size in px is the radius of the convolution.
A CoC of 10 would mean a diameter of 20 pixels.

### Output:
* `+` is for far field pixels
* `-` is for near field pixels

### Modes
The calculator supports two modes, one is physically accurate,
the other lets you tune your own DoF size.

#### Manually
When no camera data is provided to the Settings struct (just give the parameter a `None`), the manual mode will be used.
This creates a smooth falloff to the focal plane point, based on the size and max size added.
It will gradually apply the size based on the distance from the focal plane.
When using a larger size, the CoC will be increased.
Protect can be used to apply a safe region to the focal plane.

#### Camera
When the camera data is applied to the settings,
the camera values will be used instead. This matches real world CoC values.
Lowering f-stop will increase the CoC values, just like increasing the focal-length would.
This calculation is based on the CoC algorithm:
[Wikipedia](https://en.wikipedia.org/wiki/Circle_of_confusion)


### Examples
It's really simple to use, you need to assemble the settings to calculate the circle of confusion. The interface identical (besides the obvious syntax differences) for Rust and Python. For example for camera based calculations:

#### Python
```python
from circle_of_confusion import Calculator, Settings, Math, CameraData, WorldUnit

camera_data = CameraData(
    focal_length=100.0,
    f_stop=2.0,
    filmback=(24.576, 18.672),
    near_field=0.1,
    far_field=10000.0,
    world_unit=WorldUnit.M,
    resolution=(1920, 1080),
)
settings = Settings(
    size=10.0,
    max_size=100.0,
    math=Math.REAL,
    focal_plane=30.0,
    protect=0.0,
    pixel_aspect=1.0,
    camera_data=camera_data,
)
calculator = Calculator(settings)
result = calculator.calculate(10.0) # input distance value from Z-depth
assert result == 11.93532943725586
```

#### Rust
```rust
use circle_of_confusion::{Calculator, Settings, Math, CameraData, WorldUnit};

fn main() {
    let camera_data = CameraData {
        focal_length: 100.0,
        f_stop: 2.0,
        filmback: [24.576, 18.672],
        near_field: 0.1,
        far_field: 10000.0,
        world_unit: WorldUnit::M,
        resolution: [1920, 1080],
    };
    let settings = Settings {
        size: 10.0,
        max_size: 100.0,
        math: Math::REAL,
        focal_plane: 30.0,
        protect: 0.0,
        pixel_aspect: 1.0,
        camera_data: Some(camera_data),
    };
    let calculator = Calculator::new(settings);
    let result = calculator.calculate(10.0); // input distance value from Z-depth
    assert_eq!(result, 11.935329);
}
```
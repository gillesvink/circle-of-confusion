# Circle of Confusion

Calculator for Circle of Confusion (CoC) to calculate the size in pixels of an area, used for depth of field processing.

It's built in Rust, by default it is no-std only. 

For the Python package it exposes its functions via [PyO3](https://pyo3.rs/latest/).

## Usage

It's really simple to use, you need to assemble the settings to calculate the circle of confusion. The interface is almost identical for Rust and Python. For example for camera based calculations:

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
result = calculator.calculate(10)
assert result == 11.93532943725586
```


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
    let result = calculator.calculate(10.0);
    assert_eq!(result, 11.935329);
}

```
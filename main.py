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
    size=10,
    max_size=100,
    math=Math.REAL,
    focal_plane=30.0,
    protect=0.0,
    pixel_aspect=1.0,
    camera_data=camera_data,
)
calculator = Calculator(settings)
value = calculator.calculate(10)
print(value)
assert calculator.calculate(10) == -11.93532943725586

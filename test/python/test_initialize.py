import pytest
from circle_of_confusion import (
    Settings,
    Calculator,
    initialize_calculator,
    CameraData,
)
from circle_of_confusion._exception import CircleOfConfusionError


def test_initialize():
    settings = Settings(camera_data=CameraData(focal_length=51), focal_plane=20.0)
    calculator: Calculator = initialize_calculator(settings)

    assert calculator._inner_calculator.settings.focal_plane == 20.0
    assert calculator._inner_calculator.settings.camera_data.focal_length == 51


def test_initialize_with_invalid_object():
    with pytest.raises(CircleOfConfusionError, match="Provided settings is not a valid settings object: '<class 'str'>'"):
        initialize_calculator("im not a settings object")




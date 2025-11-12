"""Tests for initialization of wasm."""

# ruff: noqa: SLF001, S101

import pytest
from circle_of_confusion import (
    Calculator,
    CameraData,
    Settings,
    initialize_calculator,
)
from circle_of_confusion._exception import CircleOfConfusionError


def test_initialize() -> None:
    """Test initialization of calculator."""
    focal_length = 51
    focal_plane = 20
    settings = Settings(
        camera_data=CameraData(focal_length=focal_length),
        focal_plane=focal_plane,
    )
    calculator: Calculator = Calculator(settings)

    assert calculator._inner_calculator.settings.focal_plane == focal_plane
    assert (
        calculator._inner_calculator.settings.camera_data.focal_length == focal_length
    )


def test_initialize_with_invalid_object() -> None:
    """Test initialize fail with invalid object passed."""
    with pytest.raises(
        CircleOfConfusionError,
        match="Provided settings is not a valid settings object: '<class 'str'>'",
    ):
        initialize_calculator("im not a settings object")  # ty: ignore[invalid-argument-type]

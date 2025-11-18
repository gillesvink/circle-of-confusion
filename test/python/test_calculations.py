"""Tests for the wasm calculation."""

import json
import logging
from dataclasses import dataclass
from pathlib import Path

import pytest
from _pytest.python_api import ApproxBase
from circle_of_confusion import (
    CameraData,
    Math,
    Settings,
    calculate,
    initialize_calculator,
)
from circle_of_confusion._exception import CircleOfConfusionError
from google.protobuf.json_format import MessageToJson

logger = logging.getLogger(__name__)
CASES: Path = (Path(__file__).parent.parent / "cases.json").resolve()
"""Path to cases.json"""


def _case_to_settings(settings_json: dict) -> Settings:
    """Parse the json into a settings object (quick and dirty implementaiton)."""
    settings = Settings()

    if settings_json.get("camera_data"):
        settings = Settings(camera_data=CameraData())
        if settings_json["camera_data"].get("focal_length"):
            settings.camera_data.focal_length = settings_json["camera_data"][
                "focal_length"
            ]
        if settings_json["camera_data"].get("f_stop"):
            settings.camera_data.f_stop = settings_json["camera_data"]["f_stop"]
    else:
        settings = Settings()
    if settings_json.get("size"):
        settings.size = settings_json["size"]
    if settings_json.get("max_size"):
        settings.max_size = settings_json["max_size"]
    if settings_json.get("focal_plane"):
        settings.focal_plane = settings_json["focal_plane"]
    if settings_json.get("math"):
        if settings_json["math"] == "REAL":
            settings.math = Math.REAL
        else:
            settings.math = Math.ONE_DIVIDED_BY_Z

    return settings


@dataclass
class Result:
    """Test result data container."""

    settings: str
    coc: float
    result: float
    expected: ApproxBase

    def is_success(self) -> bool:
        """Verify result to match expected."""
        return self.result == self.expected


def test_calculations() -> None:
    """Test calculations to match the cases.json."""
    test_cases = json.loads(CASES.read_text())

    results: list[Result] = []
    for test_case in test_cases:
        settings = _case_to_settings(test_case["settings"])
        calculator = initialize_calculator(settings)
        result = calculate(calculator, test_case["coc"])
        results.append(
            Result(
                MessageToJson(settings),
                test_case["coc"],
                result,
                pytest.approx(
                    test_case["expected"],
                    0.01,  # roughly match it
                ),
            ),
        )

    result = [result for result in results if not result.is_success()]
    if not result:
        return

    for i, item in enumerate(result):
        msg = f"Test case '{i}' failed with input: '{item}'"
        logger.error(msg)
    pytest.fail("Some of the tests did not match the expected result")


def test_calculation_with_invalid_object() -> None:
    """Test calculation to fail with invalid object passed."""
    with pytest.raises(
        CircleOfConfusionError,
        match="Provided Calculator is not a valid Calculator object: '<class 'str'>'",
    ):
        calculate("im not a settings object", 20)  # ty: ignore[invalid-argument-type]


def test_calculation_with_no_float_provided() -> None:
    """Test calculation to fail when no value is provided."""
    with pytest.raises(
        CircleOfConfusionError,
        match="No correct distance value provided: '<class 'NoneType'>'",
    ):
        calculate(initialize_calculator(Settings()), None)  # ty: ignore[invalid-argument-type]


def test_calculation_with_nan_provided() -> None:
    """Test calculation to fail when nan is provided."""
    with pytest.raises(
        CircleOfConfusionError,
        match="Provided distance is not a number",
    ):
        calculate(initialize_calculator(Settings()), float("nan"))

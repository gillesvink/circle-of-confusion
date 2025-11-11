import pytest
from _pytest.python_api import ApproxBase
from google.protobuf.json_format import MessageToJson
from circle_of_confusion import (
    Settings,
    calculate,
    initialize_calculator,
    CameraData,
)
from pathlib import Path
import json
from dataclasses import dataclass
from circle_of_confusion._exception import CircleOfConfusionError
from circle_of_confusion import Math

CASES: Path = (Path(__file__).parent.parent / "cases.json").resolve()
"""Path to cases.json"""


def _case_to_settings(settings_json: dict) -> Settings:
    """A quick and dirty implementation to parse the json into a settings object."""
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
    settings: Settings
    coc: float
    result: float
    expected: ApproxBase

    def is_success(self) -> bool:
        return self.result == self.expected

def test_calculations():
    """Test calculations to match the cases.json"""
    test_cases = json.loads(CASES.read_text())

    results: list[Result] = []

    for i, test_case in enumerate(test_cases):
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
            )
        )

    result = [result for result in results if not result.is_success()]
    if not result:
        return
    
    for i, result in enumerate(result):
        msg = f"Test case '{i}' failed with input: '{result}'"
        print(msg)
    assert False

def test_calculation_with_invalid_object():
    with pytest.raises(CircleOfConfusionError, match="Provided Calculator is not a valid Calculator object: '<class 'str'>'"):
        calculate("im not a settings object", 20)

def test_calculation_with_no_float_provided():
    with pytest.raises(CircleOfConfusionError, match="No correct distance value provided: '<class 'NoneType'>'"):
        calculate(initialize_calculator(Settings()), None)


def test_calculation_with_nan_provided():
    with pytest.raises(CircleOfConfusionError, match="Provided distance is not a number"):
        calculate(initialize_calculator(Settings()), float('nan'))

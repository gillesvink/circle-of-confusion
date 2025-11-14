"""Library to calculate the Circle of Confusion for specified variables."""

# ruff: noqa: F401

from _circle_of_confusion.circle_of_confusion_pb2 import (
    CameraData,
    CircleOfConfusionSettings,
    Filmback,
    Math,
    Resolution,
    WorldUnit,
)

from circle_of_confusion._exception import CircleOfConfusionError
from circle_of_confusion._ffi import (
    CircleOfConfusionCalculator,
    calculate,
    initialize_calculator,
)

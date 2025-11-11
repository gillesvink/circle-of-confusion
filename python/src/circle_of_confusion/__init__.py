"""Library to calculate the Circle of Confusion for specified variables."""

# flake8: noqa F401

from circle_of_confusion._ffi import Calculator, initialize_calculator, calculate
from _circle_of_confusion.circle_of_confusion_pb2 import (
    CameraData,
    WorldUnit,
    Math,
    Settings,
    Filmback,
    Resolution,
)
from circle_of_confusion._exception import CircleOfConfusionError

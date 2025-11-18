"""Library to calculate the Circle of Confusion for specified variables."""

# ruff: noqa: F401

import sys

from circle_of_confusion._exception import CircleOfConfusionError
from circle_of_confusion._features import _WASMTIME, _get_features

if sys.version_info[:2] < (3, 11) and _WASMTIME not in _get_features():
    msg = (
        "Wasmtime feature not enabled and version lower than 3.11. "
        "Either enable it or use a newer Python version."
    )
    raise CircleOfConfusionError(msg)

from _circle_of_confusion.circle_of_confusion_pb2 import (
    CameraData,
    Filmback,
    Math,
    Resolution,
    Settings,
    WorldUnit,
)

from circle_of_confusion._ffi import (
    Calculator,
    calculate,
    initialize_calculator,
)

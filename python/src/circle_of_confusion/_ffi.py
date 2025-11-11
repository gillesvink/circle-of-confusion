"""Module for communicating with wasm runtime."""

from functools import lru_cache
import math
from circle_of_confusion._exception import CircleOfConfusionError
from _circle_of_confusion import circle_of_confusion_pb2
import sys
from wasmtime import (
    Store,
    Module,
    Instance,
    Func,
    Memory,
)
from wasmtime._instance import InstanceExports
from pathlib import Path
from dataclasses import dataclass

_PTR_OFFSET: int = 1
"""Offset as wasm memory starts at 0, but this would be a nullptr which can cause undefined behaviour."""
_CALCULATE: str = "calculate"
"""Wasm function for calculations"""
_INITIALIZE_CALCULATOR: str = "initialize_calculator"
"""Wasm function name for initializing a calculator based on the settings provided."""
_GET_CALCULATOR_SIZE: str = "get_calculator_size"
"""Wasm function to get max size of calculator in bytes"""
_GET_RESULT_SIZE: str = "get_result_size"
"""Wasm function to get max size of result in bytes"""
_WASM_NAME: str = "circle_of_confusion.wasm"
"""Name of Wasm binary"""


@dataclass(frozen=True)
class Calculator:
    """Calculator instance that is able to calculate the circle of confusion value.

    It holds the wasm instance including memory, so it is fast as it does not need to
    write anything to memory.
    """

    _store: Store
    _inner_calculator: circle_of_confusion_pb2.Calculator
    _size: int
    _exports: InstanceExports


def initialize_calculator(
    settings: circle_of_confusion_pb2.Settings,
) -> Calculator:
    """Initialize the calculator based on the settings provided.

    Args:
        settings: settings to calculate coc with.

    Returns:
        calculator instance able to calculate coc values
    """
    if not isinstance(settings, circle_of_confusion_pb2.Settings):
        msg = f"Provided settings is not a valid settings object: '{type(settings)}'"
        raise CircleOfConfusionError(msg)
    store = Store()
    exports = _initialize_wasm(store)
    memory: Memory = exports["memory"]
    initialize_calculator_wasm: Func = exports[_INITIALIZE_CALCULATOR]

    _set_memory_size(store, memory)

    settings_bytes = settings.SerializePartialToString()
    memory.write(store, settings_bytes, 1)
    result_size = initialize_calculator_wasm(store, len(settings_bytes))
    result = _get_result(store, memory, result_size)

    calculator = circle_of_confusion_pb2.Calculator.FromString(
        memory.read(store, _PTR_OFFSET, result.uint_value + _PTR_OFFSET)
    )
    return Calculator(store, calculator, result.uint_value, exports)


def _get_result(
    store, memory: Memory, result_size
) -> circle_of_confusion_pb2.FFIResult:
    """Map the result from memory into a FFIResult object."""
    calculator_size = _get_calculator_size()
    if result_size == 0:
        raise CircleOfConfusionError("Buffer did not have enough space to write to.")

    result = circle_of_confusion_pb2.FFIResult.FromString(
        memory.read(
            store,
            calculator_size + _PTR_OFFSET,
            calculator_size + result_size + _PTR_OFFSET,
        )
    )
    if result.WhichOneof("ResultValue") == "error":
        raise CircleOfConfusionError.map_error(result.error)
    return result


def calculate(calculator: Calculator, distance: float) -> float:
    """Calculate circle of confusion based on provided distance value.
    
    Args:
        calculator: instance of the calculator, needs to be created before calling this
        distance: distance in world unit from camera
    """
    if not isinstance(calculator, Calculator):
        msg = f"Provided Calculator is not a valid Calculator object: '{type(calculator)}'"
        raise CircleOfConfusionError(msg)

    if not isinstance(distance, float):
        msg = f"No correct distance value provided: '{type(distance)}'"
        raise CircleOfConfusionError(msg)
    
    if math.isnan(distance):
        msg = "Provided distance is not a number"
        raise CircleOfConfusionError(msg)

    memory: Memory = calculator._exports["memory"]
    calculate_wasm: Func = calculator._exports[_CALCULATE]

    result_size = calculate_wasm(calculator._store, distance, calculator._size)
    result = _get_result(calculator._store, memory, result_size)

    return result.float_value


def _set_memory_size(store: Store, memory: Memory):
    """Set the memory size according to the page size of 64.

    It just gets the max size of calculator and result, and calcultes if memory
    is big enough or need to allocate some more.
    """
    memory_size = math.ceil((_get_calculator_size() + _get_result_size() + _PTR_OFFSET) / 64)
    current_size = memory.size(store)
    if current_size <= memory_size:
        memory.grow(memory_size - current_size)


def _initialize_wasm(store: Store):
    """Initialize the wasm runtime."""
    module = Module.from_file(store.engine, _get_wasm_filepath())
    instance = Instance(store, module, [])
    exports = instance.exports(store)
    return exports


@lru_cache(maxsize=1)
def _get_wasm_filepath() -> Path:
    """Get the path to the wasm file."""
    for directory in sys.path:
        path = Path(directory) / "_circle_of_confusion" / _WASM_NAME
        if path.is_file():
            return path
    raise CircleOfConfusionError(
        "Wasm binary could not be located in PATH"
    )


@lru_cache(maxsize=1)
def _get_calculator_size() -> int:
    """Get the max byte size of the `Calculator` for allocation purposes."""
    store = Store()
    exports = _initialize_wasm(store)
    get_calculator_size: Func = exports[_GET_CALCULATOR_SIZE]
    return get_calculator_size(store)


@lru_cache(maxsize=1)
def _get_result_size() -> int:
    """Get the max byte size of the `FFIResult` for allocation purposes."""
    store = Store()
    exports = _initialize_wasm(store)
    get_calculator_size: Func = exports[_GET_RESULT_SIZE]
    return get_calculator_size(store)

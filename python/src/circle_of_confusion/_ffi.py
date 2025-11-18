"""Module for communicating with wasm runtime."""

import math
import sys
from functools import lru_cache
from pathlib import Path

from _circle_of_confusion import circle_of_confusion_pb2

from circle_of_confusion._exception import CircleOfConfusionError
from circle_of_confusion._wasm import (
    Memory,
    Module,
    Store,
)
from circle_of_confusion._wasm.abstract import (
    AbstractMemory,
    AbstractModule,
    AbstractStore,
)

_PTR_OFFSET: int = 1
"""Start address of data in wasm memory."""
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


class Calculator:
    """Calculator instance that is able to calculate the circle of confusion value.

    It holds the wasm instance including memory, so it is fast as it does not need to
    write anything to memory.
    """

    def __init__(
        self,
        settings: circle_of_confusion_pb2.Settings,
    ) -> None:
        if not isinstance(settings, circle_of_confusion_pb2.Settings):
            msg = (
                f"Provided settings is not a valid settings object: '{type(settings)}'"
            )
            raise CircleOfConfusionError(msg)
        self._store = Store()
        self._exports = _initialize_wasm(self._store)
        self._memory = Memory.from_module(self._exports, self._store)

        _set_memory_size(self._store, self._memory)

        settings_bytes = bytearray(settings.SerializePartialToString())
        self._memory.write(self._store, settings_bytes, 1)
        result_size = self._exports.run(
            self._store,
            _INITIALIZE_CALCULATOR,
            [len(settings_bytes)],
        )
        result = _get_result(self._store, self._memory, result_size)

        self._inner_calculator = circle_of_confusion_pb2.Calculator.FromString(
            self._memory.read(
                self._store,
                _PTR_OFFSET,
                result.uint_value + _PTR_OFFSET,
            ),
        )
        self._size = result.uint_value

    @property
    def store(self) -> Store:
        return self._store

    @property
    def exports(self) -> AbstractModule:
        return self._exports

    @property
    def size(self) -> int:
        return self._size

    @property
    def memory(self) -> AbstractMemory:
        return self._memory


def initialize_calculator(
    settings: circle_of_confusion_pb2.Settings,
) -> Calculator:
    """Initialize the calculator based on the settings provided.

    Args:
        settings: settings to calculate coc with.

    Returns:
        calculator instance able to calculate coc values

    """
    return Calculator(settings)


def _get_result(
    store: AbstractStore,
    memory: AbstractMemory,
    result_size: int,
) -> circle_of_confusion_pb2.FFIResult:
    """Map the result from memory into a FFIResult object."""
    calculator_size = _get_calculator_size()
    if result_size == 0:
        msg = "Buffer did not have enough space to write to."
        raise CircleOfConfusionError(msg)

    result = memory.read(
        store,
        calculator_size + _PTR_OFFSET,
        calculator_size + result_size + _PTR_OFFSET,
    )
    result: circle_of_confusion_pb2.FFIResult = (
        circle_of_confusion_pb2.FFIResult.FromString(
            bytes(result),
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
        msg = (
            "Provided Calculator is not a valid "
            f"Calculator object: '{type(calculator)}'"
        )

        raise CircleOfConfusionError(msg)

    if not isinstance(distance, float):
        msg = f"No correct distance value provided: '{type(distance)}'"
        raise CircleOfConfusionError(msg)

    if math.isnan(distance):
        msg = "Provided distance is not a number"
        raise CircleOfConfusionError(msg)

    memory: AbstractMemory = calculator.memory

    result_size = calculator.exports.run(
        calculator.store,
        _CALCULATE,
        [distance, calculator.size],
    )
    result = _get_result(calculator.store, memory, result_size)

    return result.float_value


def _set_memory_size(store: Store, memory: AbstractMemory) -> None:
    """Set the memory size according to the page size of 64.

    It just gets the max size of calculator and result, and calcultes if memory
    is big enough or need to allocate some more.
    """
    memory_size = math.ceil(
        (_get_calculator_size() + _get_result_size() + _PTR_OFFSET) / 64,
    )
    current_size = memory.size(store)
    if current_size <= memory_size:
        memory.grow(store, memory_size - current_size)


def _initialize_wasm(store: Store) -> AbstractModule:
    """Initialize the wasm runtime."""
    return Module.from_file(store, _get_wasm_filepath())


@lru_cache(maxsize=1)
def _get_wasm_filepath() -> Path:
    """Get the path to the wasm file."""
    for directory in sys.path:
        path = Path(directory) / "_circle_of_confusion" / _WASM_NAME
        if path.is_file():
            return path
    msg = "Wasm binary could not be located in PATH"
    raise CircleOfConfusionError(msg)


@lru_cache(maxsize=1)
def _get_calculator_size() -> int:
    """Get the max byte size of the `Calculator` for allocation purposes."""
    store = Store()
    exports = _initialize_wasm(store)
    return exports.run(store, _GET_CALCULATOR_SIZE, [])


@lru_cache(maxsize=1)
def _get_result_size() -> int:
    """Get the max byte size of the `FFIResult` for allocation purposes."""
    store = Store()
    exports = _initialize_wasm(store)
    return exports.run(store, _GET_RESULT_SIZE, [])

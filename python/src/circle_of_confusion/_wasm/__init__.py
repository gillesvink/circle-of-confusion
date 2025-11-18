# ruff: noqa: F401

import importlib

from circle_of_confusion._features import _WASMTIME, _get_features

if _WASMTIME in _get_features():
    from circle_of_confusion._wasm._wasmtime import WasmtimeMemory as Memory
    from circle_of_confusion._wasm._wasmtime import WasmtimeModule as Module
    from circle_of_confusion._wasm._wasmtime import WasmtimeStore as Store
else:
    from circle_of_confusion._wasm._pywasm import PywasmMemory as Memory
    from circle_of_confusion._wasm._pywasm import PywasmModule as Module
    from circle_of_confusion._wasm._pywasm import PywasmStore as Store

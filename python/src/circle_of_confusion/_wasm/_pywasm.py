"""Module to wrap around pywasm and wasmtime to support any runtime."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pywasm

from circle_of_confusion._wasm.abstract import (
    AbstractMemory,
    AbstractModule,
    AbstractStore,
)

if TYPE_CHECKING:
    from pathlib import Path


class PywasmStore(AbstractStore):
    _instance: pywasm.Runtime

    def __init__(self) -> None:
        self._instance = pywasm.Runtime()

    @property
    def instance(self) -> pywasm.Runtime:
        return self._instance

    def get_memory(self, module: AbstractModule) -> AbstractMemory:
        return PywasmMemory.from_module(module, self)


class PywasmMemory(AbstractMemory):
    _instance: pywasm.MemInst

    @classmethod
    def from_module(
        cls,
        module: AbstractModule,
        store: AbstractStore,
    ) -> AbstractMemory:
        memory = PywasmMemory()
        memory._instance = store.instance.exported_memory(
            module.instance,
            "memory",
        )
        return memory

    def size(self, store: AbstractStore) -> int:  # noqa: ARG002
        return self._instance.size

    def grow(self, store: AbstractStore, delta: int) -> None:  # noqa: ARG002
        self._instance.grow(delta)

    def read(self, store: AbstractStore, start: int, end: int) -> bytearray:  # noqa: ARG002
        return self._instance.get(start, end - start)

    def write(self, store: AbstractStore, data: bytearray, start: int) -> None:  # noqa: ARG002
        self._instance.put(start, data)


class PywasmModule(AbstractModule):
    _instance: pywasm.ModuleInst

    @classmethod
    def from_file(cls, store: AbstractStore, path: Path) -> AbstractModule:
        instance = PywasmModule()
        instance._instance = store.instance.instance_from_file(str(path))
        return instance

    @property
    def instance(self) -> pywasm.ModuleInst:
        return self._instance

    def run(self, store: PywasmStore, func: str, arguments: list) -> int:
        return int(store.instance.invocate(self._instance, func, arguments)[0])

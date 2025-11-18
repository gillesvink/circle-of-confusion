"""Module to wrap around pywasm and wasmtime to support any runtime."""

from __future__ import annotations

from typing import TYPE_CHECKING

import wasmtime

from circle_of_confusion._wasm.abstract import (
    AbstractMemory,
    AbstractModule,
    AbstractStore,
)

if TYPE_CHECKING:
    from pathlib import Path

    from wasmtime._instance import InstanceExports


class WasmtimeStore(AbstractStore):
    _instance: wasmtime.Store

    def __init__(self) -> None:
        self._instance = wasmtime.Store()

    @property
    def instance(self) -> wasmtime.Store:
        return self._instance

    def get_memory(self, module: WasmtimeModule) -> WasmtimeMemory:
        return WasmtimeMemory.from_module(module, self)


class WasmtimeMemory(AbstractMemory):
    _instance: wasmtime.Memory

    @classmethod
    def from_module(
        cls,
        module: AbstractModule,
        store: AbstractStore,  # noqa: ARG003
    ) -> WasmtimeMemory:
        memory = WasmtimeMemory()
        memory._instance = module.instance["memory"]
        return memory

    def size(self, store: AbstractStore) -> int:
        return self._instance.size(store.instance)

    def grow(self, store: AbstractStore, delta: int) -> None:
        self._instance.grow(store.instance, delta)

    def read(self, store: AbstractStore, start: int, end: int) -> bytearray:
        return self._instance.read(store.instance, start, end)

    def write(self, store: AbstractStore, data: bytearray, start: int) -> None:
        self._instance.write(store.instance, data, start)


class WasmtimeModule(AbstractModule):
    _instance: InstanceExports

    @classmethod
    def from_file(cls, store: AbstractStore, path: Path) -> AbstractModule:
        instance = WasmtimeModule()

        module = wasmtime.Module.from_file(store.instance.engine, path)
        wasmtime_instance = wasmtime.Instance(store.instance, module, [])
        instance._instance = wasmtime_instance.exports(store.instance)
        return instance

    @property
    def instance(self) -> InstanceExports:
        return self._instance

    def run(self, store: AbstractStore, func: str, arguments: list) -> int:
        target_func: wasmtime.Func = self._instance[func]
        return target_func(store.instance, *arguments)

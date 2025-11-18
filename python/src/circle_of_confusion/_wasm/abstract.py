"""Module to wrap around pywasm and wasmtime to support any runtime."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


class AbstractStore(ABC):
    @abstractmethod
    def __init__(self) -> None: ...

    @property
    @abstractmethod
    def instance(self) -> Any:  # noqa: ANN401
        ...

    @abstractmethod
    def get_memory(self, module: AbstractModule) -> AbstractMemory: ...


class AbstractMemory:
    @classmethod
    @abstractmethod
    def from_module(
        cls,
        module: AbstractModule,
        store: AbstractStore,
    ) -> AbstractMemory: ...

    @abstractmethod
    def size(self, store: AbstractStore) -> int: ...

    @abstractmethod
    def grow(self, store: AbstractStore, delta: int) -> None: ...

    @abstractmethod
    def read(self, store: AbstractStore, start: int, end: int) -> bytearray: ...

    @abstractmethod
    def write(self, store: AbstractStore, data: bytearray, start: int) -> None: ...


class AbstractModule:
    @classmethod
    @abstractmethod
    def from_file(cls, store: AbstractStore, path: Path) -> AbstractModule: ...

    @property
    @abstractmethod
    def instance(self) -> Any:  # noqa: ANN401
        ...

    @abstractmethod
    def run(self, store: AbstractStore, func: str, arguments: list) -> int: ...

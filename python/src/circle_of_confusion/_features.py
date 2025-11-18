import importlib.util

_WASMTIME: str = "wasmtime"
"""Feature that uses the wasmtime runtime"""


def _get_features() -> list[str]:
    return [_WASMTIME] if importlib.util.find_spec(_WASMTIME) else []

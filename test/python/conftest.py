"""Setup for pytest."""

import logging
from contextlib import suppress

logging.basicConfig(level=logging.DEBUG)

with suppress(ImportError):
    from pytest_readme import setup

    setup()

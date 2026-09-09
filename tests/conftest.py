"""Shared controls for reference data and operator-tool tests."""

# ruff: noqa: E402 — BLAS caps must precede scientific library imports.
import os as _os

for _blas_var in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
    "RAYON_NUM_THREADS",
):
    _os.environ.setdefault(_blas_var, "1")

# Env vars only bind before the BLAS lib loads; threadpoolctl also caps an
# already-loaded lib at runtime. Keep the limiter alive for the whole session
# via the module-global reference (its __del__ would otherwise restore threads).
try:
    from threadpoolctl import (
        threadpool_limits as _threadpool_limits,  # type: ignore[import-untyped]
    )
except ImportError:
    _BLAS_THREAD_LIMIT = None
else:
    _BLAS_THREAD_LIMIT = _threadpool_limits(limits=1)

import logging
import os
import random
from collections.abc import Generator

import pytest
from hypothesis import HealthCheck, settings
from pydantic_ai import models as _pydantic_ai_models

_pydantic_ai_models.ALLOW_MODEL_REQUESTS = False

settings.register_profile(
    "default",
    max_examples=100,
    derandomize=True,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow],
)
settings.register_profile(
    "slow",
    max_examples=500,
    derandomize=False,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow],
)
settings.load_profile(os.environ.get("HYPOTHESIS_PROFILE", "default"))


@pytest.fixture(autouse=True)
def _isolate_random_state() -> Generator[None, None, None]:
    """Keep data-tool randomness independent of test order."""
    saved_state = random.getstate()
    random.seed(42)
    try:
        yield
    finally:
        random.setstate(saved_state)


@pytest.fixture(autouse=True)
def enable_logging_propagation() -> Generator[None, None, None]:
    """Ensure caplog catches Babylon logs when a test changes propagation."""
    logger = logging.getLogger("babylon")
    old_propagate = logger.propagate
    logger.propagate = True
    yield
    logger.propagate = old_propagate

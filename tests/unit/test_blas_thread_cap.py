"""Guard: the BLAS/OpenMP thread cap must stay in force during the test suite.

The engine's numpy/scipy economics spawns one OpenBLAS thread per core (24 on the
dev box) per process by default. Under pytest-xdist (N workers) or parallel
agents that nested process x BLAS parallelism oversubscribes the CPU and stacks
per-thread buffers until the machine thrashes and the desktop freezes (it froze
the dev box twice on 2026-07-12). ``tests/conftest.py`` pins BLAS to 1 thread to
prevent this and to remove non-deterministic FP reduction order (Constitution
III.7). This test fails loudly if that pin is ever removed or defeated, so the
fix cannot silently regress.
"""

from __future__ import annotations

import os

import pytest

pytestmark = pytest.mark.unit


def test_blas_thread_env_is_pinned() -> None:
    """The conftest set the BLAS/OpenMP thread env vars to 1 before numpy loaded.

    ``RAYON_NUM_THREADS`` joined the pin in W1.8: rustworkx's centrality
    functions parallelize via rayon above ``parallel_threshold`` (default 50
    nodes) — the same oversubscription hazard (one rayon worker per core, per
    process) and additionally a determinism hazard (parallel float summation
    order) — Constitution III.7.
    """
    for var in (
        "OMP_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "MKL_NUM_THREADS",
        "RAYON_NUM_THREADS",
    ):
        assert os.environ.get(var) == "1", f"{var} is not pinned to 1 (conftest cap defeated?)"


def test_openblas_pool_is_capped() -> None:
    """The live OpenBLAS pool is capped, not one-thread-per-core.

    threadpoolctl reports the actual loaded-library thread count. Without the cap
    this is the core count (e.g. 24); with it, 1. Assert <= 2 to stay robust to a
    library that keeps a minimal 2-thread pool while still catching the 24-thread
    per-core explosion that causes the freeze.
    """
    threadpoolctl = pytest.importorskip("threadpoolctl")

    import numpy  # noqa: F401  (ensure the BLAS backend is loaded before we probe)

    blas_pools = [
        p["num_threads"]
        for p in threadpoolctl.threadpool_info()
        if p.get("internal_api") in {"openblas", "mkl", "blis"}
    ]
    assert blas_pools, "no BLAS backend detected — probe cannot verify the cap"
    worst = max(blas_pools)
    assert worst <= 2, (
        f"BLAS pool is {worst} threads — the conftest cap is not in force; under xdist "
        f"this multiplies to workers x {worst} threads and can freeze the machine"
    )

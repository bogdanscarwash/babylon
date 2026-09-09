"""The client dependency contract — retired estates never creep back.

Before the M7 cutover this file asserted the Textual stack imported cleanly;
since ``test(cutover)!: retire Textual Archive lane`` it asserts the
INVERSE, so the deleted estate can never silently creep back in through a
transitive pin. The Amendment AF (ADR186) deletion ceremony retired the
babylon_tui extension the same way in turn — its own former positive-half
test (``test_the_rust_client_extension_imports``, Task 44) is gone along
with the module it asserted; this file now asserts only absence.
"""

from __future__ import annotations

import importlib.util

import pytest

#: The retired Textual stack (M7 cutover, ADR150) plus babylon_tui (AF /
#: ADR186) — none of these may be installed (a transitive re-appearance
#: would mean a dependency regression, not harmless extra).
RETIRED_MODULES = (
    "textual",
    "textual_image",
    "textual_plotext",
    "pytest_textual_snapshot",
    "babylon_tui",
)


@pytest.mark.parametrize("module", RETIRED_MODULES)
def test_the_retired_client_stack_is_gone(module: str) -> None:
    assert importlib.util.find_spec(module) is None, (
        f"{module} is installed — this estate was retired (Textual: M7 "
        "cutover/ADR150; babylon_tui: Amendment AF/ADR186); a reappearing "
        "pin is a regression"
    )

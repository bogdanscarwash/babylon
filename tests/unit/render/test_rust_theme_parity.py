"""Cross-language ksbc palette parity: the Rust client's palette constants
must track :data:`babylon.render.tiers.TRUECOLOR_PALETTE` (the §9b SSOT).

Re-created for Program 28 B0 (Amendment AF, ADR186 sentinel disposition
table: this guard PORTS, it does not retire). The Ratatui client's
``rust/crates/babylon-tui/src/theme.rs`` is gone with the rest of that
estate; the guard now re-points at the Bevy client's
``rust/crates/babylon-client/src/palette.rs``, which restates the same §9b
role colors as ``Color::srgb_u8(r, g, b)`` literals — an FFI boundary no
import can cross, so this guard PARSES the Rust source instead (the same
drift class the Program 24 P7 palette-SSOT pass eliminated Python-side via
``test_design_bible_parity.py``: theme-local literals silently diverging
from §9b). A §9b revision that moves a token now turns this red instead of
leaving the Rust plates painting the stale color forever.

**F4 fix (adversarial verification of PR #490, B1).** The above covers only
``palette.rs`` — nothing stopped a raw ``Color::srgb_u8``/``Color::srgb``
literal from being declared in some OTHER file in the crate (B1's
``map/mesh.rs`` did exactly this for ``PANEL``, a deliberately-not-§9b
color), where NO guard watched it for drift. ``PANEL`` itself was correct
(never claimed to be a §9b token), but the gap was real: a stray literal
added anywhere else would have gone unnoticed indefinitely. The crate-wide
sweep below closes it: every ``Color::srgb[_u8]`` call in the crate must
live in ``palette.rs`` (covered by the tests above) or in a file named in
``_SWEEP_EXEMPTIONS`` with its own recorded reason.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from babylon.render.tiers import TRUECOLOR_PALETTE, RoleToken

_CRATE_SRC = Path(__file__).resolve().parents[3] / "rust" / "crates" / "babylon-client" / "src"
_PALETTE_RS = _CRATE_SRC / "palette.rs"

_CONST_RE = re.compile(
    r"pub const (?P<name>[A-Z_]+): Color = Color::srgb_u8\("
    r"(?P<r>\d+), (?P<g>\d+), (?P<b>\d+)\);"
)

#: Rust constant name -> the §9b role token it mirrors.
_ROLE_BY_CONSTANT: dict[str, RoleToken] = {
    "FIELD": RoleToken.FIELD,
    "BONE": RoleToken.TEXT,
    "CRIMSON": RoleToken.ACCENT_CRIMSON,
    "GOLD": RoleToken.ACCENT_GOLD,
    "DIM": RoleToken.MUTED_DIM,
    "MUTED_DARK": RoleToken.MUTED_DARK,
    "ROYAL": RoleToken.ROYAL,
    "GREEN_DARK": RoleToken.GREEN_DARK,
}


def _rust_constants() -> dict[str, tuple[int, int, int]]:
    """Parse ``palette.rs``'s one-line ``Color::srgb_u8`` constants."""
    source = _PALETTE_RS.read_text(encoding="utf-8")
    found = {m["name"]: (int(m["r"]), int(m["g"]), int(m["b"])) for m in _CONST_RE.finditer(source)}
    assert found, f"no Color::srgb_u8 constants parsed from {_PALETTE_RS}"
    return found


def _hex_to_rgb(value: str) -> tuple[int, int, int]:
    value = value.lstrip("#")
    return (int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16))


@pytest.mark.parametrize(("constant", "token"), sorted(_ROLE_BY_CONSTANT.items()))
def test_rust_constant_matches_truecolor_palette(constant: str, token: RoleToken) -> None:
    """Each Rust palette constant equals its §9b token's truecolor value."""
    rust = _rust_constants()
    assert constant in rust, (
        f"{constant} missing from palette.rs — keep each constant on one line "
        f"(the guard's regex contract, stated in palette.rs's module docs)"
    )
    expected = _hex_to_rgb(TRUECOLOR_PALETTE[token])
    assert rust[constant] == expected, (
        f"palette.rs {constant} = {rust[constant]} but §9b {token} = "
        f"{expected} — update the Rust side (tiers.py is the SSOT)"
    )


def test_every_rust_constant_is_mapped() -> None:
    """A new palette.rs constant must be added to the mapping here — an
    unmapped constant is an unguarded literal, the exact drift class this
    file exists to prevent."""
    unmapped = set(_rust_constants()) - set(_ROLE_BY_CONSTANT)
    assert not unmapped, f"unmapped palette.rs constants: {sorted(unmapped)}"


_COLOR_LITERAL_RE = re.compile(
    r"(?:Color::(?:srgba?(?:_u8)?|linear_rgba?|hsla?|hsva?|hex)|Srgba::new)\("
)

#: Relative (from `rust/crates/babylon-client/src/`) file paths allowed to
#: declare a raw ``Color::srgb``/``Color::srgb_u8`` call outside
#: ``palette.rs``, each with the reason a human can audit — the
#: sentinel-every-error-CLASS registry pattern this codebase already uses
#: for `EXTRA_STAMPABLE_ATTRIBUTES` (see `src/babylon/sentinels/
#: vocabulary/`). `palette.rs` itself is covered by the parity tests
#: above; this registry covers everything else, so a stray color literal
#: added to ANY other file cannot drift from §9b with nothing watching it.
_SWEEP_EXEMPTIONS: dict[str, str] = {
    "observer_theme.rs": (
        "ADR251: the Director explicitly authorized a redesigned neobrutalist "
        "observer palette on 2026-09-04. These native observer tokens are not "
        "mirrors of the frozen Python reference palette. All observer color "
        "literals remain centralized in this one theme module."
    ),
    "map/bands.rs": (
        "PANEL (#200404): the county map's absence/no-data fill. "
        "Explicitly NOT a §9b token (deliberately misses MUTED_DARK) — "
        "B1 Task 6, ADR191. Phase C's Task 9 extends this same file with "
        "the four-band diverging tension channel."
    ),
}


def test_no_stray_color_literals_outside_palette_or_a_declared_exemption() -> None:
    """Every raw color-constructor call (``srgb``/``srgba``/``_u8`` forms,
    ``linear_rgb[a]``, ``hsl[a]``/``hsv[a]``, ``hex``, ``Srgba::new``) in
    this crate's ``src/`` tree lives in ``palette.rs`` (covered by the parity tests
    above) or in a file named in ``_SWEEP_EXEMPTIONS`` with its own
    reason — closing the gap where a color literal added to some OTHER
    file would drift from §9b with nothing watching it (F4, adversarial
    verification of PR #490)."""
    offenders: list[str] = []
    for path in sorted(_CRATE_SRC.rglob("*.rs")):
        if path == _PALETTE_RS:
            continue
        rel = path.relative_to(_CRATE_SRC).as_posix()
        if rel in _SWEEP_EXEMPTIONS:
            continue
        text = path.read_text(encoding="utf-8")
        if _COLOR_LITERAL_RE.search(text):
            offenders.append(rel)
    assert not offenders, (
        f"raw Color::srgb/Color::srgb_u8 call(s) outside palette.rs with no "
        f"declared exemption: {offenders} — move into palette.rs if it is "
        "a §9b role color, or add it to _SWEEP_EXEMPTIONS with a reason if "
        "it deliberately is not"
    )

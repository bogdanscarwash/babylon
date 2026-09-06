"""Behavioral contracts for the interface SFX suite (``assets/sfx``).

The suite's rewrite test: the manifest is the spec, the renderer is a pure
function, and the committed ``.mid`` files must be byte-identical to a fresh
render of the manifest — on any machine, forever. If these tests fail, either
the manifest changed without regenerating (run ``mise run midi:generate-sfx``)
or the renderer stopped being deterministic (a bug; Constitution II).
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
SFX_DIR = REPO_ROOT / "assets" / "sfx"

EXPECTED_FAMILIES = {
    "ui": 10,
    "state": 8,
    "alert": 6,
    "stinger": 10,
    "endgame": 5,
    "entity": 6,
    "resistance": 13,
}


def _load_generator() -> ModuleType:
    spec = importlib.util.spec_from_file_location(
        "generate_sfx", REPO_ROOT / "tools" / "audio" / "sfx" / "generate_sfx.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_manifest_validates_and_covers_all_families() -> None:
    generator = _load_generator()
    manifest = generator.load_manifest(SFX_DIR / "manifest.toml")
    families: dict[str, int] = {}
    for sound in manifest.sounds:
        families[sound.family] = families.get(sound.family, 0) + 1
    assert families == EXPECTED_FAMILIES
    assert len(manifest.sounds) == sum(EXPECTED_FAMILIES.values())


def test_regeneration_is_byte_identical(tmp_path: Path) -> None:
    generator = _load_generator()
    manifest = generator.load_manifest(SFX_DIR / "manifest.toml")
    written = generator.render_suite(manifest, tmp_path)
    fresh = {path.relative_to(tmp_path) for path in written}
    committed = {path.relative_to(SFX_DIR) for path in SFX_DIR.rglob("*.mid")}
    assert fresh == committed, "committed .mid set drifted from the manifest"
    for relative in sorted(fresh):
        expected = (SFX_DIR / relative).read_bytes()
        actual = (tmp_path / relative).read_bytes()
        assert actual == expected, f"{relative} is not byte-identical on regeneration"


def test_over_budget_sound_fails_loud() -> None:
    generator = _load_generator()
    sound = generator.SfxSound(
        name="ui_too_long",
        family="ui",
        concept="a ui sound that overstays its welcome",
        trigger_hint="never",
        bpm=120,
        notes=(
            generator.NoteEvent(
                channel=0, program=8, pitch=76, start=0.0, duration=8.0, velocity=100
            ),
        ),
    )
    with pytest.raises(ValueError, match="exceeds"):
        sound.validate_sound()


def test_duplicate_sound_names_fail_loud() -> None:
    generator = _load_generator()
    note = generator.NoteEvent(
        channel=0, program=8, pitch=76, start=0.0, duration=0.1, velocity=100
    )
    sound = generator.SfxSound(
        name="ui_twin",
        family="ui",
        concept="one of two identically-named sounds",
        trigger_hint="never",
        bpm=120,
        notes=(note,),
    )
    manifest = generator.SfxManifest(schema_version=1, sounds=(sound, sound))
    with pytest.raises(ValueError, match="duplicate"):
        manifest.validate_manifest()

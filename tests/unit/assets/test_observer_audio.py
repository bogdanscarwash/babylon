"""Shipped observer audio must resolve to the original named sources and exact renders."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]
EXPECTED = {
    "assets/music/babylon_theme_phi.ogg",
    "assets/music/babylon_theme_panopticon.ogg",
    "assets/sfx/ui/ui_select.ogg",
    "assets/sfx/ui/ui_tab.ogg",
    "assets/sfx/ui/ui_open.ogg",
    "assets/sfx/ui/ui_back.ogg",
    "assets/sfx/state/tick_advance.ogg",
    "assets/sfx/state/state_fault.ogg",
}


def test_observer_audio_inventory_is_complete_and_pins_real_source_and_render_bytes() -> None:
    manifest = json.loads((ROOT / "assets/audio-renders.json").read_text())
    assert {row["output"] for row in manifest["assets"]} == EXPECTED
    assert len(manifest["assets"]) == len(EXPECTED)
    for row in manifest["assets"]:
        source = ROOT / row["source"]
        output = ROOT / row["output"]
        assert source.suffix == ".mid"
        assert source.read_bytes().startswith(b"MThd")
        assert output.read_bytes().startswith(b"OggS")
        assert hashlib.sha256(source.read_bytes()).hexdigest() == row["source_sha256"]
        assert hashlib.sha256(output.read_bytes()).hexdigest() == row["output_sha256"]
        assert row["duration_seconds"] > 0
    assert manifest["soundfont"]["license"] == "MIT"
    assert (ROOT / manifest["soundfont"]["notice"]).is_file()
    assert manifest["recipe"]["sample_rate_hz"] == 44100
    assert manifest["recipe"]["channels"] == 2
    assert manifest["recipe"]["codec"] == "vorbis"


def test_shipped_bytes_have_one_root_and_composition_code_is_tooling() -> None:
    assert (ROOT / "assets/map/county_atlas.bin").is_file()
    assert len(list((ROOT / "assets/visual").iterdir())) == 16
    assert not (ROOT / "rust/crates/babylon-client/assets/map").exists()
    assert not (ROOT / "rust/crates/babylon-client/src/visual_assets/embedded").exists()
    assert not (ROOT / "src/assets").exists()
    assert not list((ROOT / "assets").rglob("*.py"))
    assert (ROOT / "tools/audio/music/generate_music.py").is_file()
    assert (ROOT / "tools/audio/sfx/generate_sfx.py").is_file()


def test_large_audio_hook_preserves_the_general_limit_and_exact_theme_budget() -> None:
    config = yaml.safe_load((ROOT / ".pre-commit-config.yaml").read_text())
    hooks = [
        hook
        for repo in config["repos"]
        for hook in repo["hooks"]
        if hook["id"] == "check-added-large-files"
    ]
    assert len(hooks) == 2
    general = next(hook for hook in hooks if "files" not in hook)
    themes = next(hook for hook in hooks if "files" in hook)
    assert general["args"] == ["--maxkb=1024"]
    assert themes["args"] == ["--maxkb=2048", "--enforce-all"]
    expected_music = {path for path in EXPECTED if path.startswith("assets/music/")}
    assert len(expected_music) == 2
    for path in expected_music:
        assert re.search(general["exclude"], path)
        assert re.search(themes["files"], path)
        assert (ROOT / path).stat().st_size <= 2_097_152
    for path in [
        "assets/music/babylon_theme_phi.ogg.bak",
        "assets/music/babylon_theme_other.ogg",
        "assets/music/unrelated.ogg",
        "other/assets/music/babylon_theme_phi.ogg",
    ]:
        assert not re.search(general["exclude"], path)
        assert not re.search(themes["files"], path)
    assert re.search(general["exclude"], "assets/map/county_atlas.bin")
    assert not re.search(
        general["exclude"], "rust/crates/babylon-client/assets/map/county_atlas.bin"
    )

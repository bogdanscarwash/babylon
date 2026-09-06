#!/usr/bin/env python3
"""Render the existing observer themes and cues to Ogg Vorbis; never synthesize at runtime.

Run ``uv run python tools/audio/render_observer_audio.py``. ``--check`` renders
into temporary storage and verifies every committed output and provenance row.
The soundfont stays an external build input, identified by SHA-256.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

import mido  # type: ignore[import-untyped]  # mido ships no typing metadata.

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "assets" / "audio-renders.json"
SOUNDFONT = Path("/usr/share/sounds/sf2/FluidR3_GM.sf2")


@dataclass(frozen=True)
class Cue:
    path: str
    kind: str


CUES = (
    Cue("music/babylon_theme_phi", "music"),
    Cue("music/babylon_theme_panopticon", "music"),
    Cue("sfx/ui/ui_select", "sfx"),
    Cue("sfx/ui/ui_tab", "sfx"),
    Cue("sfx/ui/ui_open", "sfx"),
    Cue("sfx/ui/ui_back", "sfx"),
    Cue("sfx/state/tick_advance", "sfx"),
    Cue("sfx/state/state_fault", "sfx"),
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, check=True, text=True, capture_output=True, timeout=timeout)


def render(cue: Cue, soundfont: Path, directory: Path) -> tuple[Path, dict[str, object]]:
    source = ROOT / "assets" / f"{cue.path}.mid"
    duration = float(mido.MidiFile(source).length) + (0.5 if cue.kind == "music" else 0.15)
    if not math.isfinite(duration) or duration <= 0:
        raise ValueError(f"invalid MIDI duration: {source}")
    wav = directory / "source.wav"
    output = directory / "render.ogg"
    run(
        [
            "fluidsynth",
            "-a",
            "file",
            "-ni",
            "-q",
            "-g",
            "0.5",
            "-r",
            "44100",
            "-o",
            "synth.reverb.active=0",
            "-o",
            "synth.chorus.active=0",
            "-F",
            str(wav),
            str(soundfont),
            str(source),
        ]
    )
    trim = f"atrim=duration={duration:.9f},asetpts=PTS-STARTPTS"
    if cue.kind == "music":
        measurement = run(
            [
                "ffmpeg",
                "-nostdin",
                "-hide_banner",
                "-i",
                str(wav),
                "-af",
                f"{trim},loudnorm=I=-20:TP=-2:LRA=11:print_format=json",
                "-f",
                "null",
                "-",
            ]
        )
        match = re.search(r'\{\s*"input_i"[\s\S]*?\}', measurement.stderr)
        if match is None:
            raise ValueError("ffmpeg did not report loudness measurements")
        levels = json.loads(match.group())
        for key in ("input_i", "input_tp", "input_lra", "input_thresh", "target_offset"):
            if not math.isfinite(float(levels[key])):
                raise ValueError(f"non-finite loudness measurement {key}")
        normalization = (
            "loudnorm=I=-20:TP=-2:LRA=11:linear=true"
            f":measured_I={levels['input_i']}:measured_TP={levels['input_tp']}"
            f":measured_LRA={levels['input_lra']}:measured_thresh={levels['input_thresh']}"
            f":offset={levels['target_offset']}"
        )
    else:
        measurement = run(
            [
                "ffmpeg",
                "-nostdin",
                "-hide_banner",
                "-i",
                str(wav),
                "-af",
                f"{trim},volumedetect",
                "-f",
                "null",
                "-",
            ]
        )
        match = re.search(r"max_volume:\s*(-?\d+(?:\.\d+)?) dB", measurement.stderr)
        if match is None:
            raise ValueError("ffmpeg did not report a finite cue peak")
        normalization = f"volume={-6.0 - float(match.group(1)):.6f}dB"
    run(
        [
            "ffmpeg",
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(wav),
            "-af",
            f"{trim},{normalization}",
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "libvorbis",
            "-q:a",
            "5",
            "-fflags",
            "+bitexact",
            "-flags:a",
            "+bitexact",
            "-map_metadata",
            "-1",
            str(output),
        ]
    )
    probe = json.loads(
        run(
            [
                "ffprobe",
                "-v",
                "error",
                "-show_entries",
                "format=duration:stream=codec_name,sample_rate,channels",
                "-of",
                "json",
                str(output),
            ]
        ).stdout
    )
    stream = probe["streams"][0]
    if stream != {"codec_name": "vorbis", "sample_rate": "44100", "channels": 2}:
        raise ValueError(f"unexpected playable audio format: {stream}")
    row: dict[str, object] = {
        "source": f"assets/{cue.path}.mid",
        "source_sha256": digest(source),
        "output": f"assets/{cue.path}.ogg",
        "output_sha256": digest(output),
        "kind": cue.kind,
        "duration_seconds": float(probe["format"]["duration"]),
        "normalization": normalization,
    }
    return output, row


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--soundfont", type=Path, default=SOUNDFONT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    soundfont = args.soundfont.resolve(strict=True)
    versions = {}
    for binary in ("fluidsynth", "ffmpeg", "ffprobe"):
        executable = shutil.which(binary)
        if executable is None:
            raise FileNotFoundError(binary)
        versions[binary] = {
            "version": run(
                [binary, "--version" if binary == "fluidsynth" else "-version"], timeout=15
            ).stdout.splitlines()[0],
            "binary_sha256": digest(Path(executable).resolve()),
        }
    rows = []
    with tempfile.TemporaryDirectory(prefix="babylon-observer-audio-") as temporary:
        directory = Path(temporary)
        for cue in CUES:
            rendered, row = render(cue, soundfont, directory)
            target = ROOT / str(row["output"])
            if args.check:
                if not target.is_file() or digest(target) != row["output_sha256"]:
                    raise ValueError(f"rendered audio drift: {target}")
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(rendered, target)
            rows.append(row)
            print(
                f"{row['output']}: {row['duration_seconds']:.3f}s {row['output_sha256']}",
                flush=True,
            )
    manifest = {
        "version": 1,
        "renderer": "tools/audio/render_observer_audio.py",
        "soundfont": {
            "name": soundfont.name,
            "sha256": digest(soundfont),
            "license": "MIT",
            "notice": "assets/licenses/FluidR3-GM.txt",
        },
        "tools": versions,
        "recipe": {
            "sample_rate_hz": 44100,
            "channels": 2,
            "codec": "vorbis",
            "quality": 5,
            "synth_gain": 0.5,
            "synth_reverb": False,
            "synth_chorus": False,
            "tail_seconds": {"music": 0.5, "sfx": 0.15},
            "music_integrated_lufs": -20,
            "music_true_peak_db": -2,
            "sfx_peak_db": -6,
            "metadata": "removed",
            "ffmpeg_bitexact": True,
        },
        "license_scope": "SFX MIDI is CC0; the two legacy theme MIDI compositions retain their unresolved status in LICENSING.md. Rendering does not relicense them.",
        "assets": rows,
    }
    encoded = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if args.check:
        if MANIFEST.read_text() != encoded:
            raise ValueError("audio provenance drift: tool, source, soundfont, or recipe changed")
    else:
        MANIFEST.write_text(encoded)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

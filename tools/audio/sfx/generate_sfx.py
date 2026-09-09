#!/usr/bin/env python3
"""Deterministic renderer for the Babylon interface SFX suite.

Reads ``manifest.toml`` (the moddable source of truth for every interface
sound: notes, pitch bends, control changes) and renders one General MIDI
type-0 ``.mid`` file per sound into per-family subdirectories.

Determinism contract (Constitution II): rendering is a pure function of the
manifest — same manifest bytes, same ``.mid`` bytes, on any machine. Events
are emitted in a canonical total order (tick, event class, insertion
sequence) so regeneration is byte-identical; the unit test
``tests/unit/assets/test_sfx_assets.py`` pins this.

Usage::

    uv run python tools/audio/sfx/generate_sfx.py [--out-dir DIR]
    mise run midi:generate-sfx

:copyright: The Babylon project.
:license: CC0-1.0 (see ``assets/audio-LICENSE``) — these assets and this
    generator are dedicated to the public domain for maximal reuse.
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path
from typing import Final

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-untyped]
from pydantic import BaseModel, ConfigDict, Field

#: MIDI pulses per quarter note; matches the existing music estate.
TICKS_PER_BEAT: Final[int] = 480

#: Silent tail appended after the last event so release/reverb can breathe.
END_PAD_BEATS: Final[float] = 1.5

#: GM percussion channel (0-indexed).
PERCUSSION_CHANNEL: Final[int] = 9

#: Pitch-bend range in semitones, declared via RPN 0/0 on every channel that
#: bends. Without this the synth default (±2 semitones) silently shrinks every
#: bend gesture to a wobble — manifest bend values are authored against ±12.
BEND_RANGE_SEMITONES: Final[int] = 12

#: Hard per-family duration ceilings in seconds (interface sounds must stay
#: punctuation, not music); validated before any file is written.
FAMILY_BUDGET_SECONDS: Final[dict[str, float]] = {
    "ui": 0.8,
    "state": 3.0,
    "alert": 2.2,
    "stinger": 7.5,
    "endgame": 10.5,
    "entity": 3.0,
    "resistance": 7.5,
}

#: Control changes the suite is allowed to use (music-system.yaml vocabulary).
ALLOWED_CCS: Final[frozenset[int]] = frozenset({1, 7, 10, 11, 64, 67, 71, 73, 74, 91, 93, 94})

#: Per-channel setup emitted at tick 0 before any manifest event, so every
#: sound is self-contained on a shared synth: volume and expression at MAXIMUM
#: (loudness lives in velocity; the channel faders never eat headroom), reverb,
#: chorus off, pan centre. Manifest CCs at beat 0 land after these (insertion
#: order is the sort tiebreaker) and therefore override them.
DEFAULT_CCS: Final[tuple[tuple[int, int], ...]] = ((7, 127), (11, 127), (91, 24), (93, 0), (10, 64))

# Canonical event-class order at an identical tick: tempo and track name
# first, then channel setup, then bends, then note-offs BEFORE note-ons
# (re-strikes of the same pitch must release first).
_CLS_META: Final[int] = 0
_CLS_PROGRAM: Final[int] = 1
_CLS_CC: Final[int] = 2
_CLS_BEND: Final[int] = 3
_CLS_NOTE_OFF: Final[int] = 4
_CLS_NOTE_ON: Final[int] = 5


class NoteEvent(BaseModel):
    """One note: ``[channel, program, pitch, start_beats, duration_beats, velocity]``."""

    model_config = ConfigDict(frozen=True)

    channel: int = Field(ge=0, le=9)
    program: int = Field(ge=0, le=127)
    pitch: int = Field(ge=21, le=108)
    start: float = Field(ge=0.0)
    duration: float = Field(gt=0.0)
    velocity: int = Field(ge=1, le=127)


class BendEvent(BaseModel):
    """One pitch-wheel point: ``[channel, start_beats, value]`` (no interpolation)."""

    model_config = ConfigDict(frozen=True)

    channel: int = Field(ge=0, le=9)
    start: float = Field(ge=0.0)
    value: int = Field(ge=-8192, le=8191)


class CcEvent(BaseModel):
    """One control change: ``[channel, controller, start_beats, value]``."""

    model_config = ConfigDict(frozen=True)

    channel: int = Field(ge=0, le=9)
    controller: int
    start: float = Field(ge=0.0)
    value: int = Field(ge=0, le=127)


class SfxSound(BaseModel):
    """One interface sound: identity, intent, and its full event material."""

    model_config = ConfigDict(frozen=True)

    name: str = Field(pattern=r"^[a-z][a-z0-9_]+$")
    family: str
    concept: str = Field(min_length=1)
    trigger_hint: str = Field(min_length=1)
    bpm: int = Field(ge=40, le=220)
    notes: tuple[NoteEvent, ...] = Field(min_length=1)
    bends: tuple[BendEvent, ...] = ()
    ccs: tuple[CcEvent, ...] = ()

    def end_beats(self) -> float:
        """Last musical moment of the sound, in beats."""
        note_end = max(note.start + note.duration for note in self.notes)
        bend_end = max((bend.start for bend in self.bends), default=0.0)
        cc_end = max((cc.start for cc in self.ccs), default=0.0)
        return max(note_end, bend_end, cc_end)

    def validate_sound(self) -> None:
        """Loud, named failure for every class of manifest defect."""
        if self.family not in FAMILY_BUDGET_SECONDS:
            raise ValueError(f"{self.name}: unknown family {self.family!r}")
        budget = FAMILY_BUDGET_SECONDS[self.family]
        seconds = self.end_beats() * 60.0 / self.bpm
        if seconds > budget:
            raise ValueError(
                f"{self.name}: {seconds:.2f}s exceeds the {budget:.2f}s {self.family} budget"
            )
        for cc in self.ccs:
            if cc.controller not in ALLOWED_CCS:
                raise ValueError(f"{self.name}: CC{cc.controller} not in the sanctioned set")
        programs: dict[int, int] = {}
        for note in self.notes:
            if note.channel == PERCUSSION_CHANNEL:
                if not 27 <= note.pitch <= 87:
                    raise ValueError(f"{self.name}: drum pitch {note.pitch} outside GM map")
                continue
            known = programs.setdefault(note.channel, note.program)
            if known != note.program:
                raise ValueError(
                    f"{self.name}: channel {note.channel} mixes programs {known} and {note.program}"
                )


class SfxManifest(BaseModel):
    """The whole suite; ``sounds`` order defines render order."""

    model_config = ConfigDict(frozen=True)

    schema_version: int = Field(ge=1, le=1)
    sounds: tuple[SfxSound, ...] = Field(min_length=1)

    def validate_manifest(self) -> None:
        """Suite-level invariants: unique names, every sound valid."""
        seen: set[str] = set()
        for sound in self.sounds:
            if sound.name in seen:
                raise ValueError(f"duplicate sound name {sound.name!r}")
            seen.add(sound.name)
            sound.validate_sound()


def _row_to_note(name: str, row: list[object]) -> NoteEvent:
    if len(row) != 6:
        raise ValueError(f"{name}: note row needs 6 columns, got {len(row)}: {row!r}")
    return NoteEvent(
        channel=int(str(row[0])),
        program=int(str(row[1])),
        pitch=int(str(row[2])),
        start=float(str(row[3])),
        duration=float(str(row[4])),
        velocity=int(str(row[5])),
    )


def _row_to_bend(name: str, row: list[object]) -> BendEvent:
    if len(row) != 3:
        raise ValueError(f"{name}: bend row needs 3 columns, got {len(row)}: {row!r}")
    return BendEvent(channel=int(str(row[0])), start=float(str(row[1])), value=int(str(row[2])))


def _row_to_cc(name: str, row: list[object]) -> CcEvent:
    if len(row) != 4:
        raise ValueError(f"{name}: cc row needs 4 columns, got {len(row)}: {row!r}")
    return CcEvent(
        channel=int(str(row[0])),
        controller=int(str(row[1])),
        start=float(str(row[2])),
        value=int(str(row[3])),
    )


def load_manifest(path: Path) -> SfxManifest:
    """Parse and fully validate ``manifest.toml``; raises on any defect."""
    with path.open("rb") as handle:
        raw = tomllib.load(handle)
    sounds: list[SfxSound] = []
    for entry in raw.get("sound", []):
        name = str(entry["name"])
        sounds.append(
            SfxSound(
                name=name,
                family=str(entry["family"]),
                concept=str(entry["concept"]),
                trigger_hint=str(entry["trigger_hint"]),
                bpm=int(entry["bpm"]),
                notes=tuple(_row_to_note(name, row) for row in entry["notes"]),
                bends=tuple(_row_to_bend(name, row) for row in entry.get("bends", [])),
                ccs=tuple(_row_to_cc(name, row) for row in entry.get("ccs", [])),
            )
        )
    manifest = SfxManifest(schema_version=int(raw["schema_version"]), sounds=tuple(sounds))
    manifest.validate_manifest()
    return manifest


def _beats_to_ticks(beats: float) -> int:
    return round(beats * TICKS_PER_BEAT)


def _setup_events(sound: SfxSound) -> list[tuple[int, int, Message]]:
    """Per-channel initialisation at tick 0: program, bend reset, default CCs."""
    events: list[tuple[int, int, Message]] = []
    channels_used = sorted({note.channel for note in sound.notes})
    melodic = [channel for channel in channels_used if channel != PERCUSSION_CHANNEL]
    percussive = PERCUSSION_CHANNEL in channels_used
    programs = {
        note.channel: note.program for note in sound.notes if note.channel != PERCUSSION_CHANNEL
    }
    for channel in melodic:
        events.append(
            (
                0,
                _CLS_PROGRAM,
                Message("program_change", channel=channel, program=programs[channel], time=0),
            )
        )
        events.append((0, _CLS_BEND, Message("pitchwheel", channel=channel, pitch=0, time=0)))
    rpn_bend_range: tuple[tuple[int, int], ...] = (
        (101, 0),
        (100, 0),
        (6, BEND_RANGE_SEMITONES),
        (38, 0),
        (101, 127),
        (100, 127),
    )
    for channel in sorted({bend.channel for bend in sound.bends}):
        for controller, value in rpn_bend_range:
            events.append(
                (
                    0,
                    _CLS_CC,
                    Message(
                        "control_change", channel=channel, control=controller, value=value, time=0
                    ),
                )
            )
    channels = list(melodic) + ([PERCUSSION_CHANNEL] if percussive else [])
    for channel in channels:
        for controller, value in DEFAULT_CCS:
            events.append(
                (
                    0,
                    _CLS_CC,
                    Message(
                        "control_change", channel=channel, control=controller, value=value, time=0
                    ),
                )
            )
    return events


def render_sound(sound: SfxSound) -> MidiFile:
    """Render one sound to an in-memory type-0 :class:`mido.MidiFile`."""
    timed: list[tuple[int, int, Message | MetaMessage]] = []
    tempo = round(60_000_000 / sound.bpm)
    timed.append((0, _CLS_META, MetaMessage("track_name", name=sound.name, time=0)))
    timed.append((0, _CLS_META, MetaMessage("set_tempo", tempo=tempo, time=0)))
    timed.extend(_setup_events(sound))
    for cc in sound.ccs:
        timed.append(
            (
                _beats_to_ticks(cc.start),
                _CLS_CC,
                Message(
                    "control_change",
                    channel=cc.channel,
                    control=cc.controller,
                    value=cc.value,
                    time=0,
                ),
            )
        )
    for bend in sound.bends:
        timed.append(
            (
                _beats_to_ticks(bend.start),
                _CLS_BEND,
                Message("pitchwheel", channel=bend.channel, pitch=bend.value, time=0),
            )
        )
    for note in sound.notes:
        on_tick = _beats_to_ticks(note.start)
        off_tick = max(on_tick + 1, _beats_to_ticks(note.start + note.duration))
        timed.append(
            (
                on_tick,
                _CLS_NOTE_ON,
                Message(
                    "note_on", channel=note.channel, note=note.pitch, velocity=note.velocity, time=0
                ),
            )
        )
        timed.append(
            (
                off_tick,
                _CLS_NOTE_OFF,
                Message("note_off", channel=note.channel, note=note.pitch, velocity=0, time=0),
            )
        )
    last_tick = max(item[0] for item in timed)
    for channel in sorted({bend.channel for bend in sound.bends}):
        timed.append(
            (last_tick, _CLS_BEND, Message("pitchwheel", channel=channel, pitch=0, time=0))
        )
    end_tick = last_tick + _beats_to_ticks(END_PAD_BEATS)
    timed.append((end_tick, _CLS_NOTE_OFF, MetaMessage("end_of_track", time=0)))

    ordered = sorted(enumerate(timed), key=lambda item: (item[1][0], item[1][1], item[0]))
    track = MidiTrack()
    cursor = 0
    for entry in ordered:
        tick, message = entry[1][0], entry[1][2]
        track.append(message.copy(time=tick - cursor))
        cursor = tick
    midi = MidiFile(type=0, ticks_per_beat=TICKS_PER_BEAT)
    midi.tracks.append(track)
    return midi


def render_suite(manifest: SfxManifest, out_dir: Path) -> list[Path]:
    """Render every sound to ``out_dir/<family>/<name>.mid``; returns paths."""
    written: list[Path] = []
    for sound in manifest.sounds:
        target = out_dir / sound.family / f"{sound.name}.mid"
        target.parent.mkdir(parents=True, exist_ok=True)
        render_sound(sound).save(str(target))
        written.append(target)
    return written


def main(argv: list[str] | None = None) -> int:
    """CLI entry point; renders the suite next to this script by default."""
    here = Path(__file__).resolve().parents[3] / "assets" / "sfx"
    parser = argparse.ArgumentParser(description="Render the Babylon interface SFX suite.")
    parser.add_argument("--manifest", type=Path, default=here / "manifest.toml")
    parser.add_argument("--out-dir", type=Path, default=here)
    args = parser.parse_args(argv)
    manifest = load_manifest(args.manifest)
    written = render_suite(manifest, args.out_dir)
    for path in written:
        size = path.stat().st_size
        print(f"  {path.relative_to(args.out_dir)}  ({size} bytes)")
    print(f"Rendered {len(written)} sounds from {args.manifest.name}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

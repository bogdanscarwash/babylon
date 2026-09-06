#!/usr/bin/env python3
"""Deterministic composition kit for the Babylon soundtrack estate.

A :class:`Score` collects notes, control changes, pitch bends, tempo changes
and section markers in beats, then renders a General MIDI type-0 file in a
canonical total event order — same composition code, same bytes, on any
machine (Constitution II). Track modules in ``tracks/`` build a Score in a
pure ``compose()`` function; ``generate_music.py`` renders the suite and
``tests/unit/assets/test_music_assets.py`` pins byte-identity.

Deliberately self-contained rather than importing ``sfx/generate_sfx.py``:
the standalone-script doctrine in ``ai/epochs/epoch3/music-system.yaml``
(lessons_learned.relative_imports) applies — the ~40 shared writer lines are
duplicated on purpose, with the canonical ordering kept identical.

:license: CC0-1.0 (see ``assets/audio-LICENSE``).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Final

from mido import Message, MetaMessage, MidiFile, MidiTrack  # type: ignore[import-untyped]

TICKS_PER_BEAT: Final[int] = 480
END_PAD_BEATS: Final[float] = 2.0
PERCUSSION_CHANNEL: Final[int] = 9
BEND_RANGE_SEMITONES: Final[int] = 12

#: Sanctioned controllers (music-system.yaml vocabulary).
ALLOWED_CCS: Final[frozenset[int]] = frozenset({1, 7, 10, 11, 64, 67, 71, 73, 74, 91, 93, 94})

#: Channel init: full volume/expression (dynamics live in velocity and CC11
#: phrasing), modest reverb, chorus off, centre pan — every track re-asserts
#: its own state so files are safe on a shared synth.
DEFAULT_CCS: Final[tuple[tuple[int, int], ...]] = ((7, 127), (11, 127), (91, 40), (93, 0), (10, 64))

_CLS_META: Final[int] = 0
_CLS_PROGRAM: Final[int] = 1
_CLS_CC: Final[int] = 2
_CLS_BEND: Final[int] = 3
_CLS_NOTE_OFF: Final[int] = 4
_CLS_NOTE_ON: Final[int] = 5

#: Ceiling on programmatic event generation per track — a loop-bound backstop
#: (Power-of-10 rule 2), far above any real composition (~6k events = dense).
MAX_EVENTS_PER_TRACK: Final[int] = 20_000


def beats_to_ticks(beats: float) -> int:
    """Quantize a beat position to the 480-ppq grid."""
    return round(beats * TICKS_PER_BEAT)


def bpm_to_tempo(bpm: float) -> int:
    """Microseconds per quarter note, rounded deterministically."""
    return round(60_000_000 / bpm)


@dataclass(frozen=True)
class _NoteEvent:
    channel: int
    pitch: int
    start: float
    duration: float
    velocity: int


@dataclass(frozen=True)
class _CcEvent:
    channel: int
    controller: int
    start: float
    value: int


@dataclass(frozen=True)
class _BendEvent:
    channel: int
    start: float
    value: int


@dataclass(frozen=True)
class _TempoEvent:
    start: float
    bpm: float


@dataclass(frozen=True)
class _MarkerEvent:
    start: float
    text: str


@dataclass
class Score:
    """One track under construction; all times in beats at the current tempo map."""

    name: str
    suite: str
    concept: str
    bpm: float
    time_signature: tuple[int, int] = (4, 4)
    _programs: dict[int, int] = field(default_factory=dict)
    _notes: list[_NoteEvent] = field(default_factory=list)
    _ccs: list[_CcEvent] = field(default_factory=list)
    _bends: list[_BendEvent] = field(default_factory=list)
    _tempos: list[_TempoEvent] = field(default_factory=list)
    _markers: list[_MarkerEvent] = field(default_factory=list)

    # ------------------------------------------------------------------ voice
    def program(self, channel: int, program: int) -> None:
        """Assign a GM program to a channel (one program per channel per track)."""
        if not 0 <= channel <= 15:
            raise ValueError(f"{self.name}: channel {channel} out of range")
        if channel == PERCUSSION_CHANNEL:
            raise ValueError(f"{self.name}: percussion channel takes no program")
        if not 0 <= program <= 127:
            raise ValueError(f"{self.name}: program {program} out of range")
        known = self._programs.setdefault(channel, program)
        if known != program:
            raise ValueError(f"{self.name}: channel {channel} reassigned {known}->{program}")

    def note(self, channel: int, pitch: int, start: float, duration: float, velocity: int) -> None:
        """Add one note; percussion goes to channel 9 with drum-map pitches."""
        if not 0 <= channel <= 15:
            raise ValueError(f"{self.name}: channel {channel} out of range")
        low, high = (27, 87) if channel == PERCUSSION_CHANNEL else (21, 108)
        if not low <= pitch <= high:
            raise ValueError(f"{self.name}: pitch {pitch} out of range on channel {channel}")
        if duration <= 0 or start < 0:
            raise ValueError(f"{self.name}: bad timing start={start} duration={duration}")
        if not 1 <= velocity <= 127:
            raise ValueError(f"{self.name}: velocity {velocity} out of range")
        if channel != PERCUSSION_CHANNEL and channel not in self._programs:
            raise ValueError(f"{self.name}: channel {channel} used before program()")
        self._notes.append(_NoteEvent(channel, pitch, start, duration, velocity))
        self._check_budget()

    def chord(
        self,
        channel: int,
        pitches: tuple[int, ...],
        start: float,
        duration: float,
        velocity: int,
        spread: float = 0.0,
    ) -> None:
        """Add a chord; ``spread`` staggers onsets (arpeggiation/strum), bounded."""
        for index, pitch in enumerate(pitches):
            self.note(channel, pitch, start + spread * index, duration, velocity)

    # ------------------------------------------------------------ controllers
    def cc(self, channel: int, controller: int, start: float, value: int) -> None:
        """One control change; controller must be in the sanctioned set."""
        if controller not in ALLOWED_CCS:
            raise ValueError(f"{self.name}: CC{controller} not sanctioned")
        if not 0 <= value <= 127:
            raise ValueError(f"{self.name}: CC value {value} out of range")
        self._ccs.append(_CcEvent(channel, controller, start, value))
        self._check_budget()

    def cc_ramp(
        self,
        channel: int,
        controller: int,
        start: float,
        end: float,
        from_value: int,
        to_value: int,
        steps: int = 16,
    ) -> None:
        """Linear controller ramp as ``steps`` discrete points (crescendo etc.)."""
        if steps < 2 or steps > 128:
            raise ValueError(f"{self.name}: ramp steps {steps} out of [2,128]")
        if end <= start:
            raise ValueError(f"{self.name}: ramp end {end} <= start {start}")
        for index in range(steps):
            frac = index / (steps - 1)
            beat = start + (end - start) * frac
            value = round(from_value + (to_value - from_value) * frac)
            self.cc(channel, controller, beat, value)

    def bend(self, channel: int, start: float, value: int) -> None:
        """Pitch-wheel point (±8192 = ±12 semitones; RPN emitted at render)."""
        if not -8192 <= value <= 8191:
            raise ValueError(f"{self.name}: bend {value} out of range")
        self._bends.append(_BendEvent(channel, start, value))
        self._check_budget()

    def tempo(self, start: float, bpm: float) -> None:
        """Tempo change at a beat (accelerando = several of these)."""
        if not 20.0 <= bpm <= 300.0:
            raise ValueError(f"{self.name}: tempo {bpm} out of range")
        self._tempos.append(_TempoEvent(start, bpm))

    def marker(self, start: float, text: str) -> None:
        """Section marker (meta event) — Crisis Onset / Bifurcation Point / ..."""
        self._markers.append(_MarkerEvent(start, text))

    # ---------------------------------------------------------------- queries
    def end_beats(self) -> float:
        """Last musical moment in beats."""
        note_end = max((n.start + n.duration for n in self._notes), default=0.0)
        cc_end = max((c.start for c in self._ccs), default=0.0)
        bend_end = max((b.start for b in self._bends), default=0.0)
        return max(note_end, cc_end, bend_end)

    def duration_seconds(self) -> float:
        """Wall-clock length under the tempo map (piecewise-constant bpm)."""
        points = sorted([(0.0, self.bpm)] + [(t.start, t.bpm) for t in self._tempos])
        end = self.end_beats()
        seconds = 0.0
        for index, (beat, bpm) in enumerate(points):
            next_beat = points[index + 1][0] if index + 1 < len(points) else end
            span = max(0.0, min(next_beat, end) - beat)
            seconds += span * 60.0 / bpm
        return seconds

    def _check_budget(self) -> None:
        total = len(self._notes) + len(self._ccs) + len(self._bends)
        if total > MAX_EVENTS_PER_TRACK:
            raise ValueError(f"{self.name}: event budget {MAX_EVENTS_PER_TRACK} exceeded")

    def _check_note_overlaps(self) -> None:
        """Same-channel same-pitch overlap sentinel.

        A note_on landing inside an earlier note's span on the same (channel,
        pitch) is SILENTLY fatal downstream: the earlier note's note_off
        releases the synth voice and truncates the newer note (this killed
        three of four drone breaths in history_breathing before this check
        existed). Byte-identity tests cannot catch it — the bytes are
        faithfully wrong — so the kit refuses to render it.
        """
        spans: dict[tuple[int, int], list[tuple[float, float]]] = {}
        for note_event in self._notes:
            key = (note_event.channel, note_event.pitch)
            spans.setdefault(key, []).append(
                (note_event.start, note_event.start + note_event.duration)
            )
        for (channel, pitch), intervals in sorted(spans.items()):
            intervals.sort()
            for index in range(len(intervals) - 1):
                if intervals[index + 1][0] < intervals[index][1] - 1e-9:
                    raise ValueError(
                        f"{self.name}: overlapping notes on channel {channel} pitch {pitch} "
                        f"at beat {intervals[index + 1][0]} (previous ends "
                        f"{intervals[index][1]}) — the note_off would truncate the newer note"
                    )

    # ----------------------------------------------------------------- render
    def render(self) -> MidiFile:
        """Render to a type-0 MidiFile in canonical order (byte-deterministic)."""
        if not self._notes:
            raise ValueError(f"{self.name}: empty score")
        self._check_note_overlaps()
        timed: list[tuple[int, int, Message | MetaMessage]] = []
        timed.append((0, _CLS_META, MetaMessage("track_name", name=self.name, time=0)))
        numerator, denominator = self.time_signature
        timed.append(
            (
                0,
                _CLS_META,
                MetaMessage("time_signature", numerator=numerator, denominator=denominator, time=0),
            )
        )
        timed.append((0, _CLS_META, MetaMessage("set_tempo", tempo=bpm_to_tempo(self.bpm), time=0)))
        for tempo_event in self._tempos:
            timed.append(
                (
                    beats_to_ticks(tempo_event.start),
                    _CLS_META,
                    MetaMessage("set_tempo", tempo=bpm_to_tempo(tempo_event.bpm), time=0),
                )
            )
        for marker_event in self._markers:
            timed.append(
                (
                    beats_to_ticks(marker_event.start),
                    _CLS_META,
                    MetaMessage("marker", text=marker_event.text, time=0),
                )
            )
        for channel in sorted(self._programs):
            timed.append(
                (
                    0,
                    _CLS_PROGRAM,
                    Message(
                        "program_change", channel=channel, program=self._programs[channel], time=0
                    ),
                )
            )
            timed.append((0, _CLS_BEND, Message("pitchwheel", channel=channel, pitch=0, time=0)))
        bend_channels = sorted({b.channel for b in self._bends})
        rpn: tuple[tuple[int, int], ...] = (
            (101, 0),
            (100, 0),
            (6, BEND_RANGE_SEMITONES),
            (38, 0),
            (101, 127),
            (100, 127),
        )
        for channel in bend_channels:
            for controller, value in rpn:
                timed.append(
                    (
                        0,
                        _CLS_CC,
                        Message(
                            "control_change",
                            channel=channel,
                            control=controller,
                            value=value,
                            time=0,
                        ),
                    )
                )
        used = sorted({n.channel for n in self._notes})
        for channel in used:
            for controller, value in DEFAULT_CCS:
                timed.append(
                    (
                        0,
                        _CLS_CC,
                        Message(
                            "control_change",
                            channel=channel,
                            control=controller,
                            value=value,
                            time=0,
                        ),
                    )
                )
        for cc_event in self._ccs:
            timed.append(
                (
                    beats_to_ticks(cc_event.start),
                    _CLS_CC,
                    Message(
                        "control_change",
                        channel=cc_event.channel,
                        control=cc_event.controller,
                        value=cc_event.value,
                        time=0,
                    ),
                )
            )
        for bend_event in self._bends:
            timed.append(
                (
                    beats_to_ticks(bend_event.start),
                    _CLS_BEND,
                    Message(
                        "pitchwheel", channel=bend_event.channel, pitch=bend_event.value, time=0
                    ),
                )
            )
        for note_event in self._notes:
            on_tick = beats_to_ticks(note_event.start)
            off_tick = max(on_tick + 1, beats_to_ticks(note_event.start + note_event.duration))
            timed.append(
                (
                    on_tick,
                    _CLS_NOTE_ON,
                    Message(
                        "note_on",
                        channel=note_event.channel,
                        note=note_event.pitch,
                        velocity=note_event.velocity,
                        time=0,
                    ),
                )
            )
            timed.append(
                (
                    off_tick,
                    _CLS_NOTE_OFF,
                    Message(
                        "note_off",
                        channel=note_event.channel,
                        note=note_event.pitch,
                        velocity=0,
                        time=0,
                    ),
                )
            )
        last_tick = max(item[0] for item in timed)
        for channel in bend_channels:
            timed.append(
                (last_tick, _CLS_BEND, Message("pitchwheel", channel=channel, pitch=0, time=0))
            )
        timed.append(
            (
                last_tick + beats_to_ticks(END_PAD_BEATS),
                _CLS_NOTE_OFF,
                MetaMessage("end_of_track", time=0),
            )
        )
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

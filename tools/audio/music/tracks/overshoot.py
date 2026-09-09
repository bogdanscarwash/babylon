#!/usr/bin/env python3
"""overshoot — O = C/B > 1 (rift suite).

CONCEPTUAL BRIEF:
The metabolic rift as rhythm. Regeneration is a gentle 9-beat cycle sung by
three biocapacity voices — strings, harp, choir. Consumption is a driving
7-beat piano figure with a kick under it. Seven against nine: they phase,
almost together, then apart — sustainable tension. Then consumption
ACCELERATES: its period contracts to six beats, then five, while
regeneration CANNOT speed up (that is the whole tragedy of ecology). At
every lap — each time consumption overtakes a regeneration cycle — a
timpani strike lands and ONE BIOCAPACITY VOICE FALLS SILENT: first the
choir, then the harp, then the strings thin, then nothing regrows at all.
The consumption figure hammers on alone, loses notes cycle by cycle —
a machine stuttering on an empty tank — and stops on two isolated strikes.
Ecological overshoot does not end with a bang; it ends with the last
withdrawal from an account nothing refills.

TECHNICAL: 100 BPM, 4/4, ~240 beats ≈ 2:24. Piano 0 + kick (consumption);
Strings 48, Harp 46, Choir 52 (regeneration); Timpani 47 (the laps).
"""

from __future__ import annotations

from tools.audio.music.composer import Score

_CONSUMPTION: tuple[int, ...] = (
    64,
    59,
    65,
    67,
    64,
    59,
)  # E4 B3 F4 G4 E4 B3 — the flat-2 grain rides the motor
_REGROWTH: tuple[tuple[float, int], ...] = ((0.0, 52), (2.5, 55), (5.0, 59), (7.0, 55))

#: (start_beat, period) segments for the consumption cycle — the acceleration.
_PHASES: tuple[tuple[float, float, int], ...] = ((0.0, 7.0, 13), (91.0, 6.0, 10), (151.0, 5.0, 12))

#: Lap beats — each is a REAL consumption-cycle head from _PHASES (63 = the
#: 7x9 coincidence; 127 = 91+6*6; 161 = 151+5*2; 191 = 151+5*8; 206 = the
#: final head) — and the biocapacity voice that dies at it.
_LAPS: tuple[tuple[float, str], ...] = (
    (63.0, "choir"),
    (127.0, "harp"),
    (161.0, "strings_thin"),
    (191.0, "strings_out"),
    (206.0, "the_last"),
)


def _consumption_figure(score: Score, base: float, notes: int, velocity: int) -> None:
    for index in range(notes):
        score.note(0, _CONSUMPTION[index], base + 0.5 * index, 0.42, velocity)
    score.note(9, 36, base, 0.2, max(40, velocity - 10))


def compose() -> Score:
    score = Score(
        name="overshoot",
        suite="rift",
        concept="A 7-beat consumption figure phases against a 9-beat regeneration cycle, "
        "then accelerates to 6 and 5 while regeneration cannot; each lap kills one "
        "biocapacity voice until consumption stutters alone and stops.",
        bpm=100,
    )
    score.program(0, 0)  # piano — consumption
    score.program(1, 48)  # strings — biocapacity
    score.program(2, 46)  # harp — biocapacity
    score.program(3, 52)  # choir — biocapacity
    score.program(4, 47)  # timpani — the laps

    score.cc(0, 91, 0.0, 30)
    score.cc(1, 91, 0.0, 62)
    score.cc(1, 93, 0.0, 55)
    score.cc(2, 91, 0.0, 66)
    score.cc(3, 91, 0.0, 74)
    score.cc(3, 93, 0.0, 60)
    score.cc(4, 91, 0.0, 44)
    score.cc(0, 10, 0.0, 46)
    score.cc(2, 10, 0.0, 84)
    score.cc(3, 10, 0.0, 60)

    score.marker(0.0, "Seven Against Nine")
    score.marker(91.0, "Acceleration")
    score.marker(126.0, "Drawdown")
    score.marker(192.0, "Nothing Regrows")
    score.marker(212.0, "The Last Withdrawal")

    # Consumption: three phases of contracting period.
    for phase_start, period, cycles in _PHASES:
        for cycle in range(cycles):
            base = phase_start + period * cycle
            _consumption_figure(score, base, notes=6, velocity=88)

    # Regeneration: the 9-beat cycle, per voice, until its lap silences it.
    for cycle in range(24):  # 24 * 9 = 216 — past the last lap; gating trims it
        base = 9.0 * cycle
        if base < 191.0:  # strings live until their lap
            thin = base >= 161.0
            for beat, pitch in _REGROWTH[: 2 if thin else 4]:
                score.note(1, pitch, base + beat, 2.2, 66 if not thin else 54)
        if base < 126.0:  # harp sings two octaves up — clear of the motor
            for beat, pitch in _REGROWTH:
                score.note(2, pitch + 24, base + beat + 0.15, 1.6, 58)
        if base < 63.0:  # choir breathes low, under the piano's register
            score.chord(3, (55, 59, 64), base, 8.0, 46, spread=0.3)

    # The laps: a timpani strike as each account closes.
    for index, (beat, _casualty) in enumerate(_LAPS):
        score.note(4, 40, beat, 0.6, 96 + 4 * index)

    # After the last regrowth: the figure loses notes — 6, then 4, then 2.
    for cycle, notes in ((0, 6), (1, 4), (2, 4), (3, 2)):
        _consumption_figure(score, 216.0 + 5.0 * cycle, notes=notes, velocity=92)
    # Two isolated strikes on an empty tank.
    score.note(0, 64, 237.0, 0.4, 96)
    score.note(0, 64, 239.5, 0.4, 90)
    return score

#!/usr/bin/env python3
"""the_mask — the only consonant surface in the game (entity suite).

CONCEPTUAL BRIEF (#641, Director-ruled Arm A, 2026-08-18: consonance as the
lie, CANON):
The Vol III ticker/dashboard view gets thin, sanitized, pleasant material —
elevator-clean, genuinely in tune, cadences arriving on schedule — precisely
BECAUSE the mask lies. This is the dissonance directive operating at the meta
level: in a game where every other surface is structurally wrong, the one
clean harmony is the deception. Toggling from the ticker to the topology is
an audible lifting of the veil. Nothing here is out of tune, and every
dissonance is functional — the G7's tritone resolves on schedule, every time.
Heard alone it is merely nice, which is the point.

It is rooted away from the estate's home — C major, whose mediant happens to
be the beast's E: even the mask cannot help containing it — and its favourite
note is the F the dawn deletes. The progression is the most affirmative
cliché in the harmonic language (I–vi7–IV–V7, resolving every sixteen beats,
forever), voiced thin and struck softly with brushes: money describing itself
to itself.

Cue binding (non-event): the Vol III ticker / dashboard lens — see
assets/CUE_MAP.md. Bed channel 8's glint in beast_engine is this piece's
only intrusion into the beast's own body.

TECHNICAL: 72 BPM, 4/4, 160 beats = 2:13. THE LOOP POINT IS 160.0, declared
by marker: the tenth cycle's G7 resolves directly into the loop's own head —
the cadence lands on the seam itself, so the lie resolves on schedule forever
and nothing overhangs it (no tail note crosses the seam; the brushes run to
the final beat). Vibraphone 11, Electric Piano 4, brush percussion.
"""

from __future__ import annotations

from tools.audio.music.composer import Score

#: I - vi7 - IV - V7 in C major: the affirmative cliché, four beats a chord.
_CHORDS: tuple[tuple[int, ...], ...] = (
    (60, 64, 67),  # C
    (57, 60, 64, 67),  # Am7
    (53, 57, 60, 65),  # F — the surplus note, at home here and only here
    (55, 59, 62, 65),  # G7
)
#: The vibraphone's six-note comping figure per bar, indexed into the chord
#: (with the octave-up first tone as the sparkle).
_ARP_STEPS: tuple[int, ...] = (0, 1, 2, 1, 0, 2)

_CYCLES: int = 10  # 10 × 16 beats = 160; the loop length is the cadence cycle


def compose() -> Score:
    score = Score(
        name="the_mask",
        suite="entity",
        concept="The ticker's elevator harmony: I-vi7-IV-V7 in C major, in "
        "tune, on time, the V7 resolving into the loop's own head forever. "
        "The only clean surface in the game, because the mask lies. "
        "(Director-ruled Arm A: the exception is the deception.)",
        bpm=72,
    )
    score.program(0, 11)  # vibraphone — the sparkle
    score.program(1, 4)  # electric piano — the surface

    score.cc(0, 91, 0.0, 42)
    score.cc(1, 91, 0.0, 38)
    score.cc(0, 10, 0.0, 58)
    score.cc(1, 10, 0.0, 68)

    score.marker(0.0, "Opening Bell")
    score.marker(80.0, "The Numbers Are Good")
    score.marker(160.0, "LOOP")

    # Ten cycles of the sixteen-beat progression: piano states each chord as
    # a soft whole-note voicing; the vibraphone comps a six-note figure over
    # it. Every voice in tune, every onset on the grid, every velocity gentle.
    # The last G7 releases at 159.8 — nothing sounds across the seam.
    for cycle in range(_CYCLES):
        for bar, chord in enumerate(_CHORDS):
            base = 16.0 * cycle + 4.0 * bar
            for voice, pitch in enumerate(chord):
                score.note(1, pitch, base, 3.8, 46 - 2 * voice)
            for step, degree in enumerate(_ARP_STEPS):
                pitch = chord[degree] + 12
                score.note(0, pitch, base + 0.5 * step, 0.45, 48 + 4 * (step % 2))

    # Brushes: closed hat on every beat, rim on two and four, all whisper-soft
    # — the pulse of a lobby, not an engine, and it runs to the seam.
    for beat in range(160):
        score.note(9, 42, float(beat), 0.05, 24)
        if beat % 4 in (1, 3):
            score.note(9, 37, float(beat), 0.05, 30)
    return score

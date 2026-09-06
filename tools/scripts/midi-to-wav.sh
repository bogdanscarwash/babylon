#!/bin/bash
# Convert MIDI to WAV using FluidSynth
# Usage: midi-to-wav.sh <input.mid> [output.wav]

set -e

SOUNDFONT="${SOUNDFONT:-/usr/share/sounds/sf2/FluidR3_GM.sf2}"

INPUT="$1"
if [ -z "$INPUT" ]; then
    echo "Usage: midi-to-wav <input.mid> [output.wav]"
    exit 1
fi

OUTPUT="${2:-${INPUT%.mid}.wav}"
mkdir -p "$(dirname "$OUTPUT")"

echo "Converting: $INPUT -> $OUTPUT"
# -g 1.0 = gain boost (default 0.2 is too quiet)
fluidsynth -ni -g 1.0 "$SOUNDFONT" "$INPUT" -F "$OUTPUT" -r 44100
echo "Done: $OUTPUT"

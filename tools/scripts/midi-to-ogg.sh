#!/bin/bash
# Convert MIDI to OGG Vorbis using FluidSynth + ffmpeg
# Usage: midi-to-ogg.sh <input.mid> [output.ogg]

set -e

SOUNDFONT="${SOUNDFONT:-/usr/share/sounds/sf2/FluidR3_GM.sf2}"

INPUT="$1"
if [ -z "$INPUT" ]; then
    echo "Usage: midi-to-ogg <input.mid> [output.ogg]"
    exit 1
fi

if ! command -v ffmpeg &> /dev/null; then
    echo "Error: ffmpeg is required for OGG conversion"
    exit 1
fi

OUTPUT="${2:-${INPUT%.mid}.ogg}"
mkdir -p "$(dirname "$OUTPUT")"

TEMP_WAV=$(mktemp --suffix=.wav)
trap 'rm -f "$TEMP_WAV"' EXIT

echo "Converting: $INPUT -> WAV (temp)"
# -g 1.0 = gain boost (default 0.2 is too quiet)
fluidsynth -ni -g 1.0 "$SOUNDFONT" "$INPUT" -F "$TEMP_WAV" -r 44100

echo "Converting: WAV -> $OUTPUT"
ffmpeg -y -i "$TEMP_WAV" -acodec libvorbis -aq 6 "$OUTPUT" 2>/dev/null

echo "Done: $OUTPUT"

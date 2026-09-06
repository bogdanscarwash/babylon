#!/bin/bash
# Convert MIDI to MP3 using FluidSynth + ffmpeg
# Usage: midi-to-mp3.sh <input.mid> [output.mp3]

set -e

SOUNDFONT="${SOUNDFONT:-/usr/share/sounds/sf2/FluidR3_GM.sf2}"

INPUT="$1"
if [ -z "$INPUT" ]; then
    echo "Usage: midi-to-mp3 <input.mid> [output.mp3]"
    exit 1
fi

# Check for ffmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "Error: ffmpeg is required for MP3 conversion"
    echo "Install with: sudo apt install ffmpeg"
    exit 1
fi

OUTPUT="${2:-${INPUT%.mid}.mp3}"
mkdir -p "$(dirname "$OUTPUT")"

TEMP_WAV=$(mktemp --suffix=.wav)
trap 'rm -f "$TEMP_WAV"' EXIT

echo "Converting: $INPUT -> WAV (temp)"
# -g 1.0 = gain boost (default 0.2 is too quiet)
fluidsynth -ni -g 1.0 "$SOUNDFONT" "$INPUT" -F "$TEMP_WAV" -r 44100

echo "Converting: WAV -> $OUTPUT"
ffmpeg -y -i "$TEMP_WAV" -acodec libmp3lame -ab 192k "$OUTPUT" 2>/dev/null

echo "Done: $OUTPUT"

#!/bin/bash
# Convert all MIDI files in music directory to a specified format
# Usage: midi-convert-all.sh [wav|mp3|ogg]

set -e

# Get the script's directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SOUNDFONT="${SOUNDFONT:-/usr/share/sounds/sf2/FluidR3_GM.sf2}"
MUSIC_DIR="$PROJECT_ROOT/assets/music"
AUDIO_OUTPUT_DIR="$MUSIC_DIR"

FORMAT="${1:-wav}"

case "$FORMAT" in
    wav|mp3|ogg)
        ;;
    *)
        echo "Usage: midi-convert-all [wav|mp3|ogg]"
        echo "Default: wav"
        exit 1
        ;;
esac

if [ "$FORMAT" != "wav" ] && ! command -v ffmpeg &> /dev/null; then
    echo "Error: ffmpeg is required for $FORMAT conversion"
    exit 1
fi

mkdir -p "$AUDIO_OUTPUT_DIR"

echo "Converting all MIDI files to $FORMAT..."
echo "Soundfont: $SOUNDFONT"
echo "Music dir: $MUSIC_DIR"
echo "Output dir: $AUDIO_OUTPUT_DIR"
echo ""

TEMP_WAV=$(mktemp --suffix=.wav)
trap 'rm -f "$TEMP_WAV"' EXIT

# Store file list in array to avoid subshell issues
mapfile -t MIDI_FILES < <(find "$MUSIC_DIR" -name "*.mid" -type f | sort)

COUNT=0
for midi in "${MIDI_FILES[@]}"; do
    # Get path relative to music dir
    RELPATH="${midi#"$MUSIC_DIR/"}"
    DIRNAME=$(dirname "$RELPATH")
    BASENAME=$(basename "$midi" .mid)

    # Create output directory structure
    if [ "$DIRNAME" = "." ]; then
        OUTPUT_SUBDIR="$AUDIO_OUTPUT_DIR"
    else
        OUTPUT_SUBDIR="$AUDIO_OUTPUT_DIR/$DIRNAME"
    fi
    mkdir -p "$OUTPUT_SUBDIR"
    OUTPUT="$OUTPUT_SUBDIR/$BASENAME.$FORMAT"

    echo "  $RELPATH -> $OUTPUT"

    # -g 2.0 = gain boost (default 0.2 is too quiet, max ~5.0)
    # loudnorm normalizes to -14 LUFS (streaming standard) with -1.5dB true peak ceiling
    case "$FORMAT" in
        wav)
            fluidsynth -ni -g 2.0 "$SOUNDFONT" "$midi" -F "$TEMP_WAV" -r 44100
            ffmpeg -y -i "$TEMP_WAV" -af loudnorm=I=-14:TP=-1.5:LRA=11 "$OUTPUT" 2>/dev/null
            ;;
        mp3)
            fluidsynth -ni -g 2.0 "$SOUNDFONT" "$midi" -F "$TEMP_WAV" -r 44100
            ffmpeg -y -i "$TEMP_WAV" -af loudnorm=I=-14:TP=-1.5:LRA=11 -acodec libmp3lame -ab 192k "$OUTPUT" 2>/dev/null
            ;;
        ogg)
            fluidsynth -ni -g 2.0 "$SOUNDFONT" "$midi" -F "$TEMP_WAV" -r 44100
            ffmpeg -y -i "$TEMP_WAV" -af loudnorm=I=-14:TP=-1.5:LRA=11 -acodec libvorbis -aq 6 "$OUTPUT" 2>/dev/null
            ;;
    esac

    COUNT=$((COUNT + 1))
done

echo ""
echo "Converted $COUNT files."

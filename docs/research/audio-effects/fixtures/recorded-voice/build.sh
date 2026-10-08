#!/usr/bin/env bash
# Rebuilds recorded-voice.flac from the Commons wax-cylinder recording.
# Input (not committed; fetched as PROVENANCE.md says): $VOICE_SRC  the Public domain Commons .ogg
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
: "${VOICE_SRC:?}"
# First 20 s, resampled to 48 kHz and written as stereo (mono source duplicated to both channels). No gain change.
ffmpeg -y -v error -i "$VOICE_SRC" -t 20 -af "aresample=48000,aformat=channel_layouts=stereo" -c:a flac "$here/recorded-voice.flac"

#!/usr/bin/env bash
# Rebuilds narration.flac, bed.flac and narration-over-bed.flac from the two sources.
# Inputs (not committed; fetched as PROVENANCE.md says): $PIPER_WAV  Piper output for narration.txt
#                                                       $BED_SRC    the CC0 Commons FLAC
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
: "${PIPER_WAV:?}" "${BED_SRC:?}"
# Narration: mono 22.05 kHz -> 48 kHz stereo, 0.8 s lead-in, padded to 20 s.
ffmpeg -y -v error -i "$PIPER_WAV" -af "aresample=48000,adelay=800:all=1,apad=whole_dur=20,atrim=0:20,aformat=channel_layouts=stereo" -c:a flac "$here/narration.flac"
# Bed: 20 s from 60 s into the source, 1.5 s fade in, 2 s fade out, -20 dB under the voice.
ffmpeg -y -v error -ss 60 -t 20 -i "$BED_SRC" -af "volume=-20dB,afade=t=in:d=1.5,afade=t=out:st=18:d=2" -c:a flac "$here/bed.flac"
# Mix: plain sum, no normalisation.
ffmpeg -y -v error -i "$here/narration.flac" -i "$here/bed.flac" -filter_complex "amix=inputs=2:normalize=0:duration=longest" -c:a flac "$here/narration-over-bed.flac"

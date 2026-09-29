"""Speak `voiceover/script.txt` with Kokoro-82M and write the voiceover and its word timings.

Run from `assets/`:
`uv run --python 3.12 --with kokoro --with soundfile python ../pack-src/make_voiceover.py`.

Writes `voiceover/voiceover.wav` (24 kHz mono) and `voiceover/voiceover.words.json`, a
list of `{"word", "start", "end"}` in integer milliseconds from the start of the file.
Model: hexgrad/Kokoro-82M, voice `af_heart`, both Apache-2.0.
"""

import json
from pathlib import Path

import numpy as np
import soundfile as sf
from kokoro import KPipeline

SR = 24_000
LEAD = 0.25  # seconds of silence before the first word
script = Path("voiceover/script.txt").read_text().strip()
pipeline = KPipeline(lang_code="a", repo_id="hexgrad/Kokoro-82M")

chunks = [np.zeros(int(SR * LEAD), dtype=np.float32)]
words = []
offset = LEAD
for result in pipeline(script, voice="af_heart", speed=1.0):
    audio = result.audio.numpy()
    for token in result.tokens:
        if token.start_ts is None or not any(c.isalnum() for c in token.text):
            continue
        words.append({
            "word": token.text,
            "start": round((offset + token.start_ts) * 1000),
            "end": round((offset + token.end_ts) * 1000),
        })
    chunks.append(audio)
    offset += len(audio) / SR

chunks.append(np.zeros(int(SR * LEAD), dtype=np.float32))
sf.write("voiceover/voiceover.wav", np.concatenate(chunks), SR, subtype="PCM_16")
Path("voiceover/voiceover.words.json").write_text(json.dumps(words, indent=2) + "\n")

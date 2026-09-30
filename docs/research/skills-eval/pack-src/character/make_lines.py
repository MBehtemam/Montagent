"""Voice the owl's lines with Azure Speech and record their visemes and word timings.

Run from `assets/`:
`uv run --python 3.12 --with azure-cognitiveservices-speech python ../pack-src/character/make_lines.py`.

Writes `character/voice/line-N.wav` (48 kHz mono), `line-N.txt` (the words) and
`line-N.json`: `{"voice", "duration_ms", "visemes": [[ms, id]], "words": [{"word",
"start", "end"}]}`, all times in integer milliseconds from the start of the file. Viseme
ids are Azure's 0–21; `character/rig.json` maps them to the rig's mouths. The keys come
from the git-ignored `.env` at the repository root (`AZURE_SPEECH_KEY`,
`AZURE_SPEECH_ENDPOINT`). Synthesis is not bit-exact across runs; the committed files are
the record.
"""

import json
import sys
from pathlib import Path

import azure.cognitiveservices.speech as sdk

ENV = Path(__file__).resolve().parents[5] / ".env"
env = dict(l.strip().split("=", 1) for l in ENV.read_text().splitlines() if "=" in l and not l.startswith("#"))
VOICE = "en-US-AvaNeural"
PROSODY = 'pitch="+12%"'

LINES = [
    "Hi, I'm Hoot! Your agent writes the video as a file, and Montagent turns it into a movie.",
    "Want to change one word in a video? Just edit the file, and Montagent renders it again.",
    "Hmm. That caption lands two frames late. Let me check... Fixed! Right on the beat.",
]

cfg = sdk.SpeechConfig(subscription=env["AZURE_SPEECH_KEY"], endpoint=env["AZURE_SPEECH_ENDPOINT"])
cfg.set_speech_synthesis_output_format(sdk.SpeechSynthesisOutputFormat.Riff48Khz16BitMonoPcm)
out = Path("character/voice")
out.mkdir(parents=True, exist_ok=True)

for i, text in enumerate(LINES, 1):
    wav = out / f"line-{i}.wav"
    syn = sdk.SpeechSynthesizer(cfg, sdk.audio.AudioOutputConfig(filename=str(wav)))
    visemes, words = [], []
    syn.viseme_received.connect(lambda e: visemes.append([e.audio_offset // 10_000, e.viseme_id]))
    syn.synthesis_word_boundary.connect(lambda e: words.append({
        "word": e.text, "start": e.audio_offset // 10_000,
        "end": e.audio_offset // 10_000 + int(e.duration.total_seconds() * 1000)})
        if e.boundary_type == sdk.SpeechSynthesisBoundaryType.Word else None)
    ssml = (f'<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US">'
            f'<voice name="{VOICE}"><prosody {PROSODY}>{text}</prosody></voice></speak>')
    r = syn.speak_ssml_async(ssml).get()
    if r.reason != sdk.ResultReason.SynthesizingAudioCompleted:
        sys.exit(f"line {i}: {r.reason} {r.cancellation_details.error_details if r.cancellation_details else ''}")
    del syn
    (out / f"line-{i}.txt").write_text(text + "\n")
    words_json = ",\n  ".join(json.dumps(w) for w in words)
    (out / f"line-{i}.json").write_text(
        f'{{\n "voice": "{VOICE}",\n "duration_ms": {int(r.audio_duration.total_seconds() * 1000)},\n'
        f' "visemes": {json.dumps(visemes)},\n "words": [\n  {words_json}\n ]\n}}\n')
    print(f"line {i}: {r.audio_duration.total_seconds():.2f}s, {len(visemes)} visemes, {len(words)} words")

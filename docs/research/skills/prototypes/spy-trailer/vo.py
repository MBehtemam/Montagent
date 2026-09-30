"""Prototype: the trailer's voice lines, Azure Speech, with word timings.
Run from this folder: uv run --python 3.12 --with azure-cognitiveservices-speech python vo.py"""
import json
import sys
from pathlib import Path

import azure.cognitiveservices.speech as sdk

ENV = Path(__file__).resolve().parents[5] / ".env"
env = dict(l.strip().split("=", 1) for l in ENV.read_text().splitlines() if "=" in l and not l.startswith("#"))

NARRATOR = ("en-US-DavisNeural", 'pitch="-18%" rate="-14%"')
VILLAIN = ("en-GB-ThomasNeural", 'pitch="-12%" rate="-12%"')
AGENT = ("en-GB-RyanNeural", 'pitch="-6%" rate="-4%"')
LINES = {
    "n1": (NARRATOR, "Every empire runs on secrets."),
    "n2": (NARRATOR, "And one man knows them all."),
    "villain": (VILLAIN, "You were never supposed to find me, Mister Vale."),
    "agent": (AGENT, "I get that a lot."),
    "n3": (NARRATOR, "This winter..."),
    "n4": (NARRATOR, "No one is off the grid."),
}

cfg = sdk.SpeechConfig(subscription=env["AZURE_SPEECH_KEY"], endpoint=env["AZURE_SPEECH_ENDPOINT"])
cfg.set_speech_synthesis_output_format(sdk.SpeechSynthesisOutputFormat.Riff48Khz16BitMonoPcm)
out = Path("vo")
out.mkdir(exist_ok=True)
for key, ((voice, prosody), text) in LINES.items():
    syn = sdk.SpeechSynthesizer(cfg, sdk.audio.AudioOutputConfig(filename=str(out / f"{key}.wav")))
    words = []
    syn.synthesis_word_boundary.connect(lambda e: words.append({
        "word": e.text, "start": e.audio_offset // 10_000,
        "end": e.audio_offset // 10_000 + int(e.duration.total_seconds() * 1000)})
        if e.boundary_type == sdk.SpeechSynthesisBoundaryType.Word else None)
    ssml = (f'<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US">'
            f'<voice name="{voice}"><prosody {prosody}>{text}</prosody></voice></speak>')
    r = syn.speak_ssml_async(ssml).get()
    if r.reason != sdk.ResultReason.SynthesizingAudioCompleted:
        sys.exit(f"{key}: {r.reason} {r.cancellation_details.error_details if r.cancellation_details else ''}")
    del syn
    (out / f"{key}.json").write_text(json.dumps({"duration_ms": int(r.audio_duration.total_seconds() * 1000),
                                                 "words": words}))
    print(key, f"{r.audio_duration.total_seconds():.2f}s", [w["word"] for w in words])

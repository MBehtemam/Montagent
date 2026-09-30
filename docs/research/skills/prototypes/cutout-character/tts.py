"""Voice each line with Azure Speech and record visemes + word boundaries.

Usage: uv run --with azure-cognitiveservices-speech python tts.py
Writes voice/line-N.wav and voice/line-N.json ({"visemes": [[ms, id]], "words": [[ms, dur_ms, word]]}).
"""
import json, sys
from pathlib import Path
import azure.cognitiveservices.speech as sdk

ENV = Path(__file__).resolve().parent.parent / ".env"
env = dict(l.strip().split("=", 1) for l in ENV.read_text().splitlines() if "=" in l and not l.startswith("#"))
VOICE = sys.argv[1] if len(sys.argv) > 1 else "en-US-AnaNeural"

LINES = [
    '<prosody rate="+5%" pitch="+8%">Oh! Hi there! I\'m Pip.</prosody>',
    '<prosody pitch="+8%">I built a machine that turns big ideas into...</prosody>',
    '<prosody rate="-10%" pitch="+8%">toast.</prosody>',
]

cfg = sdk.SpeechConfig(subscription=env["AZURE_SPEECH_KEY"], endpoint=env["AZURE_SPEECH_ENDPOINT"])
cfg.set_speech_synthesis_output_format(sdk.SpeechSynthesisOutputFormat.Riff48Khz16BitMonoPcm)
out = Path(__file__).parent / "voice"
out.mkdir(exist_ok=True)

for i, line in enumerate(LINES, 1):
    wav = out / f"line-{i}.wav"
    syn = sdk.SpeechSynthesizer(cfg, sdk.audio.AudioOutputConfig(filename=str(wav)))
    visemes, words = [], []
    syn.viseme_received.connect(lambda e: visemes.append([e.audio_offset // 10_000, e.viseme_id]))
    syn.synthesis_word_boundary.connect(
        lambda e: words.append([e.audio_offset // 10_000, int(e.duration.total_seconds() * 1000), e.text]))
    ssml = (f'<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US">'
            f'<voice name="{VOICE}">{line}</voice></speak>')
    r = syn.speak_ssml_async(ssml).get()
    if r.reason != sdk.ResultReason.SynthesizingAudioCompleted:
        sys.exit(f"line {i}: {r.reason} {r.cancellation_details.error_details if r.cancellation_details else ''}")
    del syn
    (out / f"line-{i}.json").write_text(json.dumps({"voice": VOICE, "duration_ms": int(r.audio_duration.total_seconds() * 1000),
                                                    "visemes": visemes, "words": words}))
    print(f"line {i}: {r.audio_duration.total_seconds():.2f}s, {len(visemes)} visemes, {len(words)} words")

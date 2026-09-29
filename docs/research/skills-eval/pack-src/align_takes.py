"""Time every word of each presenter take, and write it beside the take.

Run from `assets/`:
`uv run --python 3.12 --with faster-whisper python ../pack-src/align_takes.py`.

For each `presenter/take-N.mp4` with a `presenter/take-N.txt` script, Whisper (the
`small.en` model, on CPU, prompted with the script) supplies word timestamps and the
script supplies the words. The two have to agree word for word, ignoring case and
punctuation; if they don't, the take is reported and nothing is written, because a
caption checked against words the presenter did not say is a wrong answer key.

Whisper's word edges drift into pauses: it will start a word half a second early,
inside the silence before it. So each edge that falls inside a silence found by ffmpeg's
`silencedetect` (-40 dB, at least 100 ms) is moved to that silence's boundary: a start to
where the silence ends, an end to where it begins.

Writes `presenter/take-N.words.json` in the voiceover's form: `{"word", "start", "end"}`
in integer milliseconds.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

from faster_whisper import WhisperModel


def norm(word):
    return re.sub(r"[^a-z0-9]", "", word.lower())


def silences(video):
    log = subprocess.run(
        ["ffmpeg", "-hide_banner", "-i", str(video), "-af", "silencedetect=n=-40dB:d=0.1", "-f", "null", "-"],
        capture_output=True, text=True, check=True,
    ).stderr
    starts = [float(x) for x in re.findall(r"silence_start: ([\d.]+)", log)]
    ends = [float(x) for x in re.findall(r"silence_end: ([\d.]+)", log)]
    return list(zip(starts, ends + [float("inf")] * (len(starts) - len(ends))))


def snap(start, end, quiet):
    for lo, hi in quiet:
        if lo < start < hi:
            start = hi
        if lo < end < hi:
            end = lo
    return start, max(end, start)


model = WhisperModel("small.en", device="cpu", compute_type="int8")
failed = False
for script_path in sorted(Path("presenter").glob("take-*.txt")):
    video = script_path.with_suffix(".mp4")
    script_words = script_path.read_text().split()
    segments, _ = model.transcribe(
        str(video), word_timestamps=True, language="en", initial_prompt=" ".join(script_words)
    )
    heard = [w for s in segments for w in s.words]
    if [norm(w) for w in script_words] != [norm(w.word) for w in heard]:
        print(f"{video}: heard {' '.join(w.word.strip() for w in heard)!r}", file=sys.stderr)
        failed = True
        continue
    quiet = silences(video)
    words = []
    for text, w in zip(script_words, heard):
        start, end = snap(w.start, w.end, quiet)
        words.append({"word": text, "start": round(start * 1000), "end": round(end * 1000)})
    video.with_suffix(".words.json").write_text(json.dumps(words, indent=2) + "\n")
    print(f"{video}: {len(words)} words, {words[0]['start']}-{words[-1]['end']} ms")
sys.exit(1 if failed else 0)

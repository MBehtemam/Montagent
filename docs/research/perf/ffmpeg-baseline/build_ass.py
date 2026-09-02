#!/usr/bin/env python3
"""Merge the fixture's per-segment .ass files onto one absolute clock."""
import re, pathlib

FIX = pathlib.Path("/Users/mohammedehtemam/projects/github/Montaget/fixtures/en-halloween-decorating")
SUBS = FIX / "reference/subtitles"

# segment order + duration (s), from the fixture README's structure table
SEGMENTS = [
    ("intro",     3.018),
    ("05-cobweb", 14.454),
    ("06-spider", 13.131),
    ("07-skeleton", 12.160),
    ("08-lights", 11.093),
    ("quiz",      10.160),
    ("loop-tail",  1.200),
]

def parse_ts(t):
    h, m, s = t.split(":")
    return int(h) * 3600 + int(m) * 60 + float(s)

def fmt_ts(v):
    h = int(v // 3600); v -= h * 3600
    m = int(v // 60);   v -= m * 60
    return f"{h}:{m:02d}:{v:05.2f}"

header, styles, events = [], [], []
seen_styles = set()
offset = 0.0

for name, dur in SEGMENTS:
    path = SUBS / f"{name}.ass"
    section = None
    for line in path.read_text().splitlines():
        st = line.strip()
        if st.startswith("["):
            section = st.lower()
            continue
        if section == "[script info]":
            if not header and st:
                pass
            if st and st not in header:
                header.append(st)
        elif section == "[v4+ styles]":
            if st.startswith("Style:") and st not in seen_styles:
                seen_styles.add(st); styles.append(st)
            elif st.startswith("Format:") and not any(s.startswith("Format:") for s in styles):
                styles.insert(0, st)
        elif section == "[events]":
            if st.startswith("Format:"):
                if not any(e.startswith("Format:") for e in events):
                    events.insert(0, st)
            elif st.startswith("Dialogue:"):
                m = re.match(r"(Dialogue:\s*[^,]*,)([^,]+),([^,]+),(.*)", st)
                start = parse_ts(m.group(2)) + offset
                end   = parse_ts(m.group(3)) + offset
                events.append(f"{m.group(1)}{fmt_ts(start)},{fmt_ts(end)},{m.group(4)}")
    offset += dur

out = ["[Script Info]"] + header + ["", "[V4+ Styles]"] + styles + ["", "[Events]"] + events
pathlib.Path("merged.ass").write_text("\n".join(out) + "\n")
print(f"merged.ass: {len(events)-1} dialogue lines, total {offset:.3f}s")

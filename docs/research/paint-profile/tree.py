"""Inclusive sample totals from a macOS `sample` call tree (main thread only).
usage: tree.py <sample.txt> [pattern=label ...]   — a node counts toward the first pattern on its path
(outermost match wins, so nested matches aren't double counted)."""
import re, sys

lines = open(sys.argv[1]).read().split("\n")
start = next(i for i, l in enumerate(lines) if "com.apple.main-thread" in l)
node = re.compile(r"^(\s*[+!:| ]*?)(\d+) (.*)$")
nodes = []  # (depth, count, name)
for l in lines[start + 1 :]:
    if not l.startswith("    +") and not l.startswith("    !") and not l.startswith("    :") and not l.startswith("    |"):
        if nodes:
            break
        continue
    m = node.match(l)
    if not m:
        continue
    nodes.append((len(m.group(1)), int(m.group(2)), m.group(3)))


total = nodes[0][1]
pats = [a.split("=", 1) for a in sys.argv[2:]]
labels = [l for _, l in pats]
excl = {l: 0 for l in labels + ["(unmatched)"]}
# self samples = count - sum of direct children
selfc = [c for _, c, _ in nodes]
st = []
for i, (d, c, n) in enumerate(nodes):
    while st and nodes[st[-1]][0] >= d:
        st.pop()
    if st:
        selfc[st[-1]] -= c
    st.append(i)
st = []
for i, (d, c, n) in enumerate(nodes):
    while st and st[-1][0] >= d:
        st.pop()
    lab = next((l for p, l in pats if re.search(p, n)), None)
    eff = lab or (st[-1][1] if st else None)
    st.append((d, eff))
    excl[eff or "(unmatched)"] += selfc[i]
print(f"total {total}")
for lab, s in excl.items():
    print(f"{lab:40s} {s:8d}  {100*s/total:5.1f}%")

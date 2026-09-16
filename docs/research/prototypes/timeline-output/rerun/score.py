runs = {  # (condition, actual tool calls -- harness-recorded, never self-reported, correct/5)
 # round 2 (#11)
 'X1':('X',10,5), 'X2':('X',10,5),
 'Y1a':('Y1',4,5), 'Y1b':('Y1',7,5), 'Y1c':('Y1',7,5),
 'Y2a':('Y2',8,5), 'Y2b':('Y2',7,5), 'Y2c':('Y2',11,5),
 # round 3 (#63) -- three more runs per arm, one model (Sonnet), blind to the ablation,
 # scored against the same GROUND-TRUTH.txt, tool count from each run's <usage> block
 'Y1d':('Y1',10,5), 'Y1e':('Y1',12,5), 'Y1f':('Y1',6,5),
 'Y2d':('Y2',11,5), 'Y2e':('Y2',8,5), 'Y2f':('Y2',11,5),
}
from statistics import mean
for c in ['X','Y1','Y2']:
    v=[n for cc,n,_ in runs.values() if cc==c]
    print(f"{c:<3} n={len(v)}  calls={sorted(v)}  mean={mean(v):.1f}  range={min(v)}-{max(v)}")
print()
print("correctness: all 14 runs 5/5 -> the view changes NO answer, only cost")

# Mann-Whitney U, Y1 vs Y2, n=6 each (round 2 + round 3 combined)
y1 = sorted(n for cc, n, _ in runs.values() if cc == 'Y1')
y2 = sorted(n for cc, n, _ in runs.values() if cc == 'Y2')
combined = sorted((v, lab) for lab, vs in (('Y1', y1), ('Y2', y2)) for v in vs)
ranks, i = {}, 0
while i < len(combined):
    j = i
    while j < len(combined) and combined[j][0] == combined[i][0]:
        j += 1
    avg_rank = (i + 1 + j) / 2
    for k in range(i, j):
        ranks.setdefault(combined[k][1], []).append(avg_rank)
    i = j
r1 = sum(ranks['Y1'])
n1, n2 = len(y1), len(y2)
u1 = r1 - n1 * (n1 + 1) / 2
u2 = n1 * n2 - u1
u = min(u1, u2)
print(f"Mann-Whitney U(Y1,Y2) n={n1},{n2}: U={u:.1f} (critical at alpha=.05 two-tailed, n=6,6: U<=5 to reject)")
print("-> not significant: the axis effect is not established even at n=6 per arm.")
print("   Y1's new runs (6,10,12) widened its own range past Y2's max (11) -- more")
print("   data pulled the two distributions together, not apart.")

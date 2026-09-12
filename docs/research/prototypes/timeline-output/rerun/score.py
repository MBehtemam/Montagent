runs = {  # (condition, actual tool calls from task metadata, correct/5)
 'X1':('X',10,5), 'X2':('X',10,5),
 'Y1a':('Y1',4,5), 'Y1b':('Y1',7,5), 'Y1c':('Y1',7,5),
 'Y2a':('Y2',8,5), 'Y2b':('Y2',7,5), 'Y2c':('Y2',11,5),
}
from statistics import mean
for c in ['X','Y1','Y2']:
    v=[n for cc,n,_ in runs.values() if cc==c]
    print(f"{c:<3} n={len(v)}  calls={sorted(v)}  mean={mean(v):.1f}  range={min(v)}-{max(v)}")
print()
print("correctness: all runs 5/5 -> the view changes NO answer, only cost")

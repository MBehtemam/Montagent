#!/usr/bin/env python3
"""Re-runnable check for ADR-0177's level arithmetic. Exits non-zero when a number stops holding.

Usage:  python3 -I check_duck_levels.py [--out measurements.json]      (stdlib only)

The duck script takes its depth in dB (`under_db`, `over_db`, `end_db`, ADR-0170) and writes linear
`volume` keyframes rounded to four decimals. This checks the three claims that rests on:

  1. The dB defaults reproduce today's linear defaults (captions.py: 0.18, 0.5, 0.85) to within
     0.15 dB, so a script that delegates to the new one does not change a duck audibly.
  2. Each dB default sits inside the range the footage skill's guidance quotes for that level
     (0.15-0.2 under speech, 0.4-0.6 in pauses, 0.8-1.0 after the last word).
  3. Rounding the linear level to four decimals costs at most 0.01 dB for every level above the
     threshold printed below, and the three defaults are inside it.
"""
import argparse, json, math, sys

DEFAULTS_DB = {"under_db": -15.0, "over_db": -6.0, "end_db": -1.5}
TODAY = {"under_db": 0.18, "over_db": 0.5, "end_db": 0.85}             # captions.py's linear defaults
GUIDANCE = {"under_db": (0.15, 0.20), "over_db": (0.4, 0.6), "end_db": (0.8, 1.0)}
DECIMALS = 4
MAX_ROUND_ERR_DB, MAX_DEFAULT_SHIFT_DB = 0.01, 0.15

lin = lambda db: 10 ** (db / 20)
db = lambda v: 20 * math.log10(v)


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--out"); a = ap.parse_args()
    fails, res = [], {"decimals": DECIMALS, "levels": {}}
    for k, d in DEFAULTS_DB.items():
        v = round(lin(d), DECIMALS)
        r = {"default_db": d, "written_linear": v, "today_linear": TODAY[k],
             "today_in_db": round(db(TODAY[k]), 3), "shift_db": round(abs(d - db(TODAY[k])), 3),
             "guidance_linear": GUIDANCE[k], "guidance_in_db": [round(db(GUIDANCE[k][0]), 2), round(db(GUIDANCE[k][1]), 2)],
             "rounding_error_db": round(abs(db(v) - d), 5)}
        res["levels"][k] = r
        if r["shift_db"] > MAX_DEFAULT_SHIFT_DB: fails.append(f"{k}: default shifts {r['shift_db']} dB from today's {TODAY[k]}")
        if not GUIDANCE[k][0] <= v <= GUIDANCE[k][1]: fails.append(f"{k}: {v} is outside the guidance range {GUIDANCE[k]}")
        if r["rounding_error_db"] > MAX_ROUND_ERR_DB: fails.append(f"{k}: rounding costs {r['rounding_error_db']} dB")
    # worst-case rounding error is half a step in the last decimal; find where it stays under 0.01 dB
    half = 0.5 * 10 ** -DECIMALS
    floor_lin = half * (20 / math.log(10)) / MAX_ROUND_ERR_DB          # 8.686 * half / v <= 0.01  (small-error limit)
    res["rounding_floor"] = {"max_error_db": MAX_ROUND_ERR_DB, "levels_above_linear": round(floor_lin, 4),
                             "levels_above_db": round(db(floor_lin), 2)}
    # the claim, checked by scan: rounding any level in [floor, 1] to four decimals costs <= 0.01 dB
    n = 200000
    worst = max(abs(db(round(v, DECIMALS)) - db(v)) for v in (floor_lin + (1 - floor_lin) * i / n for i in range(n + 1)))
    res["rounding_floor"]["worst_error_db_scanned"] = round(worst, 5)
    if worst > MAX_ROUND_ERR_DB * 1.02: fails.append(f"rounding to {DECIMALS} decimals costs {worst:.5f} dB above {floor_lin:.4f}")
    if a.out: open(a.out, "w").write(json.dumps(res, indent=1) + "\n")
    print(json.dumps(res, indent=1))
    if fails: sys.exit("FAILED:\n  " + "\n  ".join(fails))


main()

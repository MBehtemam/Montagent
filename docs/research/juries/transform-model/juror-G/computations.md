# Working computations (juror G)

## Exercise 2: de Casteljau split of CSS ease-in-out at t=0.5

Control points (x,y): P0=(0,0) P1=(0.42,0) P2=(0.58,1) P3=(1,1)

Level 1:
A=(P0+P1)/2=(0.21,0)
B=(P1+P2)/2=(0.5,0.5)
C=(P2+P3)/2=(0.79,1)

Level 2:
D=(A+B)/2=(0.355,0.25)
E=(B+C)/2=(0.645,0.75)

Level 3 (split point):
F=(D+E)/2=(0.5,0.5)

Left curve (unnormalized): P0,A,D,F = (0,0)(0.21,0)(0.355,0.25)(0.5,0.5)
Right curve (unnormalized): F,E,C,P3 = (0.5,0.5)(0.645,0.75)(0.79,1)(1,1)

Renormalize left to [0,1]x[0,1] (scale by 2, no offset):
(0,0)(0.42,0)(0.71,0.5)(1,1) -> cubic-bezier(0.42,0,0.71,0.5)
Compare ease-in cubic-bezier(0.42,0,1,1): P2 differs (0.71,0.5) vs (1,1). NOT ease-in. NOT linear, ease, ease-out either.

Renormalize right (offset -0.5,-0.5, scale 2):
(0,0)(0.29,0.5)(0.58,1)(1,1) -> cubic-bezier(0.29,0.5,0.58,1)
Compare ease-out cubic-bezier(0,0,0.58,1): P1 differs (0.29,0.5) vs (0,0). NOT ease-out.

Conclusion: neither half of the MOST SYMMETRIC possible split point reduces to any named CSS easing.
The named vocabulary {linear, ease, ease-in, ease-out, ease-in-out} is not closed under subdivision.

## Exercise 1(a): non-linear ease under MOVE/HOLD/SPLIT

Segment (invented, ease-in-out): kf1 t=17472 v=1.0, kf2 t=32472 v=1.08 (mirrors photo-06)
shift(at=20000, delta=2000)

MOVE: kf2 -> t=34472 (kf1 unchanged, 17472<20000)
Query value at new element end 32603:
local x = (32603-17472)/(34472-17472) = 15131/17000 = 0.890059
Solve bezier x(t)=0.890059 for t (control pts 0.42,0,0.58,1): t ≈ 0.9077
y(t) with y-ctrl (0,0,1,1): y = 3(1-t)t^2 + t^3 ≈ 0.975967
value = 1.0 + 0.975967*0.08 = 1.078077 (~1.0781)
Contrast: LINEAR interpolation would give 1.0 + 0.890059*0.08 = 1.071205 (~1.0712, matches round-1's quoted MOVE number)
=> the round-1 "1.0712" figure is a linear-only artifact; under non-linear ease MOVE gives a materially different number (1.0781, not 1.0712).

SPLIT: evaluate original curve at absolute t=20000.
local x = (20000-17472)/(32472-17472) = 2528/15000 = 0.168533
Solve x(t)=0.168533: t ≈ 0.1456
y(t) = 3(1-t)t^2 + t^3 at t=0.1456: 1-t=0.8544, t^2=0.021199
3*0.8544*0.021199=0.054336; t^3=0.003086; y=0.057422
value = 1.0 + 0.057422*0.08 = 1.004594 (~1.0046)
Contrast: LINEAR SPLIT (round-1's own worked number) gives 1.0 + 0.168533*0.08 = 1.013483.
=> under non-linear ease, SPLIT's frozen plateau value is 1.0046, not 1.0135 -- also diverges from the linear case materially.

Structural problem: SPLIT inserts two new keyframes (at 20000 and 22000, both v=1.004594)
which chop the original single ease-in-out ramp into three pieces:
  [17472 -> 20000] labelled ease-in-out (but is really the FIRST 16.85% of one, i.e. NOT closed, per Ex.2)
  [20000 -> 22000] flat plateau, "ease-in-out" is meaningless on an interval with equal endpoints
  [22000(orig, now +2000=34472 after shift) -> ...] labelled ease-in-out but is really the LAST 83% of one
Two of three resulting labelled segments are NOT the curve their label claims (Ex.2's math),
and the middle segment's ease label is undefined (start value == end value).

## Exercise 1(b): at exactly on an existing (interior, non-start) keyframe

Invented 3-keyframe element mirroring photo-06 pacing: start=17472, end=30603 (pre-shift)
scale = [{t:17472,v:1.0},{t:20000,v:1.02},{t:32472,v:1.08}] (linear)
shift(at=20000, delta=2000) -> end stretches to 32603 (ADR-0005 stretch rule for time-invariant straddler)

MOVE: kf@20000 and kf@32472 both >=20000, shift by +2000: 22000@1.02, 34472@1.08. kf@17472 unchanged.
Result: [{17472,1.0},{22000,1.02},{34472,1.08}], end=32603
Value at query t=32603 (between 22000 and 34472):
frac=(32603-22000)/(34472-22000)=10603/12472=0.850180
value=1.02+0.850180*0.06=1.020+0.0510108=1.071011 (~1.0710)

HOLD: keyframes untouched [{17472,1.0},{20000,1.02},{32472,1.08}], end=32603 (past last kf 32472)
clamp rule: after last keyframe, value = last value = 1.08.

SPLIT (literal algorithm, naive order: insert-then-shift-all->=at):
Value at at=20000 is already exact -- an existing keyframe there, v=1.02 (trivial "evaluation").
Insert kf at 20000 (v=1.02) [DUPLICATE of existing kf@20000] and at 22000 (v=1.02).
Sequence before shift: 17472@1.0, 20000@1.02(orig), 20000@1.02(dup,new), 22000@1.02(new), 32472@1.08
Shift all t>=20000 by +2000:
17472@1.0, 22000@1.02(orig,shifted), 22000@1.02(dup,shifted) <- COLLIDES with orig,
24000@1.02(the inserted-22000 one, now shifted to 24000) <- SPURIOUS extra keyframe, 34472@1.08
Result has a duplicate-timestamp pair at 22000 and an orphan flat keyframe at 24000 that
correspond to nothing in the source data or the edit's intent.
=> SPLIT literally applied breaks (duplicate timestamps + spurious keyframe) whenever `at`
   coincides with an existing keyframe. Needs an unstated special case: skip the `at`
   insertion when a keyframe already exists there.

## Exercise 1(c): element entirely after `at`

photo-05-quiz: start=53856, end=64016, scale=[[53856,1.0],[68856,1.08]] (already an
out-of-range/"trimmed move" keyframe past its own end, per FINDINGS.md's own reading)
shift(at=50000, delta=2000)

Classification: nothing straddles 50000 (element starts at 53856 > 50000). Whole element
must translate uniformly: start->55856, end->66016, kf1 53856->55856, kf2 68856->70856.
This IS "MOVE" trivially and is uncontroversial -- but only because it is NOT a straddle case.

If a "HOLD-flavored" implementation instead leaves keyframe TIMES untouched whenever a
keyframe is "not the thing being stretched" (a plausible naive generalisation of HOLD),
kf1 stays at 53856, which is now *before* the element's new start (55856):
Query at new start 55856: frac=(55856-53856)/(68856-53856)=2000/15000=0.133333
value=1.0+0.133333*0.08=1.010667 (~1.0107) -- i.e. the shot appears to already be
1.07% zoomed in at its very first visible frame, though nothing about its own content
changed; only an unrelated earlier insert moved it in time. This is a corruption bug,
concretely: a naive HOLD generalisation breaks every ordinary (non-straddling) shifted
element with keyframes, which is the overwhelming majority of shift's real-world targets.

If SPLIT is naively applied here: evaluate property at at=50000 -- the element does not
exist yet (starts at 53856). No principled value exists. If clamped to the first keyframe
value backward (extending round-1's "before first keyframe = first value" clamp past the
element's own start), v=1.0. Insert kf at 50000 and 52000 (both v=1.0), then shift
t>=50000 by +2000: 52000@1.0, 54000@1.0, 55856@1.0(orig kf1, shifted), 70856@1.08.
Three keyframes at value 1.0 (52000, 54000, 55856), two of them (52000, 54000) sitting
strictly BEFORE the element's own new start (55856) -- a "leading dangling keyframe"
shape that is the mirror image of the approved "trailing dangling keyframe" (trimmed
move) but has never been sanctioned by any ADR, and serves no purpose.

## Exercise 3: seven Ken Burns lists, scalar vs [sx,sy]

Scalar total char count for the 14 v-values across all seven elements:
7 * (len("1.0") + len("1.08")) = 7 * (3+4) = 49 chars

Array total char count for the same 14 values, doubled into [sx,sy]:
7 * (len("[1.0,1.0]") + len("[1.08,1.08]")) = 7 * (9+11) = 140 chars

Delta: +91 chars (2.86x) for a property that is isotropic in 14/14 keyframes across
all seven elements in the only real project file this repo has.

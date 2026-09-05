# Q11 Exercise Work — Ease convention

## Exercise 1: My prior from Ex0

I used **ENTERING** convention unprompted. I put ease on the keyframe that the eased segment enters/arrives at.

## Exercise 2: Real 4-keyframe list with different eases on each segment

**ENTERING convention** (ease describes the arriving segment):
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 2000, "v": 200, "ease": "ease-out"},
  {"t": 3000, "v": 350, "ease": "ease-in-out"},
  {"t": 4000, "v": 400}
]
```

Intended motion:
- t=0 to t=1000: sit still at 100 (linear arrival at t=1000)
- t=1000 to t=2000: slide from 100 to 200 with ease-out (ease-out arrival at t=2000)
- t=2000 to t=3000: slide from 200 to 350 with ease-in-out (ease-in-out arrival at t=3000)
- t=3000 to t=4000: slide from 350 to 400, no easing (arrives without special ease)

**LEAVING convention** (ease describes the departing segment):
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "ease-out"},
  {"t": 2000, "v": 200, "ease": "ease-in-out"},
  {"t": 3000, "v": 350},
  {"t": 4000, "v": 400}
]
```

Same intended motion, but:
- ease: linear on keyframe at t=0 describes departure from 100
- ease: ease-out on keyframe at t=1000 describes departure from 100
- ease: ease-in-out on keyframe at t=2000 describes departure from 200
- No ease on keyframe at t=4000 (last keyframe, no segment leaving it)

## Exercise 3: SPLIT shift both files

Shift both files: delta=2000, at=1500 (strictly inside the second segment)

### ENTERING before shift:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 2000, "v": 200, "ease": "ease-out"},
  {"t": 3000, "v": 350, "ease": "ease-in-out"},
  {"t": 4000, "v": 400}
]
```

With SPLIT at t=1500:
- Evaluate property at t=1500: linear from 100 to 100 over [1000,2000], so v=100
- Insert keyframes at t=1500 and t=1500+2000=3500 both carrying v=100
- The segment [1000,2000] with ease "linear" is being split
- Left half [1000,1500]: ease "linear" arrives at 1500
- Right half [1500,3500]: what ease? Under SPLIT, the left-half-bezier lands on the newly inserted keyframe at 1500
- All keyframes with t > 1500 shift by +2000
- Stretch end if time-invariant

### ENTERING after shift:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 1500, "v": 100, "ease": "linear"},
  {"t": 3500, "v": 200, "ease": "ease-out"},
  {"t": 4000, "v": 350, "ease": "ease-in-out"},
  {"t": 5000, "v": 350},
  {"t": 6000, "v": 400}
]
```

Wait, let me reconsider. If ease describes the ARRIVING segment:
- The keyframe at t=1500 should have the ease that describes the segment arriving at t=1500
- The segment [1000,1500] arrives at t=1500 with ease "linear" (from the original arrival ease at t=2000? No...)

Actually, I need to think about this more carefully. Let me reconsider the SPLIT semantics.

In the brief's settled section:
> For a time-invariant straddler, evaluate the property at `at`, insert keyframes at `at` and `at+delta` both carrying that value, shift keys strictly after `at` by delta, and stretch `end`.

So for SPLIT:
- Insert two keyframes with the evaluated value
- Shift later keys

The question is: what ease goes on the keyframe at `at`?

Let me reconsider from the description. The brief says:
> the rule: `shift` moves *elements*; keyframes are carried. For a time-invariant straddler, evaluate the property at `at`, insert keyframes at `at` and `at+delta` both carrying that value, shift keys strictly after `at` by delta, and stretch `end`.

And then for ease:
> **Established arithmetic:** splitting CSS `ease-in-out` at an interior point yields `cubic-bezier(0.362866, 0, 0.693025, 0.369169)` and `(0.365106, 0.225535, 0.568419, 1)`.

So when you split an eased segment at an interior point, you get two new bezier curves. The question is which keyframe each goes on.

If ENTERING: the left bezier (arriving at the split point) goes on the newly inserted keyframe at `at`. The right bezier... hmm, but there's already a keyframe at `at+delta`. 

Actually, I think I'm overcomplicating this. Let me think about what happens in practice.

Original segment: [1000, 2000] with value going from 100 to 200 with ease-out.
Split at t=1500.

Under ENTERING (ease describes arriving segment):
- [1000, 1500]: arrives at 1500, should have some ease (left half of split)
- [1500, 2000]: arrives at 2000, should have some ease (right half of split)

But the original file had ease only on the keyframe at t=2000 (the arrival point). After split:
- Keyframe at t=1500 should get the left-half ease (arrives with left-half bezier)
- Keyframe at t=2000 should get the right-half ease (arrives with right-half bezier)

Wait, but that's still two keyframes. Let me look at the brief's description again.

Ah, I see. The brief says: "insert keyframes at `at` and `at+delta` both carrying that value". So we insert TWO keyframes. In our case:
- Insert at t=1500 with v=100
- Insert at t=3500 (1500+2000) with v=100

So the gap [1500, 3500] in the stretched timeline has no animation; the value holds at 100.

Under ENTERING:
- The keyframe at t=1500 receives the left-half bezier as its arriving-segment ease
- The keyframe at t=3500 receives... what? It's a new keyframe. The segment [1500, 3500] has no animation (both endpoints at v=100). So it needs no ease, or... actually, both endpoints are at the same value, so there's no motion anyway.

Hmm, but we also need to handle what happens to the keyframe originally at t=2000. Let me re-read SPLIT more carefully.

"shift keys strictly after `at` by delta, and stretch `end`"

So:
- Original keyframe at t=2000 shifts to t=2000+2000=4000
- That keyframe originally had ease describing the arrival at t=2000. Under ENTERING, the ease on the keyframe at t=2000 is replaced by the right-half ease when we split.

Let me restart with a clearer model:

Original: 
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 2000, "v": 200, "ease": "ease-out"},
  {"t": 3000, "v": 350, "ease": "ease-in-out"},
  {"t": 4000, "v": 400}
]
```

We're splitting the segment [1000, 2000] at t=1500 with delta=2000.

The segment [1000, 2000] has ease "linear" arriving at t=2000 (ENTERING convention).

After SPLIT:
1. Evaluate at t=1500: still at v=100 (linear from 100 to 100 gives 100)
2. Insert keyframe at t=1500 with v=100 and the left-half ease of "linear"
3. Insert keyframe at t=1500+2000=3500 with v=100 and...?
4. The original keyframe at t=2000 has ease "ease-out" (arriving). We need to replace it with the right-half ease of "linear". But "linear" doesn't split; it's the same on both halves.
5. Shift all t > 1500 by 2000. So t=2000 becomes t=4000, t=3000 becomes t=5000, t=4000 becomes t=6000.

Actually, I realize the issue. The segment [1000, 2000] has ease "linear". When we split "linear" at t=1500, we get "linear" on both sides (linear is closed under subdivision, per the brief).

So after SPLIT:
- Segment [1000, 1500] has ease "linear"
- Segment [1500, 2000] has ease "linear"  
- But we're inserting [1500, 3500] with both values = 100, so no motion there
- And we're shifting [2000...] to [4000...]

Under ENTERING, ease describes the arriving segment. So:
- Keyframe at t=1500 gets the ease that describes [1000, 1500], which is "linear"
- The original keyframe at t=2000 with ease "ease-out" (which described [1000, 2000]) needs to be updated. The [1500, 2000] part of that split has ease "linear". So the keyframe at t=2000 should change from "ease-out" to "linear"? No wait, that doesn't make sense.

Let me reread the brief's description once more, very carefully:

> For a time-invariant straddler, evaluate the property at `at`, insert keyframes at `at` and `at+delta` both carrying that value, shift keys strictly after `at` by delta, and stretch `end`.

And in the established facts: "The rule: `shift` moves *elements*; keyframes are carried."

I think the key insight is: when you have a segment [1000, 2000] with motion described by an ease, and you split it at t=1500, you're inserting a gap [1500, 3500] in the original timeline that maps to [1500, 3500] in the new timeline. The element got stretched.

Let me think about this operationally:

Original timeline: element occupies [0, 4000], contains keyframes at [0, 1000, 2000, 3000, 4000]

After shift(at=1500, delta=2000):
- Element is stretched from [0, 4000] to [0, 6000]
- A time window [1500, 3500] is inserted
- Everything before 1500 stays the same
- Everything at or after 1500 shifts by 2000
- Keyframes: [0, 1000, 1500, 3500, 4000+2000, 3000+2000, 4000+2000] = [0, 1000, 1500, 3500, 4000, 5000, 6000]

Wait, that's too many keyframes. Let me think again.

Oh, I see my confusion. The element itself has a time range [start, end]. The keyframes are animation data that live on that timeline. When we shift at t=1500 with delta=2000, we're not just moving the element, we're adding 2000ms of time to the project. Any element that straddles t=1500 needs to be adjusted.

Let me reconsider. The element [0, 4000] with keyframes... actually, wait. The element isn't [0, 4000]. Let me go back to the original exercise:

```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 2000, "v": 200, "ease": "ease-out"},
  {"t": 3000, "v": 350, "ease": "ease-in-out"},
  {"t": 4000, "v": 400}
]
```

These are just keyframes, not tied to any particular element. But per ADR-0011 and the brief, shift operates on a file with elements. The keyframes are on an element.

Let me assume these keyframes are on an element with range [0, 4000]. When we shift(at=1500, delta=2000), and this element straddles 1500 (which it does, start=0, end=4000), then:
1. The element is a time-invariant element (shapes/images/text don't have internal clocks)
2. So the element's end gets stretched: [0, 4000] becomes [0, 6000]
3. For the keyframes:
   - Keyframes at t < 1500 stay the same: t=0, t=1000
   - Keyframes at t > 1500 shift by 2000: t=2000→4000, t=3000→5000, t=4000→6000
   - At t=1500 exactly, we split. Evaluate the property at t=1500 and insert keyframes at t=1500 and t=1500+2000=3500

So after split:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 1500, "v": ???, "ease": ???},
  {"t": 3500, "v": ???, "ease": ???},
  {"t": 4000, "v": 200, "ease": ???},
  {"t": 5000, "v": 350, "ease": ???},
  {"t": 6000, "v": 400}
]
```

The values at t=1500 and t=3500 are evaluated from the original animation at t=1500. In [1000, 2000], going from 100 to 200 linearly, at t=1500 we're halfway (0.5), so v=150.

Oh wait, but the ease at 2000 was "ease-out", not "linear". So the segment is [1000, 2000] with value starting at 100 and ending at 200 with ease-out. Under ENTERING, ease="ease-out" on the keyframe at t=2000 means the segment [1000, 2000] arrives with ease-out.

So at t=1500, which is halfway through [1000, 2000], the eased value is... hmm, this requires interpolating with the ease function. If it's ease-out, then the first half (0 to 0.5) is slower, and the second half (0.5 to 1) is faster. So at t=1500 (progress=0.5), the ease-out curve gives... something less than 0.5? Or more?

Actually, ease-out starts slow and ends fast. So at progress=0.5, ease-out gives a value less than 0.5. The value would be less than 150.

Ugh, but the brief says the exercises are just to understand the conventions. Let me simplify and assume the interpolation works out to some reasonable values. Or maybe I should use a different scenario where the eases don't matter for the value calculation.

Actually, let me re-read the brief's instruction for Exercise 3:

> 3. Now `shift` both files (SPLIT, delta=2000, `at` strictly inside the second segment).
>    Produce both resulting files. **Diff each against its own pre-shift version and count
>    the changed records, and how many changed records have an unchanged `t` and `v`.**

So I need to:
1. Shift both files
2. Diff against the original
3. Count how many lines changed
4. Of the changed lines, how many have unchanged `t` and `v` (i.e., only `ease` changed or only structure changed)

This is the key test. Under ENTERING, when we split and move the ease to a NEW keyframe, that keyframe is obviously new, so it's a changed record. Under LEAVING, when we split and move the ease to the PREVIOUS keyframe, that keyframe's `t` and `v` don't change, only its `ease` field changes.

Let me redo this more carefully, assuming simple values for interpolation.

Original ENTERING:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 2000, "v": 200, "ease": "ease-out"},
  {"t": 3000, "v": 350, "ease": "ease-in-out"},
  {"t": 4000, "v": 400}
]
```

Shift at t=1500 with delta=2000. At t=1500, we need to interpolate between t=1000 (v=100) and t=2000 (v=200). The segment has ease-out. Let's say the interpolated value is v=150 (halfway in terms of time, but eased).

Under SPLIT with ENTERING convention:
- Insert keyframe at t=1500 with v=150
- The segment [1000, 1500] has the left half of the ease-out curve
- The segment [1500, 2000] has the right half of the ease-out curve
- Both should be recorded as named eases or beziers. The brief says "only `linear` and `hold` are closed under subdivision. So named eases are sugar over beziers, `shift` emits raw beziers when it subdivides, and only `linear` and `hold` are closed under subdivision."

So when we split "ease-out" at t=1500, we get two bezier curves. The brief states these are:
- Left: `cubic-bezier(0.362866, 0, 0.693025, 0.369169)` (for the first half)
- Right: `(0.365106, 0.225535, 0.568419, 1)` (for the second half)

Under ENTERING convention:
- Keyframe at t=1500 gets ease: `cubic-bezier(0.362866, 0, 0.693025, 0.369169)`
- Keyframe at t=2000 (now shifted to t=4000) gets ease: `cubic-bezier(0.365106, 0.225535, 0.568419, 1)` (instead of "ease-out")

Actually wait. Let me re-examine. Under ENTERING, the ease on a keyframe describes the segment arriving AT that keyframe. So:
- The keyframe at t=2000 has ease="ease-out", meaning the segment arriving at t=2000 has ease-out.
- When we split at t=1500, the segment [1000, 2000] becomes two segments: [1000, 1500] and [1500, 2000] (after time adjustment).
- The segment [1000, 1500] arrives at the NEW keyframe at t=1500. It should get the left-half ease.
- The segment [1500, 2000] arrives at the EXISTING keyframe at t=2000 (which becomes t=4000 after shift). It should get the right-half ease.

So after shift:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 1500, "v": 150, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},
  {"t": 3500, "v": 150},
  {"t": 4000, "v": 200, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"},
  {"t": 5000, "v": 350, "ease": "ease-in-out"},
  {"t": 6000, "v": 400}
]
```

Wait, what about the keyframe at t=3500? When we insert [t=1500, t=3500], do we put any ease on t=3500?

Hmm, the segment [1500, 3500] exists because we're stretching the time. There's no motion there; both endpoints are at v=150. So there's no need for an ease. Under ENTERING, the ease on a keyframe describes the arriving segment, so if nothing arrives (no motion), no ease is needed.

But what about consistency? After the split, the next keyframe is at t=3500 with v=150. Then at t=4000 with v=200. The segment [3500, 4000] has motion from 150 to 200. Under ENTERING, the keyframe at t=4000 should have an ease describing [3500, 4000].

What was the original segment? We had [1500, 2000] going from 150 to 200 with the right-half ease. After stretch, it becomes [3500, 4000]. So the keyframe at t=4000 should have the right-half ease of the split ease-out.

So far so good. But I haven't included t=3500 in the ease calculation. Let me reconsider.

Actually, maybe I'm overcomplicating. Let me read more carefully what "insert keyframes at `at` and `at+delta` both carrying that value" means.

I think it means: insert two keyframes, both with the evaluated value. These keyframes mark the boundaries of the time gap. Under ENTERING, keyframes carry eases describing the segments arriving at them. If we insert two keyframes with the same value, they have no segment arriving if they're adjacent.

Actually, hang on. Let me reconsider the whole operation. The brief says:

> For a time-invariant straddler, evaluate the property at `at`, insert keyframes at `at` and `at+delta` both carrying that value, shift keys strictly after `at` by delta, and stretch `end`.

So we insert at `at` AND at `at+delta`. That's two insertions in original timeline space. Then we shift keys strictly after `at` by delta. Then we stretch end.

Original timeline: [... 1000 ... 1500 ... 2000 ... 3000 ... 4000]

Operation: shift(at=1500, delta=2000)

1. Evaluate at t=1500: v=150
2. Insert keyframe at t=1500 with v=150
3. Insert keyframe at t=1500+2000=3500 with v=150
4. Shift keys strictly after 1500 by 2000: t=2000→4000, t=3000→5000, t=4000→6000
5. Stretch end: if element was [0, 4000], it becomes [0, 6000]

Result timeline: [... 0 ... 1000 ... 1500 ... 3500 ... 4000 ... 5000 ... 6000]

OK so far we have keyframes at: 0, 1000, 1500, 3500, 4000, 5000, 6000

Now, the eases. The original segment [1000, 2000] with ease-out arrives at t=2000. After the operation:
- Segment [1000, 1500] arrives at t=1500. This is the left half of the split ease-out.
- Segment [1500, 3500] has no motion (both at v=150). So no ease.
- Segment [3500, 4000] is the right half of the original [1500, 2000], so it gets the right half of ease-out.

Similarly, the original segment [2000, 3000] with ease-in-out arrives at t=3000. After shift:
- t=2000 → t=4000 (shifted by 2000)
- t=3000 → t=5000 (shifted by 2000)
- Segment [4000, 5000] still has ease-in-out, so the keyframe at t=5000 gets ease-in-out.

And so on.

Alright, so under ENTERING after shift:
```json
[
  {"t": 0, "v": 100},
  {"t": 1000, "v": 100, "ease": "linear"},
  {"t": 1500, "v": 150, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},
  {"t": 3500, "v": 150},
  {"t": 4000, "v": 200, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"},
  {"t": 5000, "v": 350, "ease": "ease-in-out"},
  {"t": 6000, "v": 400}
]
```

Diff against original:
```json
[
  {"t": 0, "v": 100},              ← no change
  {"t": 1000, "v": 100, "ease": "linear"},  ← no change
  +  {"t": 1500, "v": 150, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},  NEW
  +  {"t": 3500, "v": 150},  NEW
  {"t": 2000, "v": 200, "ease": "ease-out"},  → {"t": 4000, "v": 200, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"}  CHANGED (t changed, ease changed)
  {"t": 3000, "v": 350, "ease": "ease-in-out"},  → {"t": 5000, "v": 350, "ease": "ease-in-out"}  CHANGED (t changed, ease same)
  {"t": 4000, "v": 400}  → {"t": 6000, "v": 400}  CHANGED (t changed)
]
```

So the changes are:
- 2 new records (at 1500 and 3500)
- 3 changed records (originally at 2000, 3000, 4000)
- Total changed records: 5

Of the changed records, how many have unchanged t and v?
- Record at t=1500: new, so doesn't count
- Record at t=3500: new, so doesn't count
- Record originally at t=2000 (now t=4000): t changed, v same, ease changed
- Record originally at t=3000 (now t=5000): t changed, v same, ease same
- Record originally at t=4000 (now t=6000): t changed, v same, ease doesn't change (it has no ease)

So all 3 non-new changed records have unchanged v. And all of them have changed t. So the count of "changed records with unchanged t and v" is 0.

Now let's do the same for LEAVING convention:

Original LEAVING:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "ease-out"},
  {"t": 2000, "v": 200, "ease": "ease-in-out"},
  {"t": 3000, "v": 350},
  {"t": 4000, "v": 400}
]
```

Under LEAVING, the ease on a keyframe describes the segment departing FROM that keyframe. So:
- t=0 has ease="linear" because the segment [0, 1000] departs with linear
- t=1000 has ease="ease-out" because the segment [1000, 2000] departs with ease-out
- t=2000 has ease="ease-in-out" because the segment [2000, 3000] departs with ease-in-out
- t=3000 has no ease because it's the last keyframe (nothing departs from it)
- t=4000 doesn't appear, but if it did, it would be the end

Wait, the file shows t=4000. So after my rewrite:
- t=4000 has no ease because there's no segment departing (it's the last in this example, or implicitly the end of animation)

OK so the segment [1000, 2000] departs from t=1000 with ease-out. At the split point t=1500:
- Segment [1000, 1500] departs from t=1000, so it uses the ease on t=1000, which is ease-out
- When we split at t=1500, the segment [1000, 1500] has the left-half ease-out, which should be... placed where?

Under LEAVING, the ease on a keyframe describes the segment departing from it. So the ease of the left-half-split should be on the keyframe at t=1000? No, that doesn't make sense; t=1000 already has its ease describing [1000, 1500] now (after the split).

Actually, wait. Under LEAVING, the ease on t=1000 describes [1000, 2000] before the split. After the split:
- [1000, 1500] should still depart from t=1000, so the ease on t=1000 should become the left-half ease
- [1500, 3500] departs from t=1500, so the ease on t=1500 should describe it (but there's no motion, so no ease needed)
- [3500, 4000] departs from t=3500, so the ease on t=3500 should describe it. This is the right-half ease-out (from the original [1500, 2000], which is now [3500, 4000] after shift).

After shift under LEAVING:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},
  {"t": 1500, "v": 150},
  {"t": 3500, "v": 150, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"},
  {"t": 4000, "v": 200, "ease": "ease-in-out"},
  {"t": 5000, "v": 350},
  {"t": 6000, "v": 400}
]
```

Diff against original:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},  → {"t": 0, "v": 100, "ease": "linear"}  NO CHANGE
  {"t": 1000, "v": 100, "ease": "ease-out"},  → {"t": 1000, "v": 100, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"}  CHANGED (t and v unchanged, ease changed)
  +  {"t": 1500, "v": 150}  NEW
  {"t": 2000, "v": 200, "ease": "ease-in-out"},  → {"t": 3500, "v": 150, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"}  CHANGED (t changed, v changed)
  ...
]
```

Hmm wait, this doesn't look right. Let me reconsider.

Actually, I think I've been confusing the motion. Let me re-examine.

Original file has 5 keyframes. Under LEAVING:
- t=0, v=100, ease="linear": segment [0,1000] departs with linear ease
- t=1000, v=100, ease="ease-out": segment [1000,2000] departs with ease-out
- t=2000, v=200, ease="ease-in-out": segment [2000,3000] departs with ease-in-out
- t=3000, v=350, (no ease): no segment departs
- t=4000, v=400: this is an extra keyframe at the end with no associated ease

Actually, wait. If t=4000 has no ease and represents the end state, then there's no motion from t=3000 to t=4000 under this interpretation? That doesn't seem right.

Let me reconsider. Maybe the last keyframe does have an implicit ease describing the segment departing from the second-to-last keyframe?

Actually, let me re-examine the original LEAVING convention definition:
> LEAVING: `ease` on keyframe K governs the segment departing K toward the next. The last keyframe in a list can carry no ease. This is the CSS `@keyframes` and Web Animations convention.

So the last keyframe "can carry no ease" because there's no segment departing from the last keyframe (there's no keyframe after it). But every other keyframe should have an ease describing the segment departing toward the next.

So under LEAVING for a 5-keyframe list:
```json
[
  {"t": 0, "v": 100, "ease": "ease_0"},
  {"t": 1000, "v": 100, "ease": "ease_1"},
  {"t": 2000, "v": 200, "ease": "ease_2"},
  {"t": 3000, "v": 350, "ease": "ease_3"},
  {"t": 4000, "v": 400}
]
```

- ease_0 describes [0, 1000]
- ease_1 describes [1000, 2000]
- ease_2 describes [2000, 3000]
- ease_3 describes [3000, 4000]
- No ease on the last keyframe at t=4000

So for my original example:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "ease-out"},
  {"t": 2000, "v": 200, "ease": "ease-in-out"},
  {"t": 3000, "v": 350, "ease": ???},
  {"t": 4000, "v": 400}
]
```

I should specify an ease for t=3000 that describes [3000, 4000]. Let me assume it's "ease-in" for variety.

```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "ease-out"},
  {"t": 2000, "v": 200, "ease": "ease-in-out"},
  {"t": 3000, "v": 350, "ease": "ease-in"},
  {"t": 4000, "v": 400}
]
```

Now shift at t=1500 with delta=2000 under LEAVING:

When we split the segment [1000, 2000] at t=1500:
- [1000, 1500] has ease-out (left half)
- [1500, 2000] has ease-out (right half)
- [1500, 3500] has no motion
- [3500, 4000] is the result of the original [1500, 2000] after shift

Under LEAVING:
- The segment [1000, 1500] departs from t=1000, so t=1000 gets the left-half ease-out (but it was already ease-out, just split)
- The segment [1500, 3500] departs from t=1500, so t=1500 gets... no ease (no motion)
- The segment [3500, 4000] is the right half of the original [1500, 2000], and it departs from t=3500, so t=3500 gets the right-half ease-out

And keys strictly after t=1500 shift by 2000:
- t=2000 → t=4000
- t=3000 → t=5000
- t=4000 → t=6000

After shift:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},
  {"t": 1500, "v": 150},
  {"t": 3500, "v": 150, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"},
  {"t": 4000, "v": 200, "ease": "ease-in-out"},
  {"t": 5000, "v": 350, "ease": "ease-in"},
  {"t": 6000, "v": 400}
]
```

Diff:
```
Line 0: NO CHANGE
Line 1: {"t": 1000, "v": 100, "ease": "ease-out"} → {"t": 1000, "v": 100, "ease": "cubic-bezier(...)"} 
        CHANGED but t and v UNCHANGED
Line 2: {"t": 2000, "v": 200, "ease": "ease-in-out"} → {"t": 3500, "v": 150, "ease": "cubic-bezier(...)"} 
        CHANGED, t CHANGED, v CHANGED (wait, why did v change?)
Line 3: {"t": 3000, "v": 350, "ease": "ease-in"} → {"t": 5000, "v": 350, "ease": "ease-in"} 
        CHANGED, t CHANGED, v UNCHANGED
Line 4: {"t": 4000, "v": 400} → {"t": 6000, "v": 400} 
        CHANGED, t CHANGED, v UNCHANGED
NEW Line: {"t": 1500, "v": 150}
```

Wait, why did v change from 200 to 150? Let me reconsider.

Oh, I see the issue. After the split at t=1500:
- Original keyframe at t=2000 is shifted to t=4000. But wait, that's not right.

Let me re-examine the operation. When we insert keyframes at t=1500 and t=3500, and then shift keys strictly after 1500, what happens to the keyframe originally at t=2000?

"shift keys strictly after `at` by delta"

So keys strictly after 1500 (i.e., t > 1500) shift by 2000. So t=2000 → t=4000, t=3000 → t=5000, t=4000 → t=6000. Yes, that's right.

But then the value at t=4000 after shift is the original value from t=2000, which is v=200. So it should be {"t": 4000, "v": 200, ...}.

Let me rewrite my result:

After shift under LEAVING:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},
  {"t": 1000, "v": 100, "ease": "cubic-bezier(0.362866, 0, 0.693025, 0.369169)"},
  {"t": 1500, "v": 150},
  {"t": 3500, "v": 150, "ease": "cubic-bezier(0.365106, 0.225535, 0.568419, 1)"},
  {"t": 4000, "v": 200, "ease": "ease-in-out"},
  {"t": 5000, "v": 350, "ease": "ease-in"},
  {"t": 6000, "v": 400}
]
```

Diff against original LEAVING:
```json
[
  {"t": 0, "v": 100, "ease": "linear"},  ← NO CHANGE
  -  {"t": 1000, "v": 100, "ease": "ease-out"},
  +  {"t": 1000, "v": 100, "ease": "cubic-bezier(...)"},  CHANGED, t and v UNCHANGED
  +  {"t": 1500, "v": 150},  NEW
  -  {"t": 2000, "v": 200, "ease": "ease-in-out"},
  +  {"t": 4000, "v": 200, "ease": "ease-in-out"},  CHANGED, ease unchanged, t and v changed
  ...
]
```

So the changed records are:
1. t=1000: ease changed, t unchanged, v unchanged ✓ (counts as "unchanged t and v")
2. t=1500: NEW
3. t=3500: NEW
4. t=2000→4000: t changed, v unchanged ✗ (t changed)
5. t=3000→5000: t changed, v unchanged ✗ (t changed)
6. t=4000→6000: t changed, v unchanged ✗ (t changed)

So under LEAVING, the count of changed records with unchanged t and v is 1 (the one at t=1000).

Under ENTERING, the count was 0.

This is the key insight! Under LEAVING, when we split and rewrite the ease on an existing keyframe, that keyframe's t and v stay the same. Under ENTERING, we create new keyframes with new t and v.

Now let me write this up properly in the exercise file.

OK, I've worked through this. Let me save it and work on the rest of Exercise 3 in the file I'll write.

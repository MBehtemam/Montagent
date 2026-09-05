# Exercise 1 — the three readings on the four hard cases
shift(at=20000, delta=2000) unless stated. Frame 1080x1920, fps 25.

## (a) eased segment (photo-06 with ease-in-out on the ramp)
scale: [{"t":17472,"v":1.0,"ease":"ease-in-out"},{"t":32472,"v":1.08}]
Eased value at 20000: x0=0.1685333, t*=0.1456073, y=0.0574303 -> v=1.004594 (NOT 1.013483)
- MOVE: [{17472,ease-in-out},{34472}] — same single bezier stretched over 17000ms.
  Frames before at change (curve renormalised); the whole shot re-times. BREAKS identity-before-at.
- HOLD: keys untouched; post-insert content not translated. BREAKS translation-after-at.
- SPLIT: must subdivide the bezier. De Casteljau at t*:
  left  = cubic-bezier(0.362866, 0, 0.693025, 0.369169)
  right = cubic-bezier(0.365106, 0.225535, 0.568419, 1)
  Neither is any named ease. SPLIT survives ONLY if raw beziers are writable. Otherwise all
  three readings are wrong on an eased segment.
  Result: [{"t":17472,"v":1.0,"ease":[0.362866,0,0.693025,0.369169]},
           {"t":20000,"v":1.004594,"ease":"linear"},
           {"t":22000,"v":1.004594,"ease":[0.365106,0.225535,0.568419,1]},
           {"t":34472,"v":1.08}]

## (b) at exactly on an existing keyframe
Element 10000-40000, scale [{10000,1.0},{20000,1.05},{30000,1.02},{40000,1.0}], shift(at=20000).
- MOVE ("every key >= at"): [{10000,1.0},{22000,1.05},{32000,1.02},{42000,1.0}] — segment
  10000->22000 stretched: frames in [10000,20000) change. Same before-at breakage.
- HOLD: post-at keys stale. Broken as always for straddlers.
- SPLIT naive ("insert at & at+delta, then move keys >= at"): evaluates v(20000)=1.05,
  inserts {20000,1.05},{22000,1.05}, then moves the EXISTING key 20000 -> 22000: two keys at
  t=22000 (same v here; different ease roles). Naive SPLIT emits a duplicate. Fix: move keys
  STRICTLY > at, then ensure {at,v} and {at+delta,v} exist (the at-key keeps its incoming
  ease; the at+delta key carries the original key's outgoing ease). With the fix:
  [{10000,1.0},{20000,1.05},{22000,1.05},{32000,1.02},{42000,1.0}] — output-identical. Clean.

## (c) element entirely after at (photo-07: 30603-42763, scale [{30603,1.0},{45603,1.08}])
Whole element translates: start/end/ALL keys +2000 -> 32603-44763, [{32603,1.0},{47603,1.08}].
- MOVE (global "every time >= at") gets this right BY LUCK: all its times are >= at.
- HOLD read as "never touch keyframes" is catastrophic here (Sonnet A's live bug: stale ramps).
  HOLD is only coherent as a straddler-scoped rule.
- SPLIT applied naively would evaluate v(20000)=clamp=1.0 and insert junk keys before the
  element's start. SPLIT must be scoped: inserts happen only when at falls strictly inside a
  straddler's active (value-changing) segment. Rigid translation otherwise.
=> The real rule is per-ELEMENT, not per-timestamp. Keyframes are anchored to their element.

## (d) trimmed move: keyframe past its own end
photo-05: 3018-17472, key at 18018 (past end). Take shift(at=18000, delta=2000):
element end 17472 <= 18000 -> element untouched. But 18018 >= at, so a global
"every time >= at" rule (MOVE) drags the out-of-range key to 20018: the ramp of an
UNTOUCHED element silently re-times (15000ms -> 17000ms of ramp). Nothing on screen moves
today, but the [start, start+15000] invariant is broken and any later un-trim renders wrong.
Element-anchored rule: element untouched => keys untouched, 18018 stays. Correct.
Conversely photo-05-quiz (53856-64016, key 68856) under shift(at=20000): element fully after
=> rigid translation: 55856-66016, keys [{55856,1.0},{70856,1.08}] — the +15000 invariant
survives, the trimmed move stays a trimmed move.

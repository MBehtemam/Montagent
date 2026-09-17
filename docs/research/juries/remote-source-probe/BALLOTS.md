# Court: remote (URL) source probe/cache design

One round, three jurors (Claude Opus 5, Claude Haiku 4.5, Claude Fable 5.1), independent,
blind to each other's ballots. Convened for [#127](https://github.com/MBehtemam/Montaget/issues/127)
to answer five sub-questions on how `validate`/`render` handle a `source` that is a URL
(permitted by ADR-0002), given ADR-0006's "always probe, no fast mode" policy was measured
only against local files (~0.04s each) and ADR-0053 deferred all remote-specific failure
modes to this ticket.

## The Question, as put to every juror (verbatim)

**Q1 — Does the "always probe, no fast mode" policy extend to remote sources unmodified, or
does remote cost force a different cadence?**
Options: (a) probe every remote source on every `validate` run, same as local, accepting the
latency; (b) probe once per process/session and reuse the result for the rest of that run;
(c) something else (specify).

**Q2 — What does "probing" a remote source actually fetch, and what does it check against the
document?**
Options: full/partial content fetch (e.g. an HTTP range request feeding something like
`ffprobe`) to verify real duration/dimensions, vs. a cheap existence-only check (HTTP HEAD,
status code only) with no duration verification, vs. attempt the former and gracefully
degrade to the latter when not feasible for a given URL/protocol (and say so explicitly in
the finding).

**Q3 — What is the cache key for a remote source, given no mtime/size?**
Options: (a) ETag (if the server sends one); (b) `Last-Modified` header; (c) no persistent
cache at all for remote sources — re-probe fresh every `validate` invocation, cached only
within a session (per your Q1 answer); (d) some fallback chain combining the above.

**Q4 — Does "couldn't verify" (e.g. request timed out, host unreachable, DNS failure — NOT
evidence the file is missing, just evidence nothing was learned) get its own new severity
tier, or reuse the existing `UNCHECKED`/`NOT CHECKED` category ADR-0006 already built for
"we did not check this"?**

**Q5 — Given your Q4 answer, does `render` refuse to proceed when a remote source's status is
`UNCHECKED` (couldn't verify), or only on a confirmed `error` (definitively
missing/inconsistent)?**
Options: (a) refuse on `UNCHECKED` too, treating unverifiable as unsafe; (b) proceed —
`render` makes its own fetch attempt independently and fails loudly and specifically if that
fails, rather than trusting a possibly-stale earlier network blip; (c) something conditional
(specify).

## Ballots

### Juror 1 — Claude Opus 5

**Q1 VOTE:** (b) probe once per process/session and reuse for the rest of that run

**Q2 VOTE:** attempt real duration/dimension probe (range request → ffprobe), gracefully
degrade to existence-only (HEAD/status) when not feasible, and say so explicitly in the
finding

**Q3 VOTE:** (c) no persistent cache for remote sources — fresh per `validate` invocation,
cached only within the session

**Q4 VOTE:** reuse the existing `UNCHECKED`/`NOT CHECKED` category from ADR-0006 — no new
severity tier

**Q5 VOTE:** (b) `render` proceeds past `UNCHECKED` and makes its own fetch attempt, failing
loudly and specifically if that fails

**Reasoning:** ADR-0006's "always probe, no fast mode" was never a statement about probing
per se — it was a statement that a ~0.04s cost buys certainty and therefore no user should
ever be offered a knob that trades certainty away. The invariant being protected is *no stale
facts, no hidden modes*, not *maximum syscalls*. Q1(b) preserves that invariant exactly:
within a single `validate` run every remote source is probed, freshly, from the live network;
the session cache only prevents probing the *same URL* twice in one run, which is
deduplication, not caching, and cannot yield a stale answer because the run itself is the
unit of time. A per-URL flag would be a fast mode; per-run freshness is not. Rejecting (a) as
literally-every-reference is not a softening — it is what "probe every source" already means
for local files too.

Q3 falls out of Q1. ETag and `Last-Modified` are attractive because they look like mtime, but
they are *server assertions*, not observations: a server may omit them, lie, rotate them on
unchanged bytes, or keep them fixed across changed bytes, and CDNs routinely do all four.
Persisting them would put network-derived, unverifiable state into the project's durable
footprint, which is exactly the hidden-state the inert-data invariant forbids — a reader of
the file could no longer understand the project by reading it, because some of the truth
would live in a cache keyed on a header nobody can see. No persistent remote cache keeps the
ledger honest: everything validate says about a remote source was learned this run or is
marked as not learned at all.

Q2 is where the no-silent-false-confidence invariant does its real work. An existence-only
check is a materially weaker claim than a local probe — it proves a route and a status code,
not that the bytes are a 12.4s video at 1920×1080. If existence-only silently occupied the
same slot in the report as a full local probe, a clean run would overstate itself precisely
in the case where the author is least able to check by hand. So: try for the real thing (a
range request feeding ffprobe gets duration and dimensions from most well-behaved HTTP media
without downloading the file), and when the URL or protocol won't support it, degrade — but
the degradation must be *legible in the finding itself*, phrased as what was and wasn't
established. The finding is then a fact about knowledge, which is what ADR-0006 said validate
traffics in.

Q4 needs no new tier because ADR-0006 already built this exact category for this exact
reason, and it fits without strain. "Request timed out" is epistemically identical to "we did
not check this": no evidence about the file was obtained. Minting a second tier for
network-flavoured unknowing would imply that unknowing-by-timeout is more or less alarming
than unknowing-by-not-looking, which is a claim nobody can support — and it would fragment a
category whose whole value is that one glance at a report separates confirmed-good from
not-confirmed. ADR-0053 already fixed the other half: *confirmed* missing is a plain `error`,
local or remote, no softening. A 404 or a resolved-host-refused-the-object is confirmed
missing and stays an `error`; a timeout is not evidence of absence and must not be laundered
into one. The whole remote story is therefore expressible in categories that already exist,
which is the strongest possible sign the existing taxonomy was cut correctly.

Q5 follows from Q4 plus validate-reports-render-enforces. `render` enforces on *facts*, and
`UNCHECKED` is the explicit absence of a fact — enforcing on it would be enforcing on
ignorance. Worse, it would make render's success contingent on a network condition sampled at
some earlier, arbitrary moment: a thirty-second outage during `validate` would block a render
attempted an hour later over a healthy link, while the reverse (verified at noon, link dead
at one) would pass the gate and fail anyway. The gate would be both false-positive and
false-negative, which is the signature of a gate reading the wrong signal. Render must fetch
the bytes regardless — it cannot render without them — so it holds the only authoritative,
correctly-timed observation. Let it fail there, loudly, naming the URL and the network error,
and the failure is accurate by construction. This also keeps render from ever refusing on a
stale blip, which would be the most maddening possible failure mode for an agent-driven tool:
a re-run of validate would "fix" the render, teaching users to re-run until green — precisely
the ritual ADR-0006's no-fast-mode rule exists to prevent.

**Trade-offs:** Q1(b) costs a genuine latency floor: a project with forty distinct remote
sources pays forty network round-trips per `validate`, and validate stops being instant. That
is the price of the report meaning something, and it is bounded by distinct URLs rather than
references. Concurrency and a bounded per-probe timeout (after which: `UNCHECKED`, not a
hang) make it tolerable without introducing a mode. (a) in its strict reading wastes
redundant probes for no epistemic gain; anything laxer is the fast mode ADR-0006 refused.

Q2's degradation path costs report complexity — two grades of remote verification a reader
must distinguish — and range-request probing is fiddly against servers that ignore `Range` or
sit behind redirects. The alternative, existence-only for everything, is cheaper and simpler
but buys a report that cannot catch the single most common real defect (a URL that resolves
but whose media is a different length than the document assumes), while *looking* like it
did. Full-download probing is off the table on cost alone.

Q3(c) costs repeated work across runs — the same unchanged CDN asset re-probed every
invocation, forever. Against that: any persistent remote cache is a correctness hazard that
fails silently and asymmetrically (it goes wrong by reporting stale confidence, never by
reporting stale doubt), and it plants durable machine state beside a file whose whole premise
is being readable as truth. A fallback chain (d) merely compounds several unreliable signals
into one confidently-wrong answer. If remote re-probing ever becomes a measured pain, the
honest fix is a scoped, explicit, separately-decided mechanism — not a header-keyed cache
smuggled in now, ahead of any evidence, against a fixture set that contains zero remote
sources.

Q5(b) costs a later failure: the author learns at render time, having waited, rather than at
validate time. That is the correct trade because the alternative buys no earlier certainty —
only earlier *guessing* — and because validate still surfaces the `UNCHECKED` finding
prominently, so a careful author is warned before they start the render. The conditional
option (c) was tempting for long renders (fail-fast before an hour of encoding), but render
fetching remote sources up front, as its own first step, delivers that fail-fast benefit from
a live observation rather than by gating on validate's stale one — same protection, no
wrong-signal gate, no new coupling between the two commands.

### Juror 2 — Claude Haiku 4.5

**Q1 VOTE:** (b) probe once per process/session and reuse the result for the rest of that run

**Q2 VOTE:** Attempt full verification (HTTP range request + metadata extraction for real
duration/dimensions), but gracefully degrade to existence-only check (HTTP HEAD status code)
when full verification isn't feasible for a given URL/protocol, explicitly reporting the
limitation in the finding

**Q3 VOTE:** (d) Fallback chain: try ETag first (server's most reliable change signal), fall
back to Last-Modified header (better than nothing), revert to session-only caching with no
persistent cache across invocations if neither is available

**Q4 VOTE:** Reuse the existing `UNCHECKED`/`NOT CHECKED` category for network verification
failures (timeouts, unreachable hosts, DNS failures)

**Q5 VOTE:** (b) Proceed — `render` makes its own fetch attempt independently and fails
loudly and specifically if that fails, rather than trusting a possibly-stale earlier network
blip

**Reasoning:** Remote probes differ fundamentally from local ones (~0.04s cheap vs.
slow/flaky network). Session-scoped probing honors "always verify" within a single invocation
without forcing network on every `validate` call. Graceful degradation (full → existence
check) respects "no silent false confidence" by reporting what was actually verified. HTTP
standard cache headers (ETag preferred) respect "inert data" while session-only fallback
acknowledges that unreliable remotes can't anchor persistent proofs. UNCHECKED reuses
ADR-0006's category for "did not verify" cases. Finally, `render`'s independent fetch
respects validate-reports-render-enforces separation: validate reports network-dependent
facts ("we learned X"), render makes its own attempt when it actually needs the file, and
fails specifically if that attempt fails — never trusting validate's ephemeral network state
as a proxy for render-time conditions.

**Trade-offs:** Session-scoped probing means multiple `validate` calls re-probe remotes
(waste), but avoids forcing network on every user invocation and matches local practice
(cheap per-session verification). Full-before-existence degrades gracefully but adds fetch
latency for supported URLs. ETag fallback chain requires servers to be well-behaved; missing
headers mean no persistent caching and re-probing on every invocation. UNCHECKED for network
failures avoids a new severity tier (simpler) but doesn't distinguish network blips from
intentional non-verification in the finding itself (mitigated by explicit reporting of probe
attempts and their failures). Render's independent fetch means a file validated as UNCHECKED
might still fail at render time (less early feedback), but aligns with principle that
`validate` cannot guarantee render-time network state and that render must be self-reliant.

### Juror 3 — Claude Fable 5.1

**Q1 VOTE:** (b) probe once per process/session and reuse the result for the rest of that run

**Q2 VOTE:** attempt content probe (HTTP range → `ffprobe`) and gracefully degrade to
existence-only (HEAD), with the finding explicitly stating which one actually happened

**Q3 VOTE:** (d) fallback chain: strong ETag → `Last-Modified` + `Content-Length` → no
persistent cache (session-only)

**Q4 VOTE:** reuse the existing `UNCHECKED`/`NOT CHECKED` category, with a mandatory reason
sub-field (timeout / DNS / unreachable / HTTP status), no new tier

**Q5 VOTE:** (b) proceed — `render` makes its own independent fetch and fails loudly and
specifically if it fails

**Reasoning:** ADR-0006's "no fast mode" was a decision about *intent* (validate always tries
to learn the truth), justified by a *measurement* (0.04s). The intent transfers to remote
sources; the measurement does not. So the right reading is: validate still attempts every
remote probe on every invocation — there is no flag to skip it — but a single run must not
re-hit the same URL once per element that references it, which is what (b) really means.
Multiple elements sharing one URL is the common case (one B-roll clip used five times), and
re-fetching five times within one run teaches nothing new. Between runs, the policy is
"attempt fresh unless the persistent cache can prove nothing changed" — which is exactly what
Q3's chain does: a strong ETag is a content identity, so a conditional GET (`If-None-Match`)
returning 304 is a cheap, honest revalidation that costs one round-trip; `Last-Modified`
+`Content-Length` is the weaker local analogue to `(size, mtime)`; and if the server offers
neither, there is nothing trustworthy to key on, so the cache is session-only. Note that in
every branch validate still contacts the server on every run; the cache only decides whether
the expensive range-fetch-and-ffprobe is repeated, never whether the check happens. That is
the same shape as the local cache and keeps the invariant that a clean report reflects this
run's reality.

Q2 follows from "validate reports facts": the fact the document cares about is
duration/dimensions, not "something answered 200". A HEAD-only check cannot catch a document
that declares 12s for a clip that is 9s long — precisely the class of defect ADR-0006 built
the probe for. But some URLs will not support range requests, some servers reject HEAD, and
some protocols have no cheap partial read; there the honest behaviour is to do what can be
done and *say what was not done*. A finding of "existence confirmed (HEAD 200), duration NOT
CHECKED — server does not support range requests" is a fact. A silent downgrade would be
false confidence.

Q4: "couldn't verify" is semantically identical to what `UNCHECKED` was built for — the
absence of knowledge, not the presence of a defect. Inventing a new tier would fragment the
report vocabulary and invite consumers (agents especially) to treat "network-unchecked"
differently from "unchecked" when they should not. What is genuinely new is the *reason*, and
that belongs in a structured sub-field on the existing finding, not in a new severity.
ADR-0053 already decided that a *confirmed* missing remote (definitive 404/410, or a probe
that succeeds and disagrees with the document) is a plain `error`; the boundary between the
two is "did we learn something" and the two categories already draw exactly that line.

Q5: refusing on `UNCHECKED` would make `validate` a blocker, which contradicts "validate
reports, render enforces" and would let a transient blip at validate-time veto a render that
would have succeeded. Render is the enforcer, and the enforcement point for a network resource
is the moment render needs the bytes. So render fetches independently; if it fails it fails
loudly with the specific URL and cause, not with "validate said unchecked". Conversely, if
validate happened to succeed and the file has since vanished, render would still fail
correctly because it does its own fetch — a stale earlier success is no more trusted than a
stale earlier failure.

**Trade-offs:** Q1 (b) over (a): the cost is that a URL that changes mid-run is not noticed;
this is the same acceptable blind spot as local mtime caching and buys predictable run time.
Over (c)-style skip flags: rejected because it would reintroduce the fast mode ADR-0006
deliberately closed. Q2: content probing costs a range fetch per remote source (typically a
few hundred KB for a moov atom, but MP4s with trailing moov may need two ranges or a larger
read) and adds latency proportional to remote count; HEAD-only would be cheap but would make
remote sources second-class citizens whose duration claims are never verified, which is
silent false confidence. Q3 (d) over ETag-only: many CDNs and object stores emit weak or no
ETags, so a single-key scheme would leave most URLs uncached; the chain's cost is more code
and the need to record *which* key was used in the cache entry. Over (c)-only: re-probing
content on every run for a stable 2GB asset is wasteful when the server can prove it is
unchanged in one round-trip. Q4 reuse over new tier: the cost is that consumers must read the
reason field to distinguish "we never tried" from "we tried and the network failed"; a new
tier would make that distinction top-level but at the price of a growing severity vocabulary
and a precedent for per-cause tiers. Q5 (b) over (a): the cost is a render that starts and
then fails partway on a network error, wasting whatever work preceded the fetch — mitigated
by having render resolve all remote sources up front before any encoding begins. Over (c)
conditional variants (e.g. refuse only if the UNCHECKED is "recent"): rejected because they
require render to reason about validate's timing, which reintroduces hidden state and blurs
the reports/enforces boundary.

## Verdict

Unanimous 3/3 on Q1, Q2, Q4 and Q5. Split on Q3: Opus votes no persistent cache at all
(session-only); Haiku and Fable vote a fallback chain (ETag → Last-Modified → session-only).

The map author (judge) resolved the Q3 split in favour of Opus's no-persistent-cache
position — ADR-0056 records the reasoning. Fable's Q1 clarification (the session cache
dedupes by *URL*, not by run — the server is still contacted every `validate` invocation, so
this was never a fast mode) and Fable's Q4 reason-sub-field refinement are adopted regardless
of the Q3 outcome, since no juror's Q3 vote depended on them.

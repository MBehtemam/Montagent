---
status: accepted
amends: 0120 (its rule is narrowed to refusals that are intent forks, and a message naming the complete set of legal options or the refusal's own consent handle does not break it; four of its §5 conflicts stand unchanged, and `E-SCHEMA-UNKNOWN-KEY` drops its list of published keys), 0043 (states that `shift`'s `release` is not a bypass of the no-override guarantee, and what a consent handle must be so no future flag can borrow the name)
---

# The one-branch rule holds on intent forks, and `release` is not a bypass

**Ticket:** [#504](https://github.com/MBehtemam/Montagent/issues/504), resolving the five
conflicts ADR-0120's §5 audit found. The rulings are in
[its resolution](https://github.com/MBehtemam/Montagent/issues/504).

## What ADR-0120 left open

ADR-0120 dropped `E-FONT-NO-GLYPH`'s list of covering fonts and ratified a rule: *a fact that
bears on only one branch of a refuse fork, or names something the author could adopt, is a
repair by another name.* Its audit found five other refuse-class messages that break the
letter of that rule. It left them unchanged and did not claim them. The question was whether
the messages were wrong or the rule was too broad.

Reading the history behind each message settled most of it. **Four of the five name what they
name on purpose, and two of those rest on measured agent evidence:**

- `E-SHIFT-STRADDLE`'s *"nearest legal boundaries"* was commissioned by ADR-0005 after all three
  agents found an edit ill-posed and two shipped a pause 3.3 s late. ADR-0005 calls it *"the
  decision's most valuable output."*
- `E-SHIFT-SLACK`'s `release: [[from, to]]` is what ADR-0047 built `release` to consume:
  *"precisely the pairs that message prints."*
- `E-FONT-BLOCKLISTED`'s substitutes were settled by ADR-0057 after a 2–1 panel split in which
  the majority wanted to auto-vendor one.
- `E-RETIRED-KEY`'s replacement was made a message-text property on purpose by ADR-0016 and
  ADR-0068.

## Decision

### 1. The rule is narrowed to intent forks

> On a refuse-class finding whose refusal is an **intent fork**, a fact that bears on only one
> branch, or names a concrete candidate the author could adopt for one branch, is a repair by
> another name, and the finding does not carry it. A message does not break the rule by naming
> **the complete set of legal options, unranked and covering every branch**, or **the refusal's
> own consent handle**. A census that partitions the siblings without ranking them is not a
> repair.

ADR-0043's evidence, which is what the rule rests on, is about intent forks: the tool leans
toward one branch, and the agent ships that branch. The rule has no grounds where the refusal
is not a fork over what the author meant, or where what is named does not favour a branch.

**The two exemptions are written tightly on purpose.** All three jurors raised it
independently: a loosely worded "complete set" becomes the loophole for one-branch hints. The
set must be unranked and cover every branch. A consent handle names **the specific fact the
finding reported** and is refused when it does not match.

| Code | Ruling | Why |
| --- | --- | --- |
| `E-FONT-BLOCKLISTED` | stands | The refusal is licence law, not an intent fork. The only legal branch is *don't embed this font*, so the substitutes have no fork to tilt. |
| `E-RETIRED-KEY` | stands | The replacement is where every branch of the fork ends up, and the fork is over values. **Caveat:** this rests on the fork's structure, not on evidence (§3). Revisit it if an experiment shows naming the replacement induces naive repairs. |
| `E-SHIFT-STRADDLE` | stands | Both nearest boundaries: the complete set, unranked. |
| `E-SHIFT-SLACK` | stands | `release` is its own consent handle (§4). |
| `E-SCHEMA-UNKNOWN-KEY` | **changes** (§2) | A real intent fork, and a list of candidates for one branch. |
| `E-FONT-NO-GLYPH` (ADR-0120) | still dropped | A real intent fork, and a list of candidates for one branch. |

### 2. `E-SCHEMA-UNKNOWN-KEY` no longer lists the keys the format publishes

Its message ended *"Here the format publishes {expected}"*: `serde`'s full list of valid keys
at that position, carried as a JSON `expected` field as well. The fork is a **typo** or **a key
from a newer format** that this binary predates. The list serves the typo branch only. On the
other branch it is a menu for the silent rename that the message's own *"Do not delete the
key to make the file validate"* exists to prevent, because renaming the key destroys it just
as surely as deleting it. The text clause and the JSON field are both dropped, since the rule
is about what the finding carries and an agent reading `--json` reads the field.

ADR-0016's arms measured 0/9 deletions without the list. No arm tested listing the expected
keys, so dropping the list is backed by the rule and by ADR-0043's direction, not by a
measurement of its own. If anyone wants the list back, the burden is theirs to measure. **It
was pinned on purpose before:** `tests/schema_check.rs` asserted the list as *"what turns a
typo back into the key it was copied from"*. That is the typo-branch help this ruling removes,
and the test now asserts that the list is absent.

### 3. A correction to ADR-0120

ADR-0120's §5 said ADR-0043's gravity experiment *"was run on such a message"*, meaning one
naming the replacement. The exact error text the agents saw is not preserved in
`docs/research/juries/format-versioning/experiment-gravity-fork/`. The glossary excerpt and
one agent's report suggest it named `x`/`y`/`origin`/`clip`, but no more than that. The
sentence in ADR-0120 is corrected in place to say what the record supports.

### 4. `release` is not a bypass

ADR-0043 guarantees that *"a refuse-class `error` has no override."* `release` does not break
that guarantee. A bypass lifts a finding and leaves the edit unchanged. `release` changes the
edit: a call carrying `release: [[from, to]]` is a different request, one that names the exact
slack the finding reported. It is refused when it names a pair that does not bound a real
protected slack this edit would change (ADR-0047), and it has no bulk form. Every call that
does not name the pair is still refused.

**Recorded so that no future `--force` can borrow `release`'s name.** A mechanism is a consent
handle only if it names the specific fact the finding reported and is refused when it does not
match. A flag that lifts a finding without naming what it consents to is a bypass, and
ADR-0043 forbids it.

**The dissent, and the tension it named.** One juror of three proposed reclassifying
`E-SHIFT-SLACK` as advise-class. That was not adopted: whether a slack should change is the
author's intent, and *don't release, change the edit* is an equally valid fix, which makes it
an intent fork and so refuse-class. But the tension the dissent named is real. A refuse-class
finding tells the agent to *stop and surface it to a human*, while this message tells the
agent exactly how to continue by itself. **Whether an agent may type `release` on its own
authority is a question about ADR-0047's consent model**, not a class label, and it is left
to [#506](https://github.com/MBehtemam/Montagent/issues/506), settled by
[ADR-0124](./0124-release-carries-the-requester-s-consent-and-surfacing-is-the-default.md).

## Consequences

- **Breaking: `E-SCHEMA-UNKNOWN-KEY`'s JSON loses `expected`, and its text loses its last
  sentence.** A consumer that read the list for spelling suggestions gets nothing. No changelog
  exists to bump, so this note is the version record.
- **Four messages ADR-0120 put in doubt are ratified unchanged.**
- **`CONTEXT.md`**'s **Repair** entry carries the narrowed rule.
- **ADR-0120's rule stands as amended here.** A new refuse-class check is written against §1.

## Evidence

A three-juror court (Claude Opus 5.5, Sonnet 5.5, Fable 5.1) voted unanimously on the narrowing
and on dropping the list, and 2–1 on `release`. The human ratified it. Ballots are in
[#504](https://github.com/MBehtemam/Montagent/issues/504).
`tests/schema_check.rs`'s `an_unknown_key_names_no_key_to_rename_it_to` and the updated
`the_unknown_key_finding_reads_as_adr_0016_wrote_it` both fail against the previous
`checks/schema.rs` and `registry.rs`.

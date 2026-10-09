---
status: accepted
amends: 0189 (CI enforces the scrub budget on the best of three cold attempts on every tested leg except x86_64 Windows, where the test is ignored until the preview's cost there is understood)
---

# The scrub-budget test is not run on x64 Windows until its cost there is understood

The owner told the agent to disable the test on the leg that fails.

[ADR-0189](0189-ci-enforces-the-scrub-budget-on-the-best-of-three-cold-attempts.md) takes the
best of three cold attempts and says *"x64 Windows may still miss; a per-platform allowance is a
separate owner decision not taken"*. It still missed: on #871 the x86_64 Windows leg failed
`preview_budget` while the other four tested legs passed.

## Decision

- **`a_ten_second_scrub_preview_of_the_fixture_stays_inside_the_budget` is ignored on
  x86_64 Windows**, with the reason in the `ignore` attribute so it shows in the test output.
  It runs unchanged on aarch64 and x86_64 Linux, aarch64 macOS and aarch64 Windows.
- **The number does not move.** `SCRUB_PREVIEW_LIMIT` stays 5 s (ADR-0021, ADR-0065), and the
  verb is not changed. No allowance is given to any platform.
- **The decision-logic tests in the same file still run on every leg.** They do not read a
  clock.

## Why

A test that fails on one leg on every run teaches people to ignore red. Raising the budget there
would be a claim about what a Windows user should expect, and nothing measured supports one.
Ignoring the test says only that it is not checked there for now.

## What it costs

Nothing on that leg guards the scrub preview against a slowdown. The other four legs still do,
and a regression in the preview's cost shows on them.

## What would end it

Finding out why the preview takes about 5 s on x64 Windows (for example whether process start-up
is outside the verb's own clock), then either fixing it or taking an owner decision on an
allowance, and removing the `ignore`.

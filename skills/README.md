# Montagent skills

Agent skills for driving Montagent from any Agent-Skills-compatible client. Install them with:

```sh
npx skills add MBehtemam/Montagent
```

Start with `montagent`, the router skill. It points to the job skills: `montagent-motion`, `montagent-footage`, `montagent-character` and `montagent-craft`.

## Experimental

These skills are experimental. A pre-registered, blind-judged eval on held-out briefs found **parity but no lift**:

- **Parity passed.** Opus 5.5 using Montagent with the skills was judged as good as Opus 5.5 working without Montagent. The no-Montagent reference won 2 of 9 pairs, against a limit of 40%.
- **Lift failed.** Montagent with the skills was not judged better than Montagent without them. With the skills won 4 pairs, without them won 6, and 17 were equal.

See [Verdict runs: every arm on the held-out briefs, judged blind](https://github.com/MBehtemam/Montagent/issues/549). A second verdict on new held-out briefs ran every arm but was closed as inconclusive, so the lift claim is dropped: the skills make no claim to produce better videos than Montagent without them. See [Second verdict runs: every arm on the new briefs, judged blind](https://github.com/MBehtemam/Montagent/issues/582).

The skills describe the main branch. If your installed `montagent` binary is older, trust its findings over the skills.

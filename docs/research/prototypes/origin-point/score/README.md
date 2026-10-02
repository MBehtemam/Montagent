# Score: the free pivot against the baker (#598)

Pre-registered on [#598](https://github.com/MBehtemam/Montagent/issues/598#issuecomment-5959024873) before any run. Pin: `c1cafe02`.

## Owl: "arm swings later", arm B

- **Runs:** `skills-eval/runs/dev/C-edit-arm/with-skills-{4,5,6}`, seeded from `../seeds/C-unpadded`.
- **Arm A** is #593's runs `with-skills-{1,2,3}`, reused rather than re-run.
- **Score:** `EDIT_SCORE_SEED=docs/research/prototypes/origin-point/seeds/C-unpadded/workspace/hoot.montagent.json python3 docs/research/skills-eval/edit_score.py <runs>`.
- **Validate** each kept workspace copied over `skills-eval/assets/`. The kept workspace alone lacks the pack's images, so on its own it raises `E-SOURCE-MISSING`.

## #299: the Ken Burns pivot, hand-authored

```sh
python3 kb_spellings.py fixtures/en-halloween-decorating   # writes kb-armA, kb-armA-int, kb-armB beside the fixture
uv run kb_mad.py fixtures/en-halloween-decorating <montagent at c1cafe02> <out dir>
```

- **Arm A:** `x`/`y` keyframes that compensate along the scale ramp, `(1 - s)(P - C)`. Fractional positions are `E-SCHEMA`, so the kept spelling is `kb-armA-int`.
- **Arm B:** `"origin": [434, 920]`, with `x`/`y` moved to keep the rest position.

---
status: accepted
amends: 0016 (ratifies E-SCHEMA-UNKNOWN-KEY as a separate code from E-SCHEMA and suppresses both on keys and values claimed by checks::retired), 0017 (ratifies the two-code schema error split across every closed object level), 0043 (ratifies that E-SCHEMA and E-SCHEMA-UNKNOWN-KEY are refuse-class findings with census: None)
---

# Ratify `E-SCHEMA` and `E-SCHEMA-UNKNOWN-KEY`'s two-code split, retirement suppression, and `census: None`

[#253](https://github.com/MBehtemam/Montagent/issues/253).
When `checks/schema.rs` and `registry.rs` implemented the closed-schema and refuse-class rules from
[ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md),
[ADR-0017](0017-closed-schema-no-escape-hatch.md), and
[ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md), three
design choices were recorded in code comments citing `#253`:

1. Splitting unknown-key findings (`E-SCHEMA-UNKNOWN-KEY`) from wrong-type, missing-field, and
   invalid-value schema findings (`E-SCHEMA`).
2. Suppressing `E-SCHEMA-UNKNOWN-KEY` and `E-SCHEMA` whenever a key or value is owned by
   `checks::retired::named` (`E-RETIRED` or `E-MASK-BARE`).
3. Setting `census: None` on both `E-SCHEMA` and `E-SCHEMA-UNKNOWN-KEY` even though both are
   `RepairForm::Refuse`.

This ADR ratifies all three decisions.

---

## Decisions

### 1. Two schema error codes: `E-SCHEMA-UNKNOWN-KEY` and `E-SCHEMA` (amends [ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md), [ADR-0017](0017-closed-schema-no-escape-hatch.md))

- **`E-SCHEMA-UNKNOWN-KEY`** fires when an object at any closed level (project root, `frame`,
  tracks, elements, text runs, keyframes, `t_from`, `clip`, `layer` anchors, `effects`,
  `audio_effects`, `transitions`, `markers`, or `master`) contains a key the schema has never
  defined. Its prose carries ADR-0016's forward-compatibility warning (*"It may belong to a newer
  format revision than this binary implements… Do not delete the key to make the file validate"*).
- **`E-SCHEMA`** fires when a known field is missing, has the wrong JSON type, is written as `null`
  ([ADR-0247](0247-optional-fields-refuse-explicit-null-across-the-project-format.md)), or carries a
  value outside its closed grammar or bounds. Separating the two codes keeps ADR-0016's
  version-skew warning OFF ordinary type and value mistakes, where blaming the binary version would
  mislead the author.

### 2. Retirement suppression across keys and values (amends [ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md))

- When a key or field value is claimed by `checks::retired::named` (`gravity`, `box`, `align` on a
  non-text element, `anchor` as a string, `bold`, `fit: "none"`/`"fill"`, `overrun: "none"`, or bare
  `mask` under `E-MASK-BARE`), `checks::schema` yields to `checks::retired` and emits neither
  `E-SCHEMA-UNKNOWN-KEY` nor `E-SCHEMA` for that entry. Each retired spelling produces exactly one
  finding naming its replacement rather than a duplicate schema error with conflicting guidance.

### 3. `RepairForm::Refuse` with `census: None` on schema findings (amends [ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))

- Both `E-SCHEMA` and `E-SCHEMA-UNKNOWN-KEY` are `RepairForm::Refuse` (`repair: "none"`) with
  `census: None`. Unlike relational refuse-class checks (`E-TRACK-OVERLAP`, `E-LAYER-TIE`,
  `E-RETIRED`), where the tool must enumerate the competing elements or sibling fields involved in
  the conflict, a schema error is completely localized by its `location` plus `fields: {subject,
  reason}` (or `{subject, key}`).

---

## Consequences

- `crates/montagent-core/src/registry.rs` and `crates/montagent-core/src/checks/schema.rs` cite
  `ADR-0250` in place of `#253`.

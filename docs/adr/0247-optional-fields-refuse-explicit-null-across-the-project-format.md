---
status: accepted
amends: 0030 (optional fields across the project format have two spellings — omitted or written with a concrete value — and explicit JSON null is refused with E-SCHEMA), 0136 (caption on a text element is omitted or a boolean; caption: null is refused with E-SCHEMA)
---

# Optional fields refuse explicit JSON `null` across the project format

[#572](https://github.com/MBehtemam/Montagent/issues/572).
[ADR-0030](0030-defaultable-field-presence-is-content.md) settled that omitting an optional or
defaultable field and writing it explicitly at its default value are two distinct, permanently legal
declarations. [ADR-0136](0136-a-text-element-opts-out-of-the-caption-checks-with-caption-false.md)
added `caption` on `text` as an optional boolean whose absence means `true`. Meanwhile, the
published JSON Schema (`schema/montagent.schema.json`) strips `"null"` from every `Option<T>` via
`deny_null` (`no_optional_field_may_be_written_as_null` in `crates/montagent-core/tests/schema.rs`),
and `group` enforced that rule in its custom deserializer (`E-SCHEMA: `group` is written as null;
omit the key instead`), while derived `Option<T>` fields across the project header, tracks,
elements, runs, keyframes, effects, transitions, and master buses still accepted `"<key>": null` via
serde's default `Option<T>` visitor.

That created a two-artifact disagreement between `schema/montagent.schema.json` and `montagent
validate`, and a third spelling (`"<key>": null`) that undermined ADR-0030's two-state model.

---

## Decision

1. **Every optional field in a Montagent project document refuses explicit JSON `null` (`E-SCHEMA`):**
   An optional field has two legal states: omitted, or present with a value of its declared type.
   Writing `"<key>": null` anywhere in the project root, tracks, elements, text runs, keyframe
   records (`t_from`, `ease`, or `offset`), effects, audio effects, transitions, markers, or master
   configuration is refused by strict deserialization and reported by `validate` as `E-SCHEMA` at
   the enclosing scope (`project`, track, or element) with the reason:
   `` `<key>` is written as null; omit the key instead ``.
2. **Omission is unchanged:**
   Omitting any optional key continues to deserialize to `None` (or its schema default) with zero
   behavior change.
3. **`caption` on `text` (amends [ADR-0136](0136-a-text-element-opts-out-of-the-caption-checks-with-caption-false.md)):**
   `caption` is omitted, `true`, or `false`; `"caption": null` is refused with `E-SCHEMA` like every
   other optional field.

---

## Consequences

- `crates/montagent-core/src/model/mod.rs` provides `refuse_null_fields`, invoked across `Project`,
  `Track`, `Element` (including `layer: null` and nested runs, effects, audio effects, transitions,
  and markers), and keyframe record lists (`read_records`).
- The published JSON Schema (`schema/montagent.schema.json`) and `montagent validate` agree on every
  optional field across the document (`no_optional_field_may_be_written_as_null` in `tests/schema.rs`
  and `optional_fields_across_the_document_refuse_explicit_null` in `tests/closed_schema.rs`).

#!/usr/bin/env python3
"""Re-derives the two structural claims ADR-0070 rests on. Exits non-zero if either stops
reproducing.

    python3 docs/adr/predicate_reserved_names_scan.py

ADR-0070 reserves `track` as a predicate path's first segment and reads an all-digit
segment as an array index. Both are safe only because of a fact about the published
schema, and a fact about a schema is exactly the kind of claim that quietly stops being
true when the schema grows:

  1. no element variant declares a `track` property, so the reservation shadows nothing;
  2. no object property anywhere in the schema is spelled with digits alone, so reading a
     digit segment as an index never steals a reachable key.

Run this before adding a property to `schema/montaget.schema.json`. A failure here is not
a bug in the scan — it means ADR-0070 needs an amendment, because a predicate that used to
reach a field would have stopped reaching it.
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA = os.path.join(HERE, "..", "..", "schema", "montaget.schema.json")

failures = []


def every_object(node):
    """Every dict in the schema document, wherever it sits."""
    if isinstance(node, dict):
        yield node
        for value in node.values():
            yield from every_object(value)
    elif isinstance(node, list):
        for value in node:
            yield from every_object(value)


schema = json.load(open(SCHEMA))
element = schema["$defs"]["Element"]

print(__doc__.splitlines()[0])
print()

# ------------------------------------------------- 1. the reservation shadows nothing
variants = element["oneOf"]
shadowed = [
    variant.get("properties", {}).get("type", {}).get("const", "?")
    for variant in variants
    if "track" in variant.get("properties", {})
]
print(f"1. `track` as an element property, across {len(variants)} element variants")
print(f"   variants declaring `track` ......... {shadowed or 'none'}")
if shadowed:
    failures.append(f"`track` is declared by element variant(s) {shadowed}")

# --------------------------------------------- 2. no declared key is spelled in digits
digit_keys = sorted(
    {
        key
        for node in every_object(schema)
        for key in node.get("properties", {})
        if key.isdigit()
    }
)
print("2. all-digit object properties, anywhere in the schema")
print(f"   properties spelled in digits ....... {digit_keys or 'none'}")
if digit_keys:
    failures.append(f"all-digit propert(ies) {digit_keys} are unreachable by a path segment")

print()
if failures:
    for failure in failures:
        print(f"FAIL: {failure}")
    print("\nADR-0070 no longer reproduces against the schema.")
    sys.exit(1)
print("Both claims reproduce.")

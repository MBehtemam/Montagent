//! The two published resources: the schema, and the rules it cannot express.

use montaget_core::resources;

#[test]
fn the_schema_resource_is_the_generated_artifact_rather_than_a_copy() {
    // The whole reason the schema is generated (spec #168): "the published schema and the
    // enforced schema must be the same thing. Two artifacts is the drift failure this
    // project has already found three times." Serving a hand-maintained file — or the
    // committed one, which is a file that *can* be edited — would put the drift back one
    // level up, where no test was looking.
    let served = resources::read(resources::SCHEMA.uri).expect("the schema resource");

    assert_eq!(served, montaget_core::schema::generated_bytes());

    let parsed: serde_json::Value = serde_json::from_str(&served).expect("a JSON document");
    assert_eq!(parsed["title"], "Project");
    assert!(
        parsed["properties"]["tracks"].is_object(),
        "and it is the project schema, not some other one"
    );
}

#[test]
fn the_schema_resource_carries_canonical_key_order() {
    // ADR-0041 ties canonical key order to the schema's property-declaration order, so an
    // agent reading this resource can see the order it is expected to write in. That only
    // holds if serving it preserves the order; a re-serialisation through a sorted map
    // would silently destroy the one thing the resource is load-bearing for.
    let served = resources::read(resources::SCHEMA.uri).expect("the schema resource");
    let parsed: serde_json::Value = serde_json::from_str(&served).expect("a JSON document");

    let order: Vec<&str> = parsed["properties"]
        .as_object()
        .expect("the header's properties")
        .keys()
        .map(String::as_str)
        .collect();
    let expected =
        montaget_core::layout::canonical_order(montaget_core::layout::Published::Project)
            .expect("the project header is a published type");

    assert_eq!(
        order,
        expected.iter().map(String::as_str).collect::<Vec<_>>()
    );
}

#[test]
fn the_format_docs_serve_the_rules_the_schema_cannot_express() {
    let docs = resources::read(resources::FORMAT.uri).expect("the format docs resource");

    // Each of these is a rule between fields, between elements, or between the file and
    // the disk — the class of thing a shape language has no way to *enforce*, and the
    // reason this resource exists beside the schema rather than instead of it. Asserted by
    // presence alone: the schema's own `description` strings are prose and mention several
    // of these in passing, and what a schema cannot express is a matter of what it checks,
    // not of which words appear in it.
    for rule in [
        // The convention the whole exact-string-editing surface leans on (ADR-0005/0041).
        "exactly one line",
        // ADR-0005's half-open ranges.
        "[start, end)",
        // ADR-0004: a track supplies stacking, never timing.
        "never timing",
        // ADR-0019's one hop.
        "one hop",
        // ADR-0030.
        "Presence is content",
        // ADR-0016: the unknown-key error is the versioning mechanism.
        "no format version field",
        // ADR-0053.
        "relative to the project file's own directory",
        // ADR-0011's write-tool invariant, which is what this ticket makes observable.
        "never `ok`",
    ] {
        assert!(
            docs.contains(rule),
            "the format docs say nothing about {rule:?}"
        );
    }
}

#[test]
fn both_resources_are_listed_and_every_listed_one_is_readable() {
    // A listing that names a URI nothing serves is worse than no listing: an agent spends a
    // round trip discovering the gap.
    let listed = resources::all();
    assert_eq!(listed.len(), 2, "ADR-0011 publishes two resources");

    for resource in listed {
        assert!(
            resources::read(resource.uri).is_some_and(|body| !body.trim().is_empty()),
            "{} is listed but serves nothing",
            resource.uri
        );
        assert!(!resource.name.is_empty());
        assert!(!resource.description.is_empty());
        assert!(resource.uri.starts_with("montaget://"), "{}", resource.uri);
    }
}

#[test]
fn an_unpublished_uri_serves_nothing() {
    assert_eq!(resources::read("montaget://everything.json"), None);
    assert_eq!(resources::read("file:///etc/passwd"), None);
}

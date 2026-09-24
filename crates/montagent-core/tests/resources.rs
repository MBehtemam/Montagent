//! The two published resources: the schema, and the rules it cannot express.

use montagent_core::resources;

#[test]
fn the_schema_resource_is_the_generated_artifact_rather_than_a_copy() {
    // The whole reason the schema is generated (spec #168): "the published schema and the
    // enforced schema must be the same thing. Two artifacts is the drift failure this
    // project has already found three times." Serving a hand-maintained file — or the
    // committed one, which is a file that *can* be edited — would put the drift back one
    // level up, where no test was looking.
    let served = resources::read(resources::SCHEMA.uri).expect("the schema resource");

    assert_eq!(served, montagent_core::schema::generated_bytes());

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
        montagent_core::layout::canonical_order(montagent_core::layout::Published::Project)
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
fn every_adr_the_format_docs_cite_exists_and_is_still_accepted() {
    // The guard the prose half needs and the schema half gets for free. The schema resource
    // is generated, so it cannot drift; these 200 lines are hand-written and restate rules
    // the ADR series owns — which is the two-artifact drift spec #168 says this project has
    // already found three times, in a place no test was looking.
    //
    // It is not a semantic check and does not pretend to be: what it catches is a rule
    // taught here whose ADR has since been retracted or superseded outright. That is the
    // failure that would otherwise be silent, because the docs would go on reading
    // perfectly well. `docs/agents/domain.md` asks a claim like this for a re-executable
    // check rather than a screenshot of one; this is it.
    let docs = resources::read(resources::FORMAT.uri).expect("the format docs resource");

    let mut cited: Vec<String> = docs
        .match_indices("ADR-")
        .filter_map(|(at, _)| docs.get(at + 4..at + 8))
        .filter(|number| number.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_string)
        .collect();
    cited.sort();
    cited.dedup();
    assert!(
        cited.len() > 20,
        "the docs cite {} ADRs, which is too few to be citing its rules at all",
        cited.len()
    );

    let adrs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/adr");
    for number in cited {
        let file = std::fs::read_dir(&adrs)
            .expect("the ADR series")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&format!("{number}-")))
            })
            .unwrap_or_else(|| panic!("the format docs cite ADR-{number}, which does not exist"));

        let body = std::fs::read_to_string(&file).expect("read the ADR");
        assert!(
            status(&body).is_some_and(|status| status.starts_with("accepted")),
            "the format docs teach a rule from ADR-{number}, whose status is {:?}: {}",
            status(&body),
            file.display()
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
        assert!(resource.uri.starts_with("montagent://"), "{}", resource.uri);
    }
}

#[test]
fn the_published_surface_is_the_one_adr_0080_names() {
    // ADR-0011 publishes "the schema" and "the format docs" and names neither. ADR-0080
    // names both, and the point of naming them is that they are quotable: an agent that
    // has been told to read `montagent://schema.json` must still find it there a release
    // later. Nothing else in the suite pins these six strings, so before this test a
    // rename was a silent break in a published surface — which is exactly the failure the
    // ADR exists to prevent.
    let listed = resources::all();
    let surface: Vec<(&str, &str, &str)> = listed
        .iter()
        .map(|r| (r.uri, r.name, r.mime_type))
        .collect();

    assert_eq!(
        surface,
        vec![
            (
                "montagent://schema.json",
                "montagent-schema",
                "application/schema+json"
            ),
            ("montagent://format.md", "montagent-format", "text/markdown"),
        ],
        "the two resource URIs, names and media types are ratified by ADR-0080 and do not \
         move; the order is shape first, then the rules over it"
    );
}

#[test]
fn an_unpublished_uri_serves_nothing() {
    assert_eq!(resources::read("montagent://everything.json"), None);
    assert_eq!(resources::read("file:///etc/passwd"), None);
}

/// An ADR's declared status, in either of the two spellings the series carries: YAML front
/// matter (`status: accepted`) and a bold header line (`**Status:** accepted`). Both are
/// real and committed, and a check that knew only the first passed 65 ADRs and failed on
/// ADR-0008 for a reason that had nothing to do with its status.
fn status(body: &str) -> Option<String> {
    body.lines().take(12).find_map(|line| {
        let line = line.trim();
        let rest = line
            .strip_prefix("status:")
            .or_else(|| line.strip_prefix("**Status:**"))?;
        Some(rest.trim().to_lowercase())
    })
}

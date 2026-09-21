//! `compare` (#221): what changed between two versions of one timeline, as facts with no
//! severity. Every assertion here reads the returned `Report` by stable code plus the
//! numbers in its fields — never by message text (spec #168).

use montaget_core::finding::{Class, Finding};
use montaget_core::report::Report;
use montaget_core::verbs::compare::compare;

mod common;
use common::{canonical, tempdir, write_project};

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn findings_of<'a>(report: &'a Report, code: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

fn write_pair(dir: &std::path::Path, ref_body: &str, current_body: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let ref_path = write_project(dir, "ref.montaget.json", &canonical(ref_body));
    let current_path = write_project(dir, "current.montaget.json", &canonical(current_body));
    (ref_path, current_path)
}

// ---------------------------------------------------------------------------
// Every `compare` finding is `Drift`-class, and never gates the exit code.
// ---------------------------------------------------------------------------

#[test]
fn every_compare_finding_is_drift_class_and_never_gates_the_exit_code() {
    let dir = tempdir(line!());
    let (r, c) = write_pair(
        &dir,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
            {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
            {"id":"r2","type":"rect","start":1200,"end":2000,"width":10,"height":10,"fill":"#FF0000"}
        ]}]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
            {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
            {"id":"r2","type":"rect","start":1300,"end":2100,"width":10,"height":10,"fill":"#FF0000"}
        ]}]}"##,
    );

    let report = compare(&r, &c);

    assert!(
        report.findings.iter().all(|f| f.class == Class::Drift),
        "every compare finding should be Drift-class, got {:?}",
        report.findings.iter().map(|f| (&f.code, f.class)).collect::<Vec<_>>()
    );
    assert_eq!(
        report.exit_code(),
        montaget_core::report::ExitCode::Ok,
        "drift never gates the exit code"
    );
    assert_eq!(report.summary().drift, report.findings.len());
    assert_eq!(report.summary().error, 0);
}

// ---------------------------------------------------------------------------
// Predicate 1: slack drift.
// ---------------------------------------------------------------------------

#[test]
fn slack_drift_fires_when_a_nonzero_distance_slack_changes_size() {
    let dir = tempdir(line!());
    let (r, c) = write_pair(
        &dir,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
            {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
            {"id":"r2","type":"rect","start":1200,"end":2000,"width":10,"height":10,"fill":"#FF0000"}
        ]}]}"##,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
            {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
            {"id":"r2","type":"rect","start":1300,"end":2100,"width":10,"height":10,"fill":"#FF0000"}
        ]}]}"##,
    );

    let report = compare(&r, &c);

    let hits = findings_of(&report, "D-SLACK-DRIFT");
    assert_eq!(hits.len(), 1, "codes: {:?}", codes(&report));
    let f = hits[0];
    assert_eq!(f.fields["from"], 1000);
    assert_eq!(f.fields["to"], 1200);
    assert_eq!(f.fields["ref_size"], 200);
    assert_eq!(f.fields["current_size"], 300);
}

#[test]
fn slack_drift_does_not_fire_when_nothing_about_the_slack_changed() {
    let dir = tempdir(line!());
    let body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"r2","type":"rect","start":1200,"end":2000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    // A no-op edit: only a field unrelated to timing changed between the two versions.
    let current = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#00FF00"},
        {"id":"r2","type":"rect","start":1200,"end":2000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    let (r, c) = write_pair(&dir, body, current);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-SLACK-DRIFT").is_empty(),
        "a check that fires on an unrelated edit is exactly what ADR-0039 was written \
         against: codes {:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------
// Predicate 2: keyframe-instant relationship drift.
// ---------------------------------------------------------------------------

#[test]
fn keyframe_instant_relationship_that_held_and_stopped_is_reported() {
    let dir = tempdir(line!());
    // `photo` has an `opacity` keyframe at t=3018, which coincides in the reference
    // document with `sentence`'s own `start` (case (b): a keyframe against a
    // *different* element's boundary).
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[
        {"name":"a","layer":1,"elements":[
            {"id":"photo","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":3018,"v":1,"ease":"linear"}]}
        ]},
        {"name":"b","layer":2,"elements":[
            {"id":"sentence","type":"rect","start":3018,"end":6000,"width":10,"height":10,"fill":"#FF0000"}
        ]}
    ]}"##;
    // `sentence`'s own `start` moves to 3200; the keyframe itself is untouched.
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[
        {"name":"a","layer":1,"elements":[
            {"id":"photo","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":3018,"v":1,"ease":"linear"}]}
        ]},
        {"name":"b","layer":2,"elements":[
            {"id":"sentence","type":"rect","start":3200,"end":6000,"width":10,"height":10,"fill":"#FF0000"}
        ]}
    ]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    let hits = findings_of(&report, "D-KEYFRAME-INSTANT-DRIFT");
    assert_eq!(hits.len(), 1, "codes: {:?}", codes(&report));
    let f = hits[0];
    assert_eq!(f.fields["kind"], "b");
    assert_eq!(f.fields["ref_at"], 3018);
    // One side is the untouched keyframe (still 3018), the other is the moved boundary.
    let (left, right) = (
        f.fields["current_left"].as_i64().unwrap(),
        f.fields["current_right"].as_i64().unwrap(),
    );
    assert_ne!(left, right);
    assert!([left, right].contains(&3018));
    assert!([left, right].contains(&3200));
}

#[test]
fn a_keyframe_instant_relationship_that_still_holds_is_not_reported() {
    let dir = tempdir(line!());
    let body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[
        {"name":"a","layer":1,"elements":[
            {"id":"photo","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":3018,"v":1,"ease":"linear"}]}
        ]},
        {"name":"b","layer":2,"elements":[
            {"id":"sentence","type":"rect","start":3018,"end":6000,"width":10,"height":10,"fill":"#FF0000"}
        ]}
    ]}"##;
    // Both versions are byte-identical: the relationship still holds, so nothing fires.
    let (r, c) = write_pair(&dir, body, body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-KEYFRAME-INSTANT-DRIFT").is_empty(),
        "codes: {:?}",
        codes(&report)
    );
}

#[test]
fn case_c_two_elements_same_property_keyframe_times_is_covered_by_a_synthetic_fixture() {
    let dir = tempdir(line!());
    // ADR-0063 records zero real-fixture coverage for case (c) — covered here instead.
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[
        {"name":"a","layer":1,"elements":[
            {"id":"card-a","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":2000,"v":1,"ease":"linear"}]},
            {"id":"card-b","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":2000,"v":1,"ease":"linear"}]}
        ]}
    ]}"##;
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[
        {"name":"a","layer":1,"elements":[
            {"id":"card-a","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":2000,"v":1,"ease":"linear"}]},
            {"id":"card-b","type":"rect","start":0,"end":5000,"width":10,"height":10,"fill":"#FF0000",
             "opacity":[{"t":0,"v":0},{"t":2400,"v":1,"ease":"linear"}]}
        ]}
    ]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    let hits = findings_of(&report, "D-KEYFRAME-INSTANT-DRIFT");
    assert_eq!(hits.len(), 1, "codes: {:?}", codes(&report));
    assert_eq!(hits[0].fields["kind"], "c");
    assert_eq!(hits[0].fields["ref_at"], 2000);
}

// ---------------------------------------------------------------------------
// Predicate 3: boundary-coincidence-cluster drift.
// ---------------------------------------------------------------------------

#[test]
fn cluster_drift_reports_moved_set_versus_stayed_set_not_pairwise() {
    let dir = tempdir(line!());
    // A, B, C all coincide at 6000 in the reference document (A's own `end`, B's and
    // C's own `start`) — a 3-way cluster.
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":4000,"end":6000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":6000,"end":8000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-c","type":"rect","start":6000,"end":9000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    // Only `elem-c` moves; `elem-a` and `elem-b` still coincide at 6000.
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":4000,"end":6000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":6000,"end":8000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-c","type":"rect","start":6300,"end":9300,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    let hits = findings_of(&report, "D-BOUNDARY-CLUSTER-DRIFT");
    assert_eq!(
        hits.len(),
        1,
        "one fact per destroyed cluster, never pairwise: codes {:?}",
        codes(&report)
    );
    let f = hits[0];
    assert_eq!(f.fields["at"], 6000);
    assert_eq!(f.fields["which"], "reference");
    let stayed = f.fields["stayed"].as_str().unwrap();
    assert!(stayed.contains("elem-a") && stayed.contains("elem-b"));
    let moved = f.fields["moved"].as_str().unwrap();
    assert!(moved.contains("elem-c"));

    let census = f.census.as_ref().expect("a cluster fact carries a census");
    assert_eq!(census.field, "boundary_instant");
    let total_members: usize = census.groups.iter().map(|g| g.members.len()).sum();
    assert_eq!(total_members, 3);
}

#[test]
fn a_cluster_that_moved_together_produces_no_fact() {
    let dir = tempdir(line!());
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":4000,"end":6000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":6000,"end":8000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    // Both members of the cluster move together, by the same amount: the cluster is
    // intact at its new instant, not destroyed.
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":4200,"end":6200,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":6200,"end":8200,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-BOUNDARY-CLUSTER-DRIFT").is_empty(),
        "a cluster that moved together, unchanged relative to itself, is not destroyed: \
         codes {:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------
// Suppression: a zero-distance slack change is reported by cluster-drift alone.
// ---------------------------------------------------------------------------

#[test]
fn a_zero_distance_slack_change_is_reported_only_by_cluster_drift() {
    let dir = tempdir(line!());
    // A 50 ms lead-out between `elem-a`'s end and `elem-b`'s start.
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":1050,"end":2000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    // The gap closes to an exact coincidence.
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"elem-a","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"},
        {"id":"elem-b","type":"rect","start":1000,"end":1950,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-SLACK-DRIFT").is_empty(),
        "a slack closing to a coincidence is cluster-drift's alone to report (ADR-0066): \
         codes {:?}",
        codes(&report)
    );
    let hits = findings_of(&report, "D-BOUNDARY-CLUSTER-DRIFT");
    assert_eq!(hits.len(), 1, "codes: {:?}", codes(&report));
    assert_eq!(hits[0].fields["at"], 1000);
    assert_eq!(hits[0].fields["which"], "current");
}

// ---------------------------------------------------------------------------
// Predicate 4: highlight text-drift.
// ---------------------------------------------------------------------------

#[test]
fn a_runs_text_changing_while_its_highlight_does_not_is_reported() {
    let dir = tempdir(line!());
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Hello","highlight":{"start":100,"end":200}}]}
    ]}]}"##;
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Howdy","highlight":{"start":100,"end":200}}]}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    let hits = findings_of(&report, "D-HIGHLIGHT-TEXT-DRIFT");
    assert_eq!(hits.len(), 1, "codes: {:?}", codes(&report));
    let f = hits[0];
    assert_eq!(f.location.element.as_deref(), Some("caption"));
    assert_eq!(f.fields["run_index"], 0);
    assert_eq!(f.fields["ref_text"], "Hello");
    assert_eq!(f.fields["current_text"], "Howdy");
}

#[test]
fn a_runs_text_changing_along_with_its_highlight_is_not_reported() {
    let dir = tempdir(line!());
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Hello","highlight":{"start":100,"end":200}}]}
    ]}]}"##;
    // The window itself also moved — a deliberate re-alignment, not the karaoke-drift
    // shape this check is written for.
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Howdy","highlight":{"start":150,"end":250}}]}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-HIGHLIGHT-TEXT-DRIFT").is_empty(),
        "codes: {:?}",
        codes(&report)
    );
}

#[test]
fn plain_unhighlighted_text_changing_is_not_highlight_drift() {
    let dir = tempdir(line!());
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Hello"}]}
    ]}]}"##;
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Howdy"}]}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-HIGHLIGHT-TEXT-DRIFT").is_empty(),
        "an ordinary text edit with no highlight in play is not karaoke drift: codes {:?}",
        codes(&report)
    );
}

#[test]
fn an_element_whose_run_count_differs_is_skipped_for_highlight_drift() {
    let dir = tempdir(line!());
    let ref_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Hello","highlight":{"start":100,"end":200}}]}
    ]}]}"##;
    let current_body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"caption","type":"text","start":0,"end":1000,"width":500,"height":100,"font":"brand","size":40,
         "runs":[{"text":"Hello ","highlight":{"start":100,"end":200}},{"text":"there"}]}
    ]}]}"##;
    let (r, c) = write_pair(&dir, ref_body, current_body);

    let report = compare(&r, &c);

    assert!(
        findings_of(&report, "D-HIGHLIGHT-TEXT-DRIFT").is_empty(),
        "no run identity to diff against when the run count itself changed: codes {:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------
// No keyframe resolver: a project with no keyframes at all still compares cleanly.
// ---------------------------------------------------------------------------

#[test]
fn a_project_with_no_keyframes_at_all_still_runs_compare_correctly() {
    let dir = tempdir(line!());
    let body = r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":1,"elements":[
        {"id":"r1","type":"rect","start":0,"end":1000,"width":10,"height":10,"fill":"#FF0000"}
    ]}]}"##;
    let (r, c) = write_pair(&dir, body, body);

    let report = compare(&r, &c);

    assert!(
        report.findings.is_empty(),
        "identical documents with no keyframes at all should report nothing: {:?}",
        codes(&report)
    );
    assert_eq!(report.tool, "compare");
}

// ---------------------------------------------------------------------------
// Both files must independently parse and look like projects.
// ---------------------------------------------------------------------------

#[test]
fn an_unparseable_current_file_is_reported_as_unparseable() {
    let dir = tempdir(line!());
    let ref_path = write_project(
        &dir,
        "ref.montaget.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[]}"##,
        ),
    );
    let current_path = write_project(&dir, "current.montaget.json", "{ not json");

    let report = compare(&ref_path, &current_path);

    assert_eq!(
        report.exit_code(),
        montaget_core::report::ExitCode::Unparseable
    );
    assert_eq!(codes(&report), vec!["E-PARSE"]);
}

#[test]
fn a_current_file_that_is_not_a_project_is_reported_as_not_a_project() {
    let dir = tempdir(line!());
    let ref_path = write_project(
        &dir,
        "ref.montaget.json",
        &canonical(
            r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[]}"##,
        ),
    );
    let current_path = write_project(&dir, "current.montaget.json", r##"{"hello":"world"}"##);

    let report = compare(&ref_path, &current_path);

    assert_eq!(codes(&report), vec!["E-NOT-A-PROJECT"]);
}

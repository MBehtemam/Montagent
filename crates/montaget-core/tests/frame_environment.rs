//! The two `frame` claims that can only be asserted by changing the process's environment
//! (#212): that no font outside the declared chain is opened even with a decoy installed
//! (ADR-0007), and that a project of stills alone needs no `ffmpeg` at all (ADR-0009).
//!
//! **Their own test binary, and that is the point.** Both set a process-wide environment
//! variable, and Rust runs the tests inside one binary as threads of one process — so a
//! test that empties `PATH` while a sibling is spawning `ffprobe` fails the sibling, for a
//! reason that has nothing to do with either. That is not hypothetical: it was observed as
//! an intermittent failure of the video decode tests before these two moved here. Cargo
//! runs each test binary as its own process, so this file is the isolation.
//!
//! The two tests here can still run concurrently with each other, and deliberately do: one
//! empties `PATH`, and the other's project is a single text element with no raster source,
//! which reaches for no `ffmpeg` either way.

use std::path::{Path, PathBuf};

use montaget_core::report::ExitCode;
use montaget_core::verbs::frame::{Ask, frame};
use serde_json::Value;

mod common;
use common::{canonical, tempdir, write_project};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn at(instant: i64) -> Ask {
    Ask {
        at: Some(instant),
        ..Ask::default()
    }
}

/// One frame, refusing to continue if the verb did not answer.
#[track_caller]
fn drawn(project: &Path, ask: &Ask) -> Value {
    let answer = frame(project, ask);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer.to_json()
}

/// A project with one track holding the given elements, on a 400×400 black frame.
fn one_track(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
            "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
    ))
}

#[test]
fn no_font_outside_the_declared_chain_is_opened_even_with_a_decoy_installed() {
    // ADR-0007: "always a file path, relative to the project. Never a system family name."
    // #212 puts the enforcement at the renderer rather than at `measure`, and asks for it
    // asserted with a decoy installed — so the decoy is a real font file, on disk, under a
    // family name the project's chain also uses, in a directory the process is pointed at
    // by every environment variable a font stack might consult.
    //
    // **What this does and does not prove, stated rather than left to be inferred.** It
    // reads a list the registry keeps of every path it handed to the filesystem, so it
    // covers the one font stack Montaget has: `parley`, compiled without `fontique`'s
    // system-font discovery, which is why ADR-0007's rule is structural rather than a
    // discipline. It says nothing about Skia's own font manager — which this build never
    // reaches, because ADR-0010 puts the text stack *beside* the rasterizer rather than
    // inside it and no Skia text API is called. When #213 paints glyphs, that is the claim
    // to re-examine, not this one.
    let dir = tempdir(line!());
    let decoy_dir = dir.join("decoy-fonts");
    std::fs::create_dir_all(&decoy_dir).expect("a decoy font directory");
    let decoy = decoy_dir.join("OpenRunde-Bold.otf");
    std::fs::copy(fixture_dir().join("fonts/OpenRunde-Bold.otf"), &decoy)
        .expect("a decoy that is a real, loadable font");

    let declared = fixture_dir().join("fonts/OpenRunde-Bold.otf");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
                "fonts":{{"brand":[{{"file":"{}"}}]}},
                "tracks":[{{"name":"only","layer":0,"elements":[
                  {{"id":"title","type":"text","start":0,"end":1000,"x":10,"y":10,
                    "origin":"top-left","width":300,"height":80,"font":"brand","size":40,
                    "runs":[{{"text":"hello"}}]}}]}}]}}"##,
            declared.display().to_string().replace('\\', "\\\\")
        )),
    );

    // Point the process at the decoy by every name a font stack is known to look under.
    // The claim is not "we did not ask for it" but "it was never opened", and the test is
    // only worth anything if finding it would have been easy.
    for key in ["XDG_DATA_HOME", "XDG_DATA_DIRS", "FONTCONFIG_PATH", "HOME"] {
        // SAFETY: the suite sets these for the duration of one test and reads them back
        // through the code under test alone; no other test in this binary consults them.
        unsafe { std::env::set_var(key, &decoy_dir) };
    }

    let json = drawn(&project, &at(500));

    let opened: Vec<String> = json["frame"]["fonts"]
        .as_array()
        .expect("a fonts list")
        .iter()
        .filter_map(Value::as_str)
        .map(common::with_forward_slashes)
        .collect();
    let decoy = common::with_forward_slashes(&decoy.display().to_string());

    assert_eq!(
        opened.len(),
        1,
        "the chain has one entry, so exactly one file is opened: {opened:?}"
    );
    assert!(
        opened[0].ends_with("fonts/OpenRunde-Bold.otf"),
        "{opened:?}"
    );
    assert!(
        !opened.contains(&decoy),
        "the decoy at {decoy} was opened: {opened:?}"
    );
}

#[test]
fn a_project_of_stills_alone_never_needs_an_ffmpeg() {
    // ADR-0009 ships Montaget as "a binary, plus an `ffmpeg` the user supplies", so a
    // project that has nothing to decode must not require one. Asserted structurally: the
    // one project below has no raster source carrying a `clip` either, which is the other
    // thing that would reach for a probe (`query --at`'s crop rectangle).
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"card","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":100,"height":100,"fill":"#FF0000"}"##,
        ),
    );

    let path = std::env::var_os("PATH");
    // SAFETY: as above — one test, one variable, read back only through the code under
    // test, and restored before the test returns.
    unsafe { std::env::set_var("PATH", "") };
    let answer = frame(&project, &at(500));
    match path {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "a project of stills drew without an ffmpeg anywhere: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
}

//! `validate` — *"is this project file internally legal, and does it agree with the
//! media on disk?"* (ADR-0006).
//!
//! Both halves now run. The document is read, or one `E-PARSE`/`E-READ` finding comes back
//! and nothing is partially processed; the file is confirmed to be a project at all
//! (ADR-0042), or one `E-NOT-A-PROJECT` finding comes back instead; then every registered
//! check runs over the whole project, and every source it references is probed.
//!
//! Two properties of `validate` are structural and are settled here rather than by any
//! individual check (ADR-0006): it **always runs every check on the whole project** —
//! there is no fast mode, no `--no-probe` and no way to narrow what is analysed — and its
//! report always ends with its own boundary. Output may be filtered; analysis may not.
//!
//! One of the checks answers a question that is not about the video at all. ADR-0041's
//! `LAYOUT` is a fourth report category rather than a severity: a file written outside the
//! canonical convention is *"unsafe to edit, not unsafe to render"*, so it is reported on
//! every run and gates nothing.

use std::path::{Path, PathBuf};

use crate::finding::Finding;
use crate::media::session::Session;
use crate::media::sidecar::Sidecar;
use crate::media::tools::Missing;
use crate::parse;
use crate::permissive::Loose;
use crate::registry::CheckSet;
use crate::report::Report;

/// The verb's own name, as it travels in the report. One spelling, so the two exits
/// from the function below cannot disagree about which tool answered.
const TOOL: &str = "validate";

/// Validate the project file at `path`.
///
/// The session — and with it the probe cache and its sidecar — is this call's own, which
/// is the CLI's shape: one process, one run, and the sidecar is what carries the cache
/// across to the next one (ADR-0069).
pub fn validate(path: &Path) -> Report {
    run(path, None, Sidecar::default_path())
}

/// The same, over a sidecar file the caller owns.
///
/// `Session::with_sidecar`'s reason, for the half of the cache that is not the session's:
/// `R-FONT-SWAP` is a comparison against what a *previous run* recorded, so a test that
/// cannot own the file it is recorded in cannot exercise the check at all without writing
/// to the developer's own cache.
pub fn validate_with_cache(path: &Path, cache: &Path) -> Report {
    run(path, None, Some(cache.to_path_buf()))
}

/// The same, against a session the caller owns — an MCP server's warm cache (ADR-0011), or
/// a test's recorded `ffprobe`. The remote half is cleared here, because this is one run.
pub fn validate_with(path: &Path, session: &mut Session) -> Report {
    session.begin_run();
    // The session's own sidecar, not this machine's default: a caller who owns the probe
    // half of the cache owns the font half too, which is the whole point of taking it as an
    // argument (`Session::open_at`). A session with no sidecar makes this run's font half
    // cacheless as well, so `R-FONT-SWAP` is silent rather than comparing against a file the
    // caller said not to use.
    let cache = session.cache_path().map(Path::to_path_buf);
    run(path, Some(session), cache)
}

fn run(path: &Path, session: Option<&mut Session>, cache: Option<PathBuf>) -> Report {
    match checked(TOOL, path, session, cache) {
        Ok((_, report)) => report,
        Err(report) => *report,
    }
}

/// Read, confirm the shape, and run every registered check — for `validate`, and for the
/// one other verb that must run the identical engine.
///
/// **This is where ADR-0006's structural defence is made structural.** *"`render` runs the
/// identical check engine and refuses on any `error`"* is a claim about one implementation,
/// and the only way to assert it is for there to be one: `validate` is this function with
/// the document thrown away, and `render` is this function with the document kept for
/// painting. A second parse-shape-check sequence in `render.rs`, however carefully copied,
/// would be a second place a check could be left out.
///
/// `Ok` carries the document and the report the checks produced — which may still say
/// exit 70, where the disk half could not run. `Err` is the report for a file that never
/// reached the checks: unparseable, or not a project — boxed, as `cli::run_verb` boxes
/// its error side, so the happy path does not carry a `Report`'s width twice.
pub(crate) fn checked(
    tool: &'static str,
    path: &Path,
    session: Option<&mut Session>,
    cache: Option<PathBuf>,
) -> Result<(Loose, Report), Box<Report>> {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        Err(finding) => return Err(Box::new(Report::unparseable(tool, project, *finding))),
    };

    let mut report = Report::new(tool, project);

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's precondition, which `fmt`, `timeline` and `query` already hold and
        // `validate` did not — because until #244 nothing downstream noticed. The gap the
        // ADR found was message quality: a `validate` pointed at a transcript export should
        // *"name the likely mismatch, not dump a raw schema error"*, and a schema check that
        // has just learned to speak would do exactly that — "the project does not fit the
        // published schema: missing field `frame`" — about a file that was never a project.
        //
        // What a verb does about the failure is the verb's own business, and this is
        // `validate`'s answer: the three other verbs' answer, for the three other verbs'
        // reason. It does not narrow what is analysed on a *project*, which is what ADR-0006
        // forbids; it declines to analyse something that is not one.
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            &format!("point {tool} at the project file"),
        ));
        return Err(Box::new(report));
    }

    if let Err(missing) = run_checks(&document, &mut report, session, cache.as_deref()) {
        // There is no `ffprobe`, so the disk half of the question cannot be asked. ADR-0006
        // forbids answering it with silence, and ADR-0011 gives "Montagent could not run"
        // its own exit code precisely so it is not mistaken for a defect in the project.
        //
        // What the document half already established stays in the report: a run that found
        // a retired spelling and *then* discovered there is no `ffprobe` has learned two
        // things, and an agent told only the second would fix its `PATH`, re-run, and only
        // then hear about the key it could have fixed in the same turn.
        missing.fail(&mut report);
    }
    Ok((document, report))
}

/// Every registered check, over the whole document.
///
/// One call per check, in no significant order: a report's findings are a set of facts
/// about the project, and nothing downstream may depend on which check spoke first.
fn run_checks(
    document: &Loose,
    report: &mut Report,
    session: Option<&mut Session>,
    cache: Option<&Path>,
) -> Result<(), Box<Missing>> {
    // ADR-0017's closed schema, turned into findings (#244). Not gating: every other check
    // still runs on a document that does not fit the types, because every other check reads
    // the permissive tree and each of them has something true to say about a file mid-edit.
    // What changes is that the file no longer validates *clean* while carrying a key the
    // format does not publish — which is ADR-0016's whole migration mechanism, and was
    // unfired until this call existed.
    crate::checks::schema::check(document, report);
    crate::checks::retired::check(document, report);
    crate::checks::anchor::check(document, report);
    // ADR-0060's layer tie (#209): two elements resolving to one layer whose boxes
    // actually overlap, in both time and space. It reads the same `crate::stack`
    // resolution the anchor check above does — one rule, asked twice, never implemented
    // twice — and samples geometry across the pair's shared time range, which is why it is
    // its own check and not a branch of that one.
    crate::checks::tie::check(document, report);
    // The checks that read the clock (#197). Three questions and one traversal each: two
    // elements of one track sharing an instant is the rule tracks exist to enforce
    // (ADR-0004); a gap is reported apart from an overlap and is never an error
    // (ADR-0006); and the source/timeline invariant is evaluated in exact rational
    // arithmetic, never `f64` (ADR-0045).
    crate::checks::track::check(document, report);
    crate::checks::speed::check(document, report);
    // ADR-0157: a `video` carrying `source_time` — the fields its curve refuses, its
    // silence, and a curve holding past its keys on painted frames. Document-only; the
    // remap arm of `E-SOURCE-OVERRUN` is in the disk checks.
    crate::checks::remap::check(document, report);
    // ADR-0107's `E-EMPTY-RANGE` (#410): an element whose range does not advance holds no
    // instant of the half-open clock, and every check above steps over it for that reason.
    // `render` refused it and this list said nothing, so this is the check that owns it.
    crate::checks::range::check(document, report);
    // The cross-track coverage question the track check cannot see (ADR-0018, #200):
    // group-scoped pairing, not a frame-wide union — see `crate::checks::coverage`.
    crate::checks::coverage::check(document, report);
    // The four caption checks (#199). Every one is a pure document read — ADR-0054 says so
    // of the one that looks like it needs the disk — which is why this call takes no
    // session and can sit anywhere in this list.
    crate::checks::caption::check(document, report);
    // `R-BOX-SLACK` (#201, ADR-0058): a declared text-box `height` that overshoots the
    // computed block height beyond `max(2px, 10%)`. Height-only arithmetic on fields the
    // document already carries — no font, no I/O — so this too can sit anywhere in the
    // list.
    crate::checks::box_slack::check(document, report);
    // `E-HIGHLIGHT-RANGE`/`E-HIGHLIGHT-OVERLAP` (#202, ADR-0051): a per-word `highlight`
    // window out of its parent run's own range, or overlapping a sibling's; and
    // `R-HIGHLIGHT-UNPAINTED` (#554, ADR-0134): a window that holds no painted frame. And
    // `E-TRANSITION-RANGE` (#202, ADR-0059): a transition's derived range drifted from
    // its two bridged elements. All three read only the document, so — like the checks
    // above — they can sit anywhere in this list.
    crate::checks::highlight::check(document, report);
    crate::checks::transition::check(document, report);
    // ADR-0007's three text-byte checks (#206): a run boundary inside a grapheme cluster,
    // the invisible-character census, and two canonically-equivalent spellings of one
    // string. All three read the concatenated `runs` text and nothing else — no font, no
    // disk — so like the checks above they can sit anywhere in this list.
    crate::checks::runs::check(document, report);
    // ADR-0153's `R-SPACING-SUPPRESSED`: a non-zero `letter_spacing` on text whose
    // joining-script letters take none between them. The pairs come from the text's Unicode
    // properties and never from a font, so like the checks above it can sit anywhere here.
    crate::checks::spacing::check(document, report);
    // ADR-0151's stagger checks: the three override errors, `R-UNIT-MERGED` and
    // `N-CAPTION-SETTLES`. The units and the joins come from the text; the shaping merges
    // from the element's fonts, which the ink check below opens too.
    crate::checks::units::check(document, report);
    // The motion and geometry checks (#211), each needing resolved values rather than
    // declared ones. `R-SOURCE-CUT-POP` compares an animated property across a same-source
    // hard cut, and across the wrap when the project declares `loop` (ADR-0033, ADR-0062);
    // `R-KEYFRAME-UNREACHED` asks whether a declared endpoint is ever on a sampled frame
    // (ADR-0035); `R-OFF-CANVAS` is a standing whole-range rect-against-frame test
    // (ADR-0044); `R-EASE-INERT` reads author-written `v` literally and touches no
    // resolver at all (ADR-0052). All four read only the document, so — like the checks
    // above — they can sit anywhere in this list.
    crate::checks::cut::check(document, report);
    crate::checks::unreached::check(document, report);
    crate::checks::canvas::check(document, report);
    crate::checks::ease::check(document, report);
    // ADR-0146 §6: a keyed element-level text paint field every run overrides changes
    // nothing. Document-only.
    crate::checks::overridden::check(document, report);
    // ADR-0149 §5: a stop list whose offsets decrease, and a gradient that paints one
    // colour. Document-only.
    crate::checks::gradient::check(document, report);
    // ADR-0086's `R-DERIVED-T` (#328): a keyframe's declared `t_from` re-derives an instant
    // that is not the `t` beside it. Both sides of a declared derivation sit in this one
    // document, which is why the check is here and not in `compare` — that verb is defined
    // by having an input this one lacks, and a declared relationship needs none.
    crate::checks::derived::check(document, report);
    // ADR-0084's non-square circle mask: also document-only, and also derived arithmetic
    // rather than written numbers — the rect it measures is usually the one nobody wrote.
    crate::checks::mask::check(document, report);
    // ADR-0154 §6: a path's vertex count, dangling handles, keyframe shape and containment,
    // decided from every literal value of `points`. Document-only.
    crate::checks::path::check(document, report);
    // ADR-0161 §8: the same four point errors on a text's own `path`, a line break on a
    // text carrying one, and a `path_offset` with no `path`. Document-only.
    crate::checks::text_path::check(document, report);
    // ADR-0158 §6–§7: a stroke join, miter limit or cap with nothing to shape, a miter and
    // its limit apart, and a cap where none draws. Document-only.
    crate::checks::stroke::check(document, report);
    // ADR-0147's `R-BLEND-BACKGROUND-ONLY`: a blended element with nothing beneath it,
    // decided from boxes at the instants `render` paints. Document-only.
    crate::checks::blend::check(document, report);
    // ADR-0155's `R-MOTION-BLUR-STILL`: a `motion_blur` on an element that never moves
    // inside its own range. Document-only, decided without painting a frame.
    crate::checks::motion_blur::check(document, report);
    // ADR-0167 §8 and ADR-0168 §3: a projection's `perspective` missing or alone, the eye
    // bound, strong foreshortening, and an element that never faces the eye. Document-only.
    crate::checks::projection::check(document, report);
    // ADR-0169's three `audio_effects` rules: a member in the other vocabulary's list, a
    // second enabled copy of a singular member, and a member left bypassed. Document-only.
    crate::checks::audio_effects::check(document, report);
    // ADR-0179 §3's five EQ findings: a range, the stack cap, an extreme gain, a crossed
    // band pair and a zero-gain stage. Document-only.
    crate::checks::audio_eq::check(document, report);
    // ADR-0180's six compressor and limiter findings: a range, a limiter over the master's
    // ceiling, the order, the make-up clip, a ratio of 1 and stacked limiters. Document-only.
    crate::checks::dynamics::check(document, report);
    // ADR-0178's two document-only loudness-normalisation findings: a target out of range,
    // and a target above the master's. Its third, a source with no sound, is on the disk.
    crate::checks::normalize::check(document, report);
    // ADR-0156's `R-GRAIN-SEED-SHARED`: two grains drawing one pattern. Document-only,
    // decided on the frame grid without painting a frame.
    crate::checks::grain::check(document, report);
    // ADR-0088's three document-only `chroma` findings (#342): a colour operation ahead of
    // the key in the same ordered list, a key on pixels the format itself authored, and a
    // `tolerance` sitting on its identity value. Its fourth finding needs the probe and is
    // in `run_disk_checks` below — one module, two call sites, because what a check *reads*
    // is what decides where it runs, not which ADR it comes from.
    crate::checks::chroma::check(document, report);
    // Not "which boundaries are off the grid" — which the fixture answers 109 times — but
    // what the grid actually changes, which on a correct project is nothing (ADR-0006).
    crate::checks::quantization::check(document, report);
    // ADR-0146 §6: a box with no positive size at any frame of its range is never painted.
    // Decided here and in the painter through one function at the same frame instants.
    crate::checks::extent::check(document, report);
    // ADR-0172's four `master` reviews: a target with no ceiling, an unusual target, a
    // ceiling above lossy-delivery guidance, and too little headroom. Document-only.
    crate::checks::master::check(document, report);
    // ADR-0041: checked here **unconditionally**, and `fmt --check`-only was rejected
    // outright — the agent that pretty-printed the fixture from 154 lines to 1595 was not
    // running a formatter and had no reason to invoke one, while `validate` runs on files
    // nobody ever ran `fmt` over. Its findings are `LAYOUT`, which is not a severity: they
    // never gate a render, because the video is byte-identical either way.
    crate::checks::layout::check(document, report);
    // ADR-0057's two attestation checks (#207): every `fonts`-table path resolves to a
    // `fontVendor` entry whose hash matches the bytes on disk, and every entry is still
    // referenced. Reads the disk — the font files — but needs no subprocess, so it sits
    // here rather than behind the session below, and it runs whether or not the project
    // references any media.
    // Also `E-FONT-NO-GLYPH`, `N-FONT-CENSUS` and `R-FONT-SWAP` (#206, ADR-0007). The last
    // of those is the one call in this list that reads the *cache* rather than the project:
    // a chain's identity is compared against what the last run recorded, which is how a font
    // swapped in place — "a silent whole-project render change that no census sees" —
    // becomes visible at all.
    crate::checks::fonts::check(document, cache, report);
    // ADR-0087's `R-LINE-INK-COLLISION` (#325): one line's real ink reaching past where the
    // next line's begins. It opens the same font files the call above does — the ink is in
    // them and nowhere else — and like that one it needs no subprocess, so it sits here
    // rather than behind the session below.
    //
    // It shapes every text element in the project, which is the most expensive thing
    // `validate` does without a subprocess. That buys it no place in this list — the
    // findings are a set and nothing downstream reads their order — and it is named only so
    // the next reader wondering where a slow run went does not have to measure to find out.
    crate::checks::ink::check(document, report);
    // ADR-0112: the document half has completed, and its findings stand whatever the disk
    // half does next — so it is recorded here, before anything below can stop the run.
    report.record(CheckSet::Document);

    // The two checks that need a subprocess, and the only ones that can fail rather than
    // find. Whether one needs to be opened at all is decided once, here, rather than per
    // check: `crate::checks::fit` needs the identical session `crate::checks::source`
    // does, and a project referencing no media has nothing to ask either of them.
    if !document.elements().any(|e| e["source"].is_string()) {
        // ADR-0112 §4: the disk half was entered, found nothing to probe, and completed.
        // A no-media run is complete on the wire because it is complete in fact.
        report.record(CheckSet::Disk);
        return Ok(());
    }
    let mut opened;
    let session = match session {
        Some(session) => session,
        None => {
            // The same cache file the font half above just wrote: one run, one sidecar.
            opened = Session::open_at(cache.map(Path::to_path_buf)).map_err(Box::new)?;
            opened.begin_run();
            &mut opened
        }
    };
    run_disk_checks(document, session, report)?;
    report.record(CheckSet::Disk);
    // ADR-0115: an `ffmpeg` that failed the tool qualification is an `error` here, because
    // `render` is guaranteed to refuse. After the disk half rather than instead of it — that
    // half needs only `ffprobe` — and in no check set: it is refusal-shaped, not a question
    // asked of the project (ADR-0112).
    if let Some(unqualified) = session.unqualified() {
        unqualified.fail(report);
    }
    Ok(())
}

/// The checks that read the disk, over the one session both share.
///
/// `source` first: it is what populates `report.misses`, and `fit` re-probing the same
/// sources afterwards is a cache hit that adds none — running them in the other order
/// would leave `fit`'s partial view overwriting `source`'s complete one.
fn run_disk_checks(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    crate::checks::source::check(document, session, report)?;
    crate::checks::fit::check(document, session, report)?;
    // `R-CHROMA-ON-ALPHA-SOURCE` (ADR-0088): one `ffprobe` field, off a probe the call
    // above has already cached. Last for the same reason `fit` follows `source` — every
    // probe it needs is a cache hit by the time it runs, so it adds no subprocess.
    crate::checks::chroma::on_disk(document, session, report)?;
    // ADR-0176: whether a transition's bridged sources carry sound, off the same cached probes.
    crate::checks::transition::on_disk(document, session, report)?;
    // ADR-0178: a normalised element whose source carries no sound, off the same probes.
    crate::checks::normalize::on_disk(document, session, report)
}

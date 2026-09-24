//! The disk half of `validate`: *"does this document agree with the media on disk?"*
//!
//! ADR-0006 settles the cost model once, and it is the whole shape of this module:
//! **no fast mode, no `--no-probe`, no scoping of what is checked.** Unanimous, 5 of 5.
//! *"The moment a fast path exists it becomes the mode used in the edit loop, so the single
//! highest-value check in the tool surface is the one that gets skipped."* Scoping the
//! probe to "the sources this write touched" was rejected on the same page and for a
//! sharper reason: in the fixture **the truth changed on disk, not in the project**, and a
//! touched-sources scope would never re-probe it. So every referenced source is probed on
//! every run, and there is no argument to this function that could narrow it.
//!
//! What the probe establishes is [#190](https://github.com/MBehtemam/Montagent/issues/190)'s
//! ([`crate::media`]); what it *means for the document* is here.
//!
//! # The line this module draws
//!
//! ADR-0053 and ADR-0056 between them fix four outcomes, and the whole value of the report
//! is that they stay four rather than collapsing into two:
//!
//! | what was established | finding |
//! | --- | --- |
//! | a *confirmed* absence — no such local file, or a server refusing the object | `E-SOURCE-MISSING`, plain `error` |
//! | the source is there, and its declared range names bytes it does not hold | `E-SOURCE-OVERRUN`, refuse-class `error` |
//! | existence confirmed, content not | `U-SOURCE-EXISTENCE-ONLY` |
//! | nothing learned — timeout, DNS, unreachable, a status | `U-SOURCE-UNPROBEABLE` + ADR-0056's structured reason |
//!
//! *"I could not look"* and *"it is not there"* are different facts, and ADR-0056 is
//! explicit that they *"must never be conflated in either direction"*.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::media::Source;
use crate::media::probe::Probe;
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::permissive::Loose;
use crate::report::Report;

/// Probe every source the document references, and report what the disk says.
///
/// The error is ADR-0011's exit 70: there is no `ffprobe`, so the disk half of the
/// question cannot be asked at all. Answering it with silence — or with a run that quietly
/// checked less — is the one thing ADR-0006 forbids.
///
/// Takes an already-open session rather than an `Option`: whether a project references
/// any media at all, and therefore whether a session is worth opening in the first place,
/// is `crate::verbs::validate`'s own call — `crate::checks::fit` needs the identical
/// session for the same reason, and *"a project referencing no media has nothing to ask a
/// subprocess"* is a decision made once, not per check.
pub fn check(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    probe_every_source(document, session, report)
}

fn probe_every_source(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    // ADR-0053: a relative `source` resolves against the directory the project file lives
    // in, and nothing else. There is no `assetRoot` and no flag.
    let base = crate::checks::project_dir(document);

    for element in document.elements() {
        let Some(source) = element["source"].as_str() else {
            continue;
        };
        let id = element["id"].as_str().unwrap_or("<no id>");

        let outcome = session.probe(&Source::resolve(source, &base))?;
        let before = report.findings.len();
        // The four outcomes ADR-0053 and ADR-0056 fix, and the facts where there is no
        // defect — one mapping, shared with the `probe` verb.
        crate::media::probe::record(&outcome, source, report);
        locate(report, before, document.path(), id);

        // Only a source that actually answered can be overrun.
        if let Some(finding) = outcome
            .probe()
            .and_then(|probe| overrun(element, probe, source))
        {
            report.push(finding.at_file(document.path()).at_element(id));
        }
    }

    // ADR-0006's cache-miss line, on the report rather than behind a flag. Every probe that
    // actually ran says so, and a source that changed says *that*.
    report.misses = session.misses().to_vec();
    Ok(())
}

/// Put the element's own location on whatever the shared mapping just appended.
///
/// The mapping is about a *source*; a finding is about an *element that references one*,
/// and the location is what makes it actionable without a re-read (ADR-0006).
fn locate(report: &mut Report, before: usize, file: &str, element: &str) {
    for finding in report.findings.iter_mut().skip(before) {
        finding.location.file = file.to_string();
        finding.location.element = Some(element.to_string());
    }
}

/// Does the declared source range name bytes the file does not hold?
///
/// `source_end` is exclusive (ADR-0005's half-open interval), so a range ending exactly at
/// the source's last millisecond is legal and only a range reaching *past* it is the
/// defect.
fn overrun(element: &Value, probe: &Probe, declared: &str) -> Option<Finding> {
    let source_start = element["source_start"].as_i64()?;
    let source_end = element["source_end"].as_i64()?;
    let axis = Axis::for_element(element["type"].as_str());
    let available = axis.available_ms(probe)?;
    if source_end <= available {
        return None;
    }

    // ADR-0006 names what a finding must carry: *"every number inline — especially the
    // numbers that are not in the file: the probed duration, the delta"*. That is the
    // file's length, the range declared against it, and how far past it reaches.
    //
    // **`speed` is deliberately absent, and so is the timeline span it implies.** Whether a
    // source range names bytes the file holds is a question about the source alone: the
    // same range overruns by the same amount at any rate. The timeline consequence belongs
    // to ADR-0020's invariant, which ADR-0045 then requires be evaluated in exact rational
    // arithmetic *"never from a `float`/`f64` intermediate"* — and that invariant is
    // #197's check, evaluated once, there. Restating it here in `f64` produced a second
    // number that could disagree with the first: on ADR-0045's own minimal case, a 7 ms
    // span at `speed: 0.560`, this printed 12 where exact arithmetic gives 13. A finding
    // whose whole value is that its inline numbers are true cannot carry a number computed
    // by the one method an ADR forbids.
    Some(
        Finding::new("E-SOURCE-OVERRUN")
            .field("source", json!(declared))
            .field("probed_duration", json!(available))
            .field("axis", json!(axis.name(probe)))
            .field("source_start", json!(source_start))
            .field("source_end", json!(source_end))
            .field("declared_source_span", json!(source_end - source_start))
            .field("over_by", json!(source_end - available)),
    )
}

/// Which of ADR-0011's four durations an element's declared range is measured against.
///
/// ADR-0011 returns the quad and *"forces the caller to pick"*, so this is the pick. It is
/// made **per element type**, not per file, because the same file can answer differently
/// on each axis: the fixture's reference MP4 holds 65216 ms of video and 65258 ms of audio.
/// An `audio` element pointing at it is playing the audio stream, and measuring that
/// against the video stream would report an overrun on 42 ms the renderer can decode
/// perfectly well.
///
/// The stream is preferred over the container on both axes for the same reason:
/// `source_start`/`source_end` name a range of *decodable content* — what the renderer will
/// seek into — while the container's duration is the length of the muxed file, which may
/// include a tail no stream carries. It is also the axis the fixture's own `duration: 65216`
/// agrees with, against the container's 65258.
///
/// The container is the fallback rather than the primary for the same reason it is not the
/// primary: it answers a slightly different question, and answering with it silently would
/// be the false confidence ADR-0006 is written against. A source that offers only a
/// container duration is measured against it, and the finding says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    /// `type: "audio"` — the sound is the content.
    Audio,
    /// Everything else that carries a source range. `image` has no range to overrun.
    Video,
}

impl Axis {
    fn for_element(element_type: Option<&str>) -> Axis {
        match element_type {
            Some("audio") => Axis::Audio,
            _ => Axis::Video,
        }
    }

    /// This element's own axis, where the file states it.
    fn stream_ms(self, probe: &Probe) -> Option<i64> {
        match self {
            Axis::Audio => probe.audio.and_then(|audio| audio.audio_stream_ms),
            Axis::Video => probe.quad.video_stream_ms,
        }
    }

    /// The other one — because a `video` element pointed at an audio-only file should be
    /// measured against the stream that is actually there rather than fall straight through
    /// to the container, which is the weaker claim and may include a muxing tail.
    fn other(self) -> Axis {
        match self {
            Axis::Audio => Axis::Video,
            Axis::Video => Axis::Audio,
        }
    }

    fn available_ms(self, probe: &Probe) -> Option<i64> {
        self.stream_ms(probe)
            .or_else(|| self.other().stream_ms(probe))
            .or(probe.quad.container_ms)
    }

    /// What the finding calls the number it used. Never a guess: whichever branch of
    /// [`Axis::available_ms`] answered is the one named.
    fn name(self, probe: &Probe) -> &'static str {
        if self.stream_ms(probe).is_some() {
            return self.spelling();
        }
        if self.other().stream_ms(probe).is_some() {
            return self.other().spelling();
        }
        "container"
    }

    fn spelling(self) -> &'static str {
        match self {
            Axis::Audio => "audio stream",
            Axis::Video => "video stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::Rational;
    use crate::media::probe::{Audio, Quad};

    fn probe(quad: Quad, audio: Option<Audio>) -> Probe {
        Probe {
            source: "s".into(),
            quad,
            dimensions: None,
            alpha: None,
            codec_name: None,
            audio,
        }
    }

    #[test]
    fn a_video_is_measured_against_its_video_stream() {
        // The fixture's reference MP4: the two durations differ by 42 ms, and the
        // project's own `duration: 65216` agrees with the stream.
        let probe = probe(
            Quad {
                video_stream_ms: Some(65216),
                container_ms: Some(65258),
                start_time_ms: Some(42),
                r_frame_rate: Some(Rational::new(50, 1)),
                avg_frame_rate: Some(Rational::new(1_390_080, 55_651)),
            },
            Some(Audio {
                sample_rate: Some(24000),
                channels: Some(1),
                audio_stream_ms: Some(65258),
            }),
        );

        let axis = Axis::for_element(Some("video"));
        assert_eq!(axis.available_ms(&probe), Some(65216));
        assert_eq!(axis.name(&probe), "video stream");

        // The same file, referenced by an `audio` element: it is playing the sound, and
        // the sound is 42 ms longer than the picture. Measuring it against the video
        // stream would report an overrun on audio the renderer decodes perfectly well.
        let audio = Axis::for_element(Some("audio"));
        assert_eq!(audio.available_ms(&probe), Some(65258));
        assert_eq!(audio.name(&probe), "audio stream");
    }

    #[test]
    fn an_audio_file_is_measured_against_its_audio_stream_not_its_container() {
        // Unexercised by any committed media and unit-tested for exactly that reason: all
        // eleven fixture mp3s show no split (ADR-0011: "format == stream, start_time 0"),
        // so the *ordering* between these two branches is invisible end to end. A
        // container that outruns its stream is ordinary in other containers, and the axis
        // that matters is the decodable content either way.
        let probe = probe(
            Quad {
                video_stream_ms: None,
                container_ms: Some(2000),
                ..Quad::default()
            },
            Some(Audio {
                audio_stream_ms: Some(1776),
                ..Audio::default()
            }),
        );

        let axis = Axis::for_element(Some("audio"));
        assert_eq!(axis.available_ms(&probe), Some(1776));
        assert_eq!(axis.name(&probe), "audio stream");
    }

    #[test]
    fn a_source_that_states_only_a_container_duration_is_measured_against_it_and_says_so() {
        let probe = probe(
            Quad {
                container_ms: Some(2000),
                ..Quad::default()
            },
            None,
        );

        let axis = Axis::for_element(Some("audio"));
        assert_eq!(axis.available_ms(&probe), Some(2000));
        assert_eq!(
            axis.name(&probe),
            "container",
            "the finding names the weaker axis rather than presenting it as the stream's"
        );
    }

    #[test]
    fn an_element_is_measured_against_the_stream_that_exists_before_the_container() {
        // A `video` element pointed at an audio-only file: its own axis is silent, and the
        // stream that is actually there is a better answer than the container, which may
        // carry a muxing tail the decoder never reaches.
        let probe = probe(
            Quad {
                video_stream_ms: None,
                container_ms: Some(2000),
                ..Quad::default()
            },
            Some(Audio {
                audio_stream_ms: Some(1776),
                ..Audio::default()
            }),
        );

        let axis = Axis::for_element(Some("video"));
        assert_eq!(axis.available_ms(&probe), Some(1776));
        assert_eq!(
            axis.name(&probe),
            "audio stream",
            "and it says which one it fell back to, rather than naming its own"
        );
    }

    #[test]
    fn a_still_image_has_no_length_to_overrun() {
        // No duration on any axis: an image is not a range anything can reach past, so the
        // check has nothing to say rather than something weak to say.
        assert_eq!(
            Axis::for_element(Some("image")).available_ms(&probe(Quad::default(), None)),
            None
        );
    }
}

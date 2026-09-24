//! The proxy ladder `preview` climbs down, and the two floors it stops at.
//!
//! **This module is arithmetic and refusal text. It renders nothing** — the verb
//! (`montagent-core`'s `preview`) walks the ladder and this module says what each
//! rung is, whether a rung may be admitted, and what a refusal has to say. Keeping it
//! here rather than in the verb is [ADR-0021]'s split doing its job: the rasterizer owns
//! what a proxy frame *is*, the verb owns when to ask for one.
//!
//! ## One target, one degrade step, then a refusal
//!
//! ```text
//!   720p   long edge <= 1280 px   the default every caller hits   (ADR-0046)
//!     |    a miss against the <5 s scrub budget degrades exactly once
//!   540p   long edge <=  960 px   the one degrade tier, disclosed  (ADR-0065)
//!     |    a miss here is not degraded again
//!   hard fail
//! ```
//!
//! There is no `360p` rung and adding one is not a small change: [ADR-0065] measured
//! `720p -> 540p -> 360p` as buying under 1 s combined at 4K and ~1.06 s at 8K, against a
//! 5 s budget — *"measurable, not load-bearing, for a mechanism whose whole point is a
//! large win"*.
//!
//! **One degrade step does not always rescue a miss, and that is the design.** At 8K the
//! step buys ~0.71 s, because the cost that remains is decode, which is driven by the
//! *source* size and does not shrink when the target does. A project missing 720p by more
//! than that lands on the hard fail; a ladder that claimed otherwise would be asserting a
//! capacity the measurements do not support.
//!
//! The `<5 s` guarantee behind all of this is `skia-safe`-specific. `tiny-skia` is
//! explicitly **not** certified at 8K/720p (ADR-0065) — it was measured once, at a 1.8%
//! margin the source findings refuse to certify.
//!
//! ## The two floors are two different refusals
//!
//! [ADR-0067] separates what one word was doing twice, and this module keeps them as two
//! constants on purpose — [`WALL_CLOCK_GIVE_UP_LONG_EDGE`] and
//! [`LEGIBILITY_FLOOR_LONG_EDGE`]:
//!
//! | | 540p | 360p |
//! | --- | --- | --- |
//! | the question | where does the **ladder give up** rather than degrade again? | below what resolution is a frame **not worth looking at**? |
//! | triggered by | a *time* miss one degrade step could not rescue | a *resolution* that loses the picture |
//! | the evidence | [#87]'s savings curve | [#117]'s legibility pass on the real fixture |
//!
//! **360p does not fire today, and that is stated rather than hidden.** The ladder stops
//! at 540p, so degradation never reaches it: [`admit`] is a guard on any future rung and
//! on any caller-specified proxy resolution, should one ever be admitted, not a live
//! branch. Collapsing the two into one number would lose a measurement that was expensive
//! to produce, and would make the next person to extend the ladder re-derive it.
//!
//! **The floor governs a proxy resolution, never the author's declared frame.** A project
//! that declares a 320x180 frame is previewed at its own pixels: nothing downscaled it, so
//! there is no proxy for the floor to judge, and `preview` is not the verb that gets to
//! tell an author their project is too small. [ADR-0078] ratifies this, bounding ADR-0050's
//! *"any resolution request below 360x640-equivalent is a refusal"* to a **proxy**
//! resolution.
//!
//! [ADR-0021]: ../../../../docs/adr/0021-preview-budget-and-graceful-degradation.md
//! [ADR-0046]: ../../../../docs/adr/0046-proxy-preview-target-is-720p-long-edge-capped.md
//! [ADR-0065]: ../../../../docs/adr/0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md
//! [ADR-0067]: ../../../../docs/adr/0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md
//! [ADR-0078]: ../../../../docs/adr/0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md
//! [#87]: https://github.com/MBehtemam/Montagent/issues/87
//! [#117]: https://github.com/MBehtemam/Montagent/issues/117

/// The proxy target: the project's **longer** edge (width or height, whichever is larger)
/// scaled to at most this many pixels, aspect preserved, both dimensions rounded to even
/// (ADR-0046).
///
/// A cap, not a scale factor, and not the landscape-era height convention. Both
/// alternatives were considered and rejected by ADR-0046: a scale factor *"doesn't bound
/// anything at the high end"* (half of 16K is still 8K), and a height cap makes "720p"
/// mean two different things depending on orientation — ~405x720 for the 9:16 fixture,
/// 44% fewer pixels than the long-edge cap and a different performance point than the one
/// that was actually measured.
pub const TARGET_LONG_EDGE: i64 = 1280;

/// **The wall-clock give-up point.** The one degrade tier below the target, 540p, and the
/// last rung on the ladder: a miss here is a refusal, not another degrade (ADR-0065).
///
/// This is a statement about *time*. It is not [`LEGIBILITY_FLOOR_LONG_EDGE`], it does not
/// mean 540p is where a frame stops being readable, and the two may not be collapsed
/// (ADR-0067). 540p is in fact two tiers clear of the measured legibility break point —
/// what ADR-0067 retires is ADR-0065's *"its legibility is explicitly unmeasured"* hedge,
/// not the give-up point itself, which stands on the wall-clock argument it was always
/// resting on.
pub const WALL_CLOCK_GIVE_UP_LONG_EDGE: i64 = 960;

/// **The legibility floor.** Below this, a frame stops being worth looking at, and
/// `preview` refuses rather than hand back something it cannot be read from (ADR-0050).
///
/// This is a statement about *pixels*, measured rather than argued: the committed fixture
/// rendered at 720p/540p/360p/240p at four timestamps chosen to stress its smallest
/// declared text, judged by three independent jurors who unanimously located the break
/// point between 360p and 240p. What forces it is the 34-35 px handle/chip badge and the
/// smallest caption line, which read cleanly through 360p and at 240p lose their contour
/// to a smear an agent can only "read" by already knowing the string — which is exactly
/// the position an agent checking its own unfamiliar render is not in. The 55 px subtitle
/// cards survive 240p and do not force the floor.
///
/// **It cannot fire under today's ladder**, which stops at [`WALL_CLOCK_GIVE_UP_LONG_EDGE`].
/// It is kept because it is a real measurement and it governs the two live cases ADR-0067
/// names: any future extension of the ladder, and any caller-specified proxy resolution.
pub const LEGIBILITY_FLOOR_LONG_EDGE: i64 = 640;

/// One rung of the ladder, or the true pixels that are not on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// True pixels. Either the caller took ADR-0021's explicit full-resolution escape
    /// hatch, or the project is already inside the cap and no proxy engages. Never
    /// reached by degrading: the escape hatch is the observational arm of the budget, and
    /// a caller who asked for true pixels and accepted the cost is not handed fewer.
    Native,
    /// The 720p proxy target (ADR-0046) — the default every caller hits.
    Target,
    /// The one degrade tier, 540p (ADR-0065), reached only from [`Tier::Target`] on a
    /// budget miss, and always disclosed (ADR-0021).
    Degraded,
}

impl Tier {
    /// The long-edge cap this tier scales to, or `None` for true pixels.
    pub fn cap(self) -> Option<i64> {
        match self {
            Tier::Native => None,
            Tier::Target => Some(TARGET_LONG_EDGE),
            Tier::Degraded => Some(WALL_CLOCK_GIVE_UP_LONG_EDGE),
        }
    }

    /// The name the result discloses this tier under.
    pub fn name(self) -> &'static str {
        match self {
            Tier::Native => "native",
            Tier::Target => "720p",
            Tier::Degraded => "540p",
        }
    }

    /// Whether this tier is a degradation from the default, which is the fact ADR-0021
    /// makes the result carry mandatorily.
    pub fn is_degraded(self) -> bool {
        matches!(self, Tier::Degraded)
    }

    /// The next rung down after a budget miss, or `None` where the ladder gives up.
    ///
    /// Exactly one step exists, and both of the `None` arms are deliberate: the ladder
    /// hard-fails below 540p rather than degrading again (ADR-0065), and the
    /// full-resolution escape hatch is never degraded at all — it is the unenforced arm
    /// of the budget (ADR-0021), so there is no miss for a step to answer.
    pub fn next(self) -> Option<Tier> {
        match self {
            Tier::Target => Some(Tier::Degraded),
            Tier::Native | Tier::Degraded => None,
        }
    }

    /// The surface `width` x `height` is rasterized on at this tier.
    ///
    /// [`Frame::proxied`] is false where the cap never engaged — a project already inside
    /// it, or [`Tier::Native`] — and the caller discloses true pixels rather than a proxy
    /// that did nothing.
    pub fn frame(self, width: i64, height: i64) -> Frame {
        match self.cap() {
            Some(cap) => fit(width, height, cap),
            None => Frame {
                width,
                height,
                proxied: false,
            },
        }
    }
}

/// A surface to rasterize on: the dimensions, and whether the cap engaged to produce them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub width: i64,
    pub height: i64,
    /// Whether these dimensions are smaller than the project's declared frame.
    pub proxied: bool,
}

impl Frame {
    /// The longer edge, which is the edge every cap and both floors are stated on.
    pub fn long_edge(self) -> i64 {
        self.width.max(self.height)
    }

    /// The scale from project coordinates to this surface, per axis.
    ///
    /// Two numbers rather than one: rounding each edge to even can leave the two ratios
    /// differing by a fraction of a pixel, and painting at one averaged factor would put
    /// that error into every element's position instead of into the frame's own edge.
    pub fn scale(self, width: i64, height: i64) -> (f64, f64) {
        let axis = |to: i64, from: i64| {
            if from <= 0 {
                1.0
            } else {
                to as f64 / from as f64
            }
        };
        (axis(self.width, width), axis(self.height, height))
    }
}

/// Scale `width` x `height` so its longer edge is at most `cap`, aspect preserved, both
/// dimensions rounded to the nearest even integer (ADR-0046).
///
/// A frame already inside the cap comes back untouched and unproxied: *"for a 1280x720-native
/// or smaller project, no proxy applies at all — the cap only engages above it"*. Even
/// dimensions because the encoder demands them — `render` pads an odd declared frame and
/// discloses it, and a proxy that produced an odd frame would be asking for that padding
/// on a number Montagent chose itself rather than one the author declared.
pub fn fit(width: i64, height: i64, cap: i64) -> Frame {
    let long = width.max(height);
    if width <= 0 || height <= 0 || cap <= 0 || long <= cap {
        return Frame {
            width,
            height,
            proxied: false,
        };
    }
    let factor = cap as f64 / long as f64;
    let even = |edge: i64| {
        let scaled = (edge as f64 * factor / 2.0).round() * 2.0;
        // Floored at 2 rather than 0: a zero-sized edge is not a smaller picture, it is
        // no picture, and the encoder refuses it.
        (scaled as i64).max(2)
    };
    Frame {
        width: even(width),
        height: even(height),
        proxied: true,
    }
}

/// The guard on [`LEGIBILITY_FLOOR_LONG_EDGE`]: may a proxy of this long edge be handed
/// back at all?
///
/// `Ok` for every rung of today's ladder — it cannot fire, by construction, and
/// ADR-0067 asks for it to exist anyway. `Err` carries the sentence the refusal is made
/// of, which names the floor **and** the reason: a bare failure below 360p is
/// indistinguishable from a render error, a bad timestamp or a missing asset, and a
/// caller given no explanation retries the identical call or abandons `preview` (ADR-0050).
pub fn admit(long_edge: i64) -> Result<(), String> {
    if long_edge >= LEGIBILITY_FLOOR_LONG_EDGE {
        return Ok(());
    }
    Err(format!(
        "`preview` refused: a {long_edge} px long edge is below the 360p floor \
         ({LEGIBILITY_FLOOR_LONG_EDGE} px, long-edge capped, aspect preserved); below this, \
         on-screen text is not legible enough to judge a cut from"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_floors_are_two_numbers() {
        // ADR-0067's whole point. A test rather than a comment because the failure mode
        // is someone tidying one of them away.
        assert_ne!(WALL_CLOCK_GIVE_UP_LONG_EDGE, LEGIBILITY_FLOOR_LONG_EDGE);
        // In `const` blocks so the ordering is checked where the constants are, not only
        // when this test runs.
        const { assert!(WALL_CLOCK_GIVE_UP_LONG_EDGE > LEGIBILITY_FLOOR_LONG_EDGE) };
        const { assert!(TARGET_LONG_EDGE > WALL_CLOCK_GIVE_UP_LONG_EDGE) };
    }

    #[test]
    fn the_fixtures_frame_lands_on_adr_0046s_measured_numbers() {
        // 1080x1920 portrait, the committed fixture and #87's harness: 720x1280 at the
        // target, exactly the frame the measurements were taken at.
        assert_eq!(
            Tier::Target.frame(1080, 1920),
            Frame {
                width: 720,
                height: 1280,
                proxied: true
            }
        );
        // And the one degrade step, 540p for the same aspect.
        assert_eq!(
            Tier::Degraded.frame(1080, 1920),
            Frame {
                width: 540,
                height: 960,
                proxied: true
            }
        );
    }

    #[test]
    fn the_cap_is_on_the_long_edge_whichever_edge_that_is() {
        // Landscape 4K: the width is the long edge.
        assert_eq!(
            Tier::Target.frame(3840, 2160),
            Frame {
                width: 1280,
                height: 720,
                proxied: true
            }
        );
        // The same project rotated: the same pixel count, the height capped instead.
        assert_eq!(
            Tier::Target.frame(2160, 3840),
            Frame {
                width: 720,
                height: 1280,
                proxied: true
            }
        );
    }

    #[test]
    fn an_unusual_ratio_keeps_its_aspect_and_comes_back_even() {
        for (width, height) in [(4096, 1716), (1600, 1200), (5000, 3), (1281, 1280)] {
            let frame = Tier::Target.frame(width, height);
            assert_eq!(frame.long_edge(), TARGET_LONG_EDGE, "{width}x{height}");
            assert_eq!(frame.width % 2, 0, "{width}x{height} width is even");
            assert_eq!(frame.height % 2, 0, "{width}x{height} height is even");
            assert!(frame.width >= 2 && frame.height >= 2, "{width}x{height}");
            // Aspect preserved to within the rounding the even-dimension rule costs.
            let native = width as f64 / height as f64;
            let proxy = frame.width as f64 / frame.height as f64;
            assert!(
                (native - proxy).abs() / native < 0.02 || frame.height == 2,
                "{width}x{height} became {}x{}",
                frame.width,
                frame.height
            );
        }
    }

    #[test]
    fn a_project_already_inside_the_cap_is_not_proxied() {
        // ADR-0046: "for a 1280x720-native or smaller project, no proxy applies at all".
        for (width, height) in [(1280, 720), (720, 1280), (200, 200), (1, 1)] {
            let frame = Tier::Target.frame(width, height);
            assert_eq!(
                frame,
                Frame {
                    width,
                    height,
                    proxied: false
                }
            );
        }
        // And the degrade tier still engages for a project between the two caps, which is
        // what makes it a real second rung rather than a second name for the first.
        assert_eq!(
            Tier::Degraded.frame(1000, 1000),
            Frame {
                width: 960,
                height: 960,
                proxied: true
            }
        );
    }

    #[test]
    fn native_is_true_pixels_at_any_size() {
        assert_eq!(
            Tier::Native.frame(4320, 7680),
            Frame {
                width: 4320,
                height: 7680,
                proxied: false
            }
        );
    }

    #[test]
    fn the_ladder_degrades_exactly_once_and_then_gives_up() {
        assert_eq!(Tier::Target.next(), Some(Tier::Degraded));
        assert_eq!(Tier::Degraded.next(), None);
        // The escape hatch is the unenforced arm: there is no miss for a step to answer.
        assert_eq!(Tier::Native.next(), None);
        // Which is the whole ladder: two rungs, no third.
        let mut rungs = vec![Tier::Target];
        while let Some(next) = rungs.last().unwrap().next() {
            rungs.push(next);
            assert!(rungs.len() <= 2, "the ladder grew a third rung");
        }
        assert_eq!(rungs, vec![Tier::Target, Tier::Degraded]);
    }

    #[test]
    fn only_the_degrade_tier_discloses_a_degradation() {
        assert!(Tier::Degraded.is_degraded());
        assert!(!Tier::Target.is_degraded());
        assert!(!Tier::Native.is_degraded());
    }

    #[test]
    fn every_rung_of_todays_ladder_clears_the_legibility_floor() {
        // The guard cannot fire as the ladder stands (ADR-0067), and this is that claim
        // stated as a test rather than as a sentence.
        for tier in [Tier::Target, Tier::Degraded] {
            let frame = tier.frame(1080, 1920);
            assert!(admit(frame.long_edge()).is_ok(), "{}", tier.name());
        }
    }

    #[test]
    fn a_rung_below_the_floor_is_refused_naming_the_floor_and_the_reason() {
        // 360p itself is admitted — the floor is the last legible tier, and the refusal
        // is for what is *below* it.
        assert!(admit(LEGIBILITY_FLOOR_LONG_EDGE).is_ok());
        // 240p, the tier the jurors located the break point at.
        let refusal = admit(fit(1080, 1920, 426).long_edge()).expect_err("below the floor");
        assert!(refusal.contains("360p floor"), "{refusal}");
        assert!(refusal.contains("640"), "{refusal}");
        assert!(refusal.contains("legible"), "{refusal}");
        assert!(refusal.contains("judge a cut from"), "{refusal}");
    }

    #[test]
    fn the_scale_is_per_axis_and_is_one_where_nothing_was_scaled() {
        let frame = Tier::Target.frame(1080, 1920);
        assert_eq!(frame.scale(1080, 1920), (720.0 / 1080.0, 1280.0 / 1920.0));
        let native = Tier::Native.frame(1080, 1920);
        assert_eq!(native.scale(1080, 1920), (1.0, 1.0));
    }
}

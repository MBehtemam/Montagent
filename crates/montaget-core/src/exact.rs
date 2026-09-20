//! Exact arithmetic: the places the format puts a boundary-sensitive operation in front
//! of a value that must not reach it through `f64`.
//!
//! Two ADRs put a division in front of a rounding boundary, and both say the same thing
//! about how it is evaluated:
//!
//! - **ADR-0045**: `speed`'s invariant, `end - start == round(source_span / speed)`, is
//!   evaluated *"as exact rational/decimal arithmetic ... never from a `float`/`f64`
//!   intermediate"*. Its measured case is `7 / 0.560`, which is exactly `12.5` in decimal
//!   and so rounds half-up to `13` — where `f64` computes `12.499999999999998` and gives
//!   `12`.
//! - **ADR-0035**: the frame grid is `1000/fps` ms, *"not necessarily integral"*, and the
//!   sampled-instant formula is to be evaluated *"in exact rational arithmetic (integer
//!   numerator/denominator, never float)"* — at `fps = 30` the step is `100/3` ms and only
//!   multiples of 100 ms are frame-exact.
//!
//! One module for both, because they are one obligation: a rounding boundary reached
//! through a division, evaluated on integers. Nothing here returns a float, and nothing
//! here takes one.
//!
//! A third, [`Decimal::tenths`] and [`text_block_height`], is the same obligation reached
//! through a *multiplication* instead: **ADR-0028**'s text-block height,
//! `ceil(size × line_height × line_count)`, evaluated as exact integer arithmetic on
//! `line_height`'s tenths rather than on the `f64` product the ADR measured diverging
//! from the exact value on 6.35% of sampled cases.
//!
//! # How the literal reaches this module
//!
//! ADR-0045 asks that *"the exact literal must reach the arithmetic as a string or an
//! exact type before any division happens"*, and names a parser that eagerly widens to
//! `f64` as insufficient **on its own**. [`Decimal::parse`] is the string door, and
//! [`Decimal::of`] is the door a `serde_json::Number` comes through.
//!
//! `serde_json`'s `arbitrary_precision` — the feature that would carry the literal's own
//! digits all the way here — is **not** enabled, and cannot be: it is documented as
//! incompatible with `#[serde(untagged)]`, which [`crate::model::keyframe`] uses for the
//! two record shapes the format publishes, so turning it on would break the strict parse
//! of every keyframe in the fixture. What reaches [`Decimal::of`] instead is the `Number`'s
//! own `Display`, which is the shortest decimal that round-trips the value — for every
//! finite decimal an author can write, that is the literal itself. The residual gap is a
//! literal carrying more significant digits than an `f64` can distinguish (`0.6450000000000000001`),
//! which no author writes and which the parser has already lost before any Montaget code
//! runs; it is raised as [#256](https://github.com/MBehtemam/Montaget/issues/256) rather
//! than left to be discovered here.
//!
//! What this module does **not** do is the thing ADR-0045 was written about: it never
//! divides in `f64`. The literal's digits become an exact rational and the division is
//! integer arithmetic from there.

/// A finite decimal, held exactly: `units × 10⁻ˢᶜᵃˡᵉ`.
///
/// `0.645` is `Decimal { units: 645, scale: 3 }` — the exact rational `645/1000` ADR-0045
/// says it already is, with no `f64` anywhere in its construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decimal {
    units: i128,
    scale: u32,
}

/// The widest decimal this module will hold, in **significant** digits — leading zeros do
/// not count against it.
///
/// Well past `f64`'s 17 and past anything the format's numbers reach: a `speed` of
/// `0.645` against a source span in milliseconds leaves the product `span × 10^scale`
/// many orders below `i128`'s range. A literal beyond it is refused rather than
/// silently truncated, because a truncated literal is exactly the wrong answer this
/// module exists to avoid.
const MAX_DIGITS: usize = 30;

impl Decimal {
    /// The decimal a string of digits spells, or `None` if it does not spell one.
    ///
    /// Accepts what JSON's number grammar produces, including the exponent form `ryu`
    /// reaches for on very small magnitudes.
    pub fn parse(literal: &str) -> Option<Decimal> {
        let literal = literal.trim();
        let (negative, rest) = match literal.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, literal.strip_prefix('+').unwrap_or(literal)),
        };

        let (mantissa, exponent) = match rest.split_once(['e', 'E']) {
            Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().ok()?),
            None => (rest, 0),
        };
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        if whole.is_empty() && fraction.is_empty() {
            return None;
        }
        if !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
        {
            return None;
        }

        let digits = format!("{whole}{fraction}");
        // Leading zeros are not significant — `0.0000000001` is one digit and a scale,
        // not eleven — so they are stripped before the width is judged. Counting them
        // would refuse a legal literal for being written with a long run of nothing.
        if digits.trim_start_matches('0').len() > MAX_DIGITS {
            return None;
        }
        let units: i128 = digits.parse().ok()?;
        // The exponent moves the point; a positive one takes the scale below zero, which
        // is not a decimal place but a multiplier.
        let scale = i32::try_from(fraction.len()).ok()? - exponent;
        let (units, scale) = if scale < 0 {
            (units.checked_mul(pow10(scale.unsigned_abs())?)?, 0)
        } else {
            (units, u32::try_from(scale).ok()?)
        };
        // The scale is bounded here rather than at the division. `1e-40` has one
        // significant digit and a scale `10^scale` cannot hold, and a `Decimal` that
        // parses but whose ratio overflows would reach the check as a rate it silently
        // declines to evaluate — a `speed` that is neither legal nor reported.
        pow10(scale)?;

        Some(Decimal {
            units: if negative { -units } else { units },
            scale,
        })
    }

    /// The decimal a JSON number holds.
    ///
    /// Integers travel as integers; everything else travels as the `Number`'s own
    /// `Display`, which is the module doc's one concession and its only one.
    pub fn of(number: &serde_json::Number) -> Option<Decimal> {
        if let Some(units) = number.as_i64() {
            return Some(Decimal {
                units: units as i128,
                scale: 0,
            });
        }
        Decimal::parse(&number.to_string())
    }

    /// Is this decimal strictly greater than zero? ADR-0020's condition on `speed`.
    pub fn is_positive(self) -> bool {
        self.units > 0
    }

    /// `numerator / denominator`, as the exact rational this decimal is.
    fn as_ratio(self) -> Option<(i128, i128)> {
        Some((self.units, pow10(self.scale)?))
    }

    /// **ADR-0028's `line_height`**: this decimal as `n` where the value is exactly
    /// `n/10` — recovered from the literal's own digits, never from `f64 × 10`, which
    /// ADR-0028 measured diverging from the exact value on 6.35% of sampled
    /// `(size, line_count)` pairs at `line_height = 1.1`.
    ///
    /// `None` if the value is not an exact multiple of `0.1` — ADR-0028 makes that a
    /// schema-time error belonging to another check, not a value for this function to
    /// round toward.
    pub fn tenths(self) -> Option<i64> {
        if self.scale <= 1 {
            i64::try_from(self.units.checked_mul(pow10(1 - self.scale)?)?).ok()
        } else {
            let divisor = pow10(self.scale - 1)?;
            (self.units % divisor == 0)
                .then(|| i64::try_from(self.units / divisor).ok())
                .flatten()
        }
    }
}

/// **ADR-0028's text-block height**: `ceil(size × line_height × line_count)`, evaluated
/// as exact integer arithmetic on `line_height`'s tenths —
/// `(size × line_height_tenths × line_count + 9) // 10` — never `f64`.
///
/// `None` where any input is not positive, or where the arithmetic would not fit; both
/// are schema facts belonging to another check, not a height for this function to guess.
pub fn text_block_height(size: i64, line_height_tenths: i64, line_count: i64) -> Option<i64> {
    if size <= 0 || line_height_tenths <= 0 || line_count <= 0 {
        return None;
    }
    let tenths = i128::from(size)
        .checked_mul(i128::from(line_height_tenths))?
        .checked_mul(i128::from(line_count))?;
    block_height_of_tenths(tenths)
}

/// The same `ceil`, over a block height already summed in tenths.
///
/// **This is the one implementation**, and [`text_block_height`] is the uniform-size case
/// of it: `size × line_height × line_count` is what `Σ (largest size on line ×
/// line_height)` comes to when every line carries one size. `measure` sums the per-line
/// slots itself — a line's height is *the largest `size` among the runs on that line* ×
/// `line_height` (ADR-0007), which the three-argument form cannot express once two lines
/// differ — and then arrives here, so the two never carry two `ceil`s that could disagree
/// about a boundary (ADR-0028's whole subject).
///
/// `None` where the height is not positive, or where the arithmetic would not fit.
pub fn block_height_of_tenths(tenths: impl Into<i128>) -> Option<i64> {
    let tenths = tenths.into();
    if tenths <= 0 {
        return None;
    }
    i64::try_from(ceil_div(tenths, 10)).ok()
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.scale == 0 {
            return write!(f, "{}", self.units);
        }
        let sign = if self.units < 0 { "-" } else { "" };
        let digits = self.units.unsigned_abs().to_string();
        let scale = self.scale as usize;
        let padded = format!("{:0>width$}", digits, width = scale + 1);
        let split = padded.len() - scale;
        write!(f, "{sign}{}.{}", &padded[..split], &padded[split..])
    }
}

fn pow10(exponent: u32) -> Option<i128> {
    10i128.checked_pow(exponent)
}

/// `numerator / denominator`, rounded half-up to an integer, on a positive denominator.
///
/// Round-half-up rather than round-half-even because ADR-0020 says so, and an exact tie
/// under it *"is not ambiguous — it has one correct answer"* (ADR-0045).
fn round_half_up(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator <= 0 {
        return None;
    }
    // `floor((2n + d) / 2d)`: the half-up tie-break folded into one floored division, so
    // there is no branch on the remainder to get wrong.
    let doubled = numerator.checked_mul(2)?.checked_add(denominator)?;
    let divisor = denominator.checked_mul(2)?;
    Some(floor_div(doubled, divisor))
}

fn floor_div(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator != 0 && (numerator < 0) != (denominator < 0) {
        quotient - 1
    } else {
        quotient
    }
}

/// **ADR-0020's invariant, evaluated exactly**: how long a source span of `source_span` ms
/// plays for at `speed`.
///
/// `round(source_span / speed)`, round-half-up, in exact rational arithmetic —
/// `source_span × 10ˢᶜᵃˡᵉ / units`, integers throughout. `None` where `speed` is not
/// strictly positive (ADR-0020 makes that a schema error, which is another check's
/// finding, not this function's guess) or where the arithmetic would not fit.
pub fn played_ms(source_span: i64, speed: Decimal) -> Option<i64> {
    if !speed.is_positive() {
        return None;
    }
    let (units, decimal_scale) = speed.as_ratio()?;
    let numerator = i128::from(source_span).checked_mul(decimal_scale)?;
    i64::try_from(round_half_up(numerator, units)?).ok()
}

/// **The `speed` that would satisfy the invariant**, as a decimal an author can write.
///
/// ADR-0020 designates `speed` as the free variable — *"`start`/`end` and
/// `source_start`/`source_end` are authoritative and must never move silently"* — and
/// requires `validate` to *print* the corrective value rather than only flag the
/// mismatch.
///
/// The exact quotient `source_span / timeline_span` is usually not a finite decimal, so
/// what is stated is the shortest decimal that **is verified to satisfy the invariant**,
/// widening a place at a time. A stated repair that does not in fact repair is worse than
/// none, and the check that already evaluates the invariant is the one thing that can tell.
pub fn corrective_speed(source_span: i64, timeline_span: i64) -> Option<Decimal> {
    if source_span <= 0 || timeline_span <= 0 {
        return None;
    }
    // The satisfying set is the interval `(span/(D+½), span/(D−½)]`, whose width is
    // `span/(D²−¼)` and which contains the exact quotient strictly, so some finite decimal
    // inside it always exists. The search is bounded rather than trusted: eighteen places
    // covers any timeline the format's integer milliseconds can address, and the caller
    // states the exact ratio rather than a guess if it ever does not.
    // From zero, so an exact integer repair prints as `2` and not `2.0`.
    (0..=18).find_map(|scale| {
        let unit = pow10(scale)?;
        let units = round_half_up(
            i128::from(source_span).checked_mul(unit)?,
            i128::from(timeline_span),
        )?;
        let candidate = Decimal { units, scale };
        (played_ms(source_span, candidate)? == timeline_span).then_some(candidate)
    })
}

/// **ADR-0035's grid**, asked the one question a check needs of it: does any sampled frame
/// fall inside `[from, to)`?
///
/// Frame *N* samples at `N × 1000/fps` ms, which is not an integer in general. The test is
/// kept on integers rather than on that instant: the first frame at or after `from` is
/// `N = ceil(from × fps / 1000)`, and it lands inside the half-open range exactly when
/// `N × 1000 < to × fps`.
///
/// A range no frame falls in is a range the render never shows — an element that does not
/// appear, or a gap whose black frames are not there.
///
/// A negative `from` is clamped to zero, which the published formula does not state
/// because the format's times are not negative: there is no frame *−1* to be the first one
/// at or after it, and `ceil` on a negative would name one.
pub fn holds_a_sampled_frame(from: i64, to: i64, fps: i64) -> Option<bool> {
    if fps <= 0 || to <= from {
        return None;
    }
    let first = ceil_div(i128::from(from.max(0)).checked_mul(i128::from(fps))?, 1000);
    Some(first.checked_mul(1000)? < i128::from(to).checked_mul(i128::from(fps))?)
}

fn ceil_div(numerator: i128, denominator: i128) -> i128 {
    -floor_div(-numerator, denominator)
}

/// **ADR-0013/ADR-0015's `cover`/`contain` rule**: `cover` or `contain`. `literal` names
/// no rule and is not a member — a caller holding one has nothing to ask this function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitRule {
    /// Drawn rect ⊇ the box.
    Cover,
    /// Drawn rect ⊆ the box.
    Contain,
}

/// Which axis took the box dimension verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrivingAxis {
    Width,
    Height,
}

/// A fitted extent, and which axis drove it. **No verdict, no diff** — ADR-0024 draws
/// that line at `validate` alone; this is the bare derivation, the same one `measure`
/// (#205) will hand an author before they write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FittedExtent {
    pub width: i64,
    pub height: i64,
    pub driving_axis: DrivingAxis,
}

/// **ADR-0013's fitted-extent arithmetic, extended to `contain` by ADR-0015**: a source
/// `sw x sh` fit into a box `bw x bh`, in exact integer arithmetic throughout.
///
/// The driving axis is chosen by integer cross-multiplication — width drives `cover`
/// when `bw*sh >= bh*sw`, and `contain` when `bw*sh <= bh*sw` — **never** by comparing
/// `bw/sw` against `bh/sh` as floats. ADR-0013 measured the float method disagreeing
/// with this one on 4.466% of 31,402,800 sampled (source, box) pairs; its worked
/// divergence is `sw=103, bw=1920`, where `bw as f64 / sw as f64 * sw as f64` computes
/// `1919.9999999999998` and floors to **1919**, a pixel short of the box on the axis
/// that is exact by construction — see `the_driving_axis_never_reaches_a_float` below.
///
/// The driving axis takes the box dimension **verbatim**. The slack axis is
/// `(s_slack * b_driving) // s_driving`, floor, in integer division — the same floor for
/// `contain` as for `cover` (ADR-0015's court found ceil equally safe for `contain`, but
/// floor still wins on additivity and implementation entropy, so there is one rounding
/// operation across the whole vocabulary rather than a per-value table).
///
/// A legal `0` on the slack axis — `contain` of a 300x7 source into a 10x10 box gives
/// 10x**0** — is returned exactly as computed, **never clamped** (ADR-0015: a clamp would
/// fabricate an integer the published rule did not produce).
///
/// `None` where a source or aperture dimension is not positive — nothing this rule can
/// divide by, or draw a box through, and another check's fact to report, not this
/// function's to guess. Nothing in the published schema bounds `clip`'s width/height
/// below, so a malformed document is exactly the input this guards.
pub fn fitted_extent(
    rule: FitRule,
    source: (i64, i64),
    aperture: (i64, i64),
) -> Option<FittedExtent> {
    let (source_width, source_height) = source;
    let (box_width, box_height) = aperture;
    if source_width <= 0 || source_height <= 0 || box_width <= 0 || box_height <= 0 {
        return None;
    }

    let cross_box_width = i128::from(box_width).checked_mul(i128::from(source_height))?;
    let cross_box_height = i128::from(box_height).checked_mul(i128::from(source_width))?;
    let width_drives = match rule {
        FitRule::Cover => cross_box_width >= cross_box_height,
        FitRule::Contain => cross_box_width <= cross_box_height,
    };

    // The driving axis takes its own (source, box) pair verbatim; the slack axis takes
    // the source half of the *other* pair and floors against the driving box dimension.
    // Picking `(driving_source, driving_box, slack_source)` once, by axis, is what keeps
    // `floor_div` written in exactly one place rather than once per branch with its
    // operands swapped by hand.
    let (driving_axis, driving_source, driving_box, slack_source) = if width_drives {
        (DrivingAxis::Width, source_width, box_width, source_height)
    } else {
        (DrivingAxis::Height, source_height, box_height, source_width)
    };
    let slack = i64::try_from(floor_div(
        i128::from(slack_source).checked_mul(i128::from(driving_box))?,
        i128::from(driving_source),
    ))
    .ok()?;

    Some(match driving_axis {
        DrivingAxis::Width => FittedExtent {
            width: driving_box,
            height: slack,
            driving_axis,
        },
        DrivingAxis::Height => FittedExtent {
            width: slack,
            height: driving_box,
            driving_axis,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_literal_becomes_the_rational_it_already_is() {
        // ADR-0045: "`speed`'s authored literal (`0.645`) is already a finite decimal —
        // already an exact rational (`645/1000`). No information is lost at the
        // file-format boundary."
        assert_eq!(
            Decimal::parse("0.645"),
            Some(Decimal {
                units: 645,
                scale: 3
            })
        );
        assert_eq!(Decimal::parse("1"), Some(Decimal { units: 1, scale: 0 }));
        assert_eq!(
            Decimal::parse("-2.50"),
            Some(Decimal {
                units: -250,
                scale: 2
            })
        );
        // The exponent form `ryu` reaches for on small magnitudes.
        assert_eq!(Decimal::parse("1e-3"), Some(Decimal { units: 1, scale: 3 }));
        assert_eq!(
            Decimal::parse("2e2"),
            Some(Decimal {
                units: 200,
                scale: 0
            })
        );
        assert_eq!(Decimal::parse("abc"), None);
        assert_eq!(Decimal::parse(""), None);
    }

    #[test]
    fn the_tie_f64_lands_on_the_wrong_side_of() {
        // ADR-0045's minimal case, verbatim: "`7 / 0.560` is exactly `12.5` in decimal, so
        // round-half-up gives `13`, but `f64` computes `12.499999999999998`, giving `12`."
        let speed = Decimal::parse("0.560").unwrap();
        assert_eq!(played_ms(7, speed), Some(13));
        // And the divergence it is a guard against, spelled out: the naive evaluation.
        let naive = (7.0f64 / 0.560f64 + 0.5).floor() as i64;
        assert_eq!(naive, 12, "the f64 route is still wrong; that is the point");
    }

    #[test]
    fn the_fixtures_own_speed_is_unchanged_by_exact_evaluation() {
        // ADR-0045: "All four `speed` elements in the committed fixture ... share
        // `speed: 0.645` ... None sits on a round-half-up tie."
        let speed = Decimal::parse("0.645").unwrap();
        assert_eq!(played_ms(2184, speed), Some(3386));
        assert_eq!(played_ms(2568, speed), Some(3981));
        assert_eq!(played_ms(1992, speed), Some(3088));
    }

    #[test]
    fn a_speed_of_one_is_the_identity_adr_0005_stated() {
        assert_eq!(played_ms(2568, Decimal::parse("1").unwrap()), Some(2568));
    }

    #[test]
    fn a_speed_that_is_not_strictly_positive_has_no_answer_here() {
        // ADR-0020 makes `0` and negatives schema errors. This function reports that it
        // cannot answer rather than inventing one.
        assert_eq!(played_ms(2568, Decimal::parse("0").unwrap()), None);
        assert_eq!(played_ms(2568, Decimal::parse("-0.5").unwrap()), None);
    }

    #[test]
    fn the_corrective_speed_is_verified_to_satisfy_the_invariant() {
        // Every corrective value this states is fed back through the invariant it is
        // supposed to repair, for a span that has no finite-decimal quotient at all.
        for (source_span, timeline_span) in [(2184, 3386), (2568, 3981), (7, 13), (1000, 3000)] {
            let corrective = corrective_speed(source_span, timeline_span).unwrap();
            assert_eq!(
                played_ms(source_span, corrective),
                Some(timeline_span),
                "{source_span} / {corrective} must come back to {timeline_span}"
            );
        }
    }

    #[test]
    fn a_literal_too_wide_to_hold_exactly_is_refused_rather_than_truncated() {
        // A truncated literal is exactly the wrong answer this module exists to avoid, so
        // both widths refuse: too many significant digits, and a scale `10^scale` cannot
        // hold. The second is the one that matters — a `Decimal` that parsed and then
        // overflowed at the division would reach the check as a rate it silently declines
        // to evaluate.
        assert_eq!(Decimal::parse(&format!("0.{}", "1".repeat(31))), None);
        assert_eq!(Decimal::parse("1e-40"), None);

        // Leading zeros are not significant and do not count against the width.
        let tiny = Decimal::parse("0.000000000000000001").expect("one digit, and a scale");
        assert_eq!(tiny.to_string(), "0.000000000000000001");
    }

    #[test]
    fn an_exact_integer_repair_is_written_as_an_integer() {
        // `2568 / 1284` is exactly 2, and a repair that said `2.0` would be telling an
        // author to write a spelling the fixture never uses.
        assert_eq!(corrective_speed(2568, 1284).unwrap().to_string(), "2");
    }

    #[test]
    fn a_decimal_prints_the_way_a_project_would_write_it() {
        assert_eq!(Decimal::parse("0.645").unwrap().to_string(), "0.645");
        assert_eq!(Decimal::parse("1").unwrap().to_string(), "1");
        assert_eq!(Decimal::parse("-0.5").unwrap().to_string(), "-0.5");
        assert_eq!(Decimal { units: 5, scale: 3 }.to_string(), "0.005");
    }

    #[test]
    fn the_grid_step_is_not_necessarily_integral() {
        // ADR-0035: "at `fps=30` the grid step is `100/3` ms and ... multiples of 100ms
        // [are] the only frame-exact instants". A 34 ms window starting at 1 ms holds the
        // frame at 100/3 ≈ 33.3; a 1 ms window between two frames holds none.
        assert_eq!(holds_a_sampled_frame(1, 35, 30), Some(true));
        assert_eq!(holds_a_sampled_frame(34, 35, 30), Some(false));
        // At 25 fps the step is exactly 40 ms, so a 40 ms window always holds one and a
        // window strictly inside one step need not.
        assert_eq!(holds_a_sampled_frame(0, 40, 25), Some(true));
        assert_eq!(holds_a_sampled_frame(41, 79, 25), Some(false));
        assert_eq!(holds_a_sampled_frame(41, 81, 25), Some(true));
    }

    #[test]
    fn an_empty_or_inverted_range_holds_nothing_and_says_so() {
        assert_eq!(holds_a_sampled_frame(40, 40, 25), None);
        assert_eq!(holds_a_sampled_frame(80, 40, 25), None);
        assert_eq!(holds_a_sampled_frame(0, 40, 0), None);
    }

    #[test]
    fn line_height_recovers_its_tenths_from_the_literal_not_from_f64_times_ten() {
        assert_eq!(Decimal::parse("1.1").unwrap().tenths(), Some(11));
        assert_eq!(Decimal::parse("1.2").unwrap().tenths(), Some(12));
        assert_eq!(Decimal::parse("1").unwrap().tenths(), Some(10));
        // A trailing zero is still an exact tenth: `1.10` is `1.1`.
        assert_eq!(Decimal::parse("1.10").unwrap().tenths(), Some(11));
        // Not a multiple of `0.1` — a schema-time error belonging to another check.
        assert_eq!(Decimal::parse("1.15").unwrap().tenths(), None);
    }

    #[test]
    fn adr_0028_s_own_measured_divergence_a_ceil_boundary_f64_lands_on_the_wrong_side_of() {
        // `size = 25`, `line_height = 1.1`, two lines: the exact product is `550/10 =
        // 55.0`, an integer, so the true `ceil` is `55`. The naive "parse `line_height`
        // as `f64`, multiply, `ceil`" route computes `25.0 * 1.1 * 2.0 ==
        // 55.00000000000001` and ceils it to `56` — one pixel short of what the document
        // actually states, exactly the ULP hazard ADR-0028 measured on 6.35% of sampled
        // `(size, line_count)` pairs.
        let naive = (25.0f64 * 1.1f64 * 2.0f64).ceil() as i64;
        assert_eq!(naive, 56, "the f64 route is still wrong; that is the point");
        assert_eq!(
            text_block_height(25, Decimal::parse("1.1").unwrap().tenths().unwrap(), 2),
            Some(55)
        );
    }

    #[test]
    fn a_non_positive_input_has_no_block_height_here() {
        assert_eq!(text_block_height(0, 11, 1), None);
        assert_eq!(text_block_height(55, 0, 1), None);
        assert_eq!(text_block_height(55, 11, 0), None);
    }

    #[test]
    fn the_fixture_s_seven_photos_derive_the_published_1912() {
        // ADR-0013's own worked case: `images/06.png` is 1536x2720, `clip` is
        // 1080x1300, and the published element is `width:1080, height:1912`. Exact
        // cover is 1912.5; the rule floors.
        let extent =
            fitted_extent(FitRule::Cover, (1536, 2720), (1080, 1300)).expect("positive source");
        assert_eq!((extent.width, extent.height), (1080, 1912));
        assert_eq!(extent.driving_axis, DrivingAxis::Width);
    }

    #[test]
    fn the_driving_axis_never_reaches_a_float() {
        // ADR-0013's measured divergence: `sw=103, bw=1920`. The naive route — compute
        // the scale factor in `f64`, multiply back through — loses the driving axis to
        // a ULP.
        let naive = (103.0f64 * (1920.0f64 / 103.0f64)) as i64;
        assert_eq!(
            naive, 1919,
            "the f64 route is a pixel short; that is the bug"
        );

        // The box's height is chosen large enough that width still drives under
        // `cover`: `bw*sh = 1920*900 = 1,728,000 >= bh*sw = 100*103 = 10,300`.
        let extent =
            fitted_extent(FitRule::Cover, (103, 900), (1920, 100)).expect("positive source");
        assert_eq!(
            extent.width, 1920,
            "the driving axis takes the box dimension verbatim, never a rounded float"
        );
    }

    #[test]
    fn contain_can_derive_a_legal_zero_with_no_clamp() {
        // ADR-0015's own example: a 300x7 source into a 10x10 box.
        let extent = fitted_extent(FitRule::Contain, (300, 7), (10, 10)).expect("positive source");
        assert_eq!((extent.width, extent.height), (10, 0));
    }

    #[test]
    fn exact_aspect_match_makes_cover_and_contain_agree() {
        // ADR-0026: `handle-logo`'s 800x800 source into a 68x68 aperture — cover and
        // contain compute the identical rect, so both spellings are simultaneously true.
        let cover = fitted_extent(FitRule::Cover, (800, 800), (68, 68)).expect("positive source");
        let contain =
            fitted_extent(FitRule::Contain, (800, 800), (68, 68)).expect("positive source");
        assert_eq!((cover.width, cover.height), (68, 68));
        assert_eq!((cover.width, cover.height), (contain.width, contain.height));
    }

    #[test]
    fn a_non_positive_source_dimension_has_no_fitted_extent_here() {
        assert_eq!(fitted_extent(FitRule::Cover, (0, 10), (5, 5)), None);
        assert_eq!(fitted_extent(FitRule::Cover, (10, 0), (5, 5)), None);
        assert_eq!(fitted_extent(FitRule::Contain, (-1, 10), (5, 5)), None);
    }

    #[test]
    fn a_non_positive_aperture_dimension_has_no_fitted_extent_here() {
        // Nothing in the published schema bounds `clip`'s width/height below, so a
        // malformed `[x, y, -500, -500]` must not flow through to a `width: -500` taken
        // verbatim — there is no box to draw that rect through.
        assert_eq!(fitted_extent(FitRule::Cover, (100, 100), (0, 5)), None);
        assert_eq!(fitted_extent(FitRule::Cover, (100, 100), (5, 0)), None);
        assert_eq!(
            fitted_extent(FitRule::Contain, (100, 100), (-500, -500)),
            None
        );
    }
}

//! ADR-0223 typography features (#1098): `FontFile.axes`, closed OpenType `features` on
//! `TextElement` and `Run`, `Run.baseline_shift`, and `Run.letter_spacing`.

use std::path::{Path, PathBuf};

use montagent_core::report::{ExitCode, Report};
use montagent_core::validate;
use serde_json::{Value, json};
use skrifa::raw::FileRef;

mod common;
use common::{canonical, write_project};

fn workspace(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .canonicalize()
        .expect("a committed fixture")
}

const INTER: &str = "fixtures/skills/fonts/Inter-Bold.ttf";
const OSWALD: &str = "fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf";
const NASKH: &str = "fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf";

/// Build a valid TrueType variable font by copying every table from `base_ttf` and
/// inserting `fvar` (`wght` in `100..=900`, default `400`) and `HVAR` (`+200` font units
/// of advance width at `wght = 900`), plus shifting a contour point in `gvar` or `HVAR`
/// so HarfBuzz (`rustybuzz`), `skrifa`, and Skia all see a real variable font.
fn build_variable_ttf(base_ttf: &Path) -> Vec<u8> {
    let bytes = std::fs::read(base_ttf).expect("read base ttf");
    let Ok(FileRef::Font(font)) = FileRef::new(&bytes) else {
        panic!("expected single-face ttf");
    };

    // Minimal `fvar` table with 1 axis: `wght` min=100.0, default=400.0, max=900.0.
    let mut fvar = Vec::new();
    fvar.extend_from_slice(&1u16.to_be_bytes()); // majorVersion
    fvar.extend_from_slice(&0u16.to_be_bytes()); // minorVersion
    fvar.extend_from_slice(&16u16.to_be_bytes()); // axesArrayOffset
    fvar.extend_from_slice(&2u16.to_be_bytes()); // reserved
    fvar.extend_from_slice(&1u16.to_be_bytes()); // axisCount = 1
    fvar.extend_from_slice(&20u16.to_be_bytes()); // axisSize = 20
    fvar.extend_from_slice(&0u16.to_be_bytes()); // instanceCount = 0
    fvar.extend_from_slice(&4u16.to_be_bytes()); // instanceSize = 4
    // VariationAxisRecord:
    fvar.extend_from_slice(b"wght");
    fvar.extend_from_slice(&(100i32 << 16).to_be_bytes()); // minValue = 100.0
    fvar.extend_from_slice(&(400i32 << 16).to_be_bytes()); // defaultValue = 400.0
    fvar.extend_from_slice(&(900i32 << 16).to_be_bytes()); // maxValue = 900.0
    fvar.extend_from_slice(&0u16.to_be_bytes()); // flags
    fvar.extend_from_slice(&256u16.to_be_bytes()); // axisNameID

    // Minimal `HVAR` table that adds +300 font units of advance width at `wght = 900`
    // (normalized coordinate +1.0) for every glyph.
    // Header (20 bytes):
    //   major=1, minor=0, ivsOffset=20, advanceWidthMappingOffset=54, lsb=0, rsb=0
    // ItemVariationStore at offset 20 (34 bytes):
    //   format=1 (2B), regionListOffset=12 (4B), dataCount=1 (2B), dataOffsets[0]=22 (4B)
    //   VariationRegionList at offset 20+12=32 (10 bytes):
    //     axisCount=1 (2B), regionCount=1 (2B),
    //     startCoord=0 (2B), peakCoord=0x4000 (2B), endCoord=0x4000 (2B)
    //   ItemVariationData at offset 20+22=42 (12 bytes):
    //     itemCount=1 (2B), wordDeltaCount=1 (2B), regionIndexCount=1 (2B),
    //     regionIndexes[0]=0 (2B), deltaSets[0]=+300 (2B), pad (2B)
    // DeltaSetIndexMap at offset 54 (6 bytes):
    //   format=0 (1B), entryFormat=0 (1B), mapCount=1 (2B), mapData=[0] (2B)
    let mut hvar = Vec::new();
    hvar.extend_from_slice(&1u16.to_be_bytes()); // majorVersion
    hvar.extend_from_slice(&0u16.to_be_bytes()); // minorVersion
    hvar.extend_from_slice(&20u32.to_be_bytes()); // itemVariationStoreOffset
    hvar.extend_from_slice(&54u32.to_be_bytes()); // advanceWidthMappingOffset
    hvar.extend_from_slice(&0u32.to_be_bytes()); // lsbMappingOffset
    hvar.extend_from_slice(&0u32.to_be_bytes()); // rsbMappingOffset
    // IVS at 20:
    hvar.extend_from_slice(&1u16.to_be_bytes()); // format = 1
    hvar.extend_from_slice(&12u32.to_be_bytes()); // variationRegionListOffset = 12
    hvar.extend_from_slice(&1u16.to_be_bytes()); // itemVariationDataCount = 1
    hvar.extend_from_slice(&22u32.to_be_bytes()); // itemVariationDataOffsets[0] = 22
    // VariationRegionList at 32:
    hvar.extend_from_slice(&1u16.to_be_bytes()); // axisCount = 1
    hvar.extend_from_slice(&1u16.to_be_bytes()); // regionCount = 1
    hvar.extend_from_slice(&0i16.to_be_bytes()); // startCoord = 0.0
    hvar.extend_from_slice(&0x4000i16.to_be_bytes()); // peakCoord = 1.0
    hvar.extend_from_slice(&0x4000i16.to_be_bytes()); // endCoord = 1.0
    // ItemVariationData at 42:
    hvar.extend_from_slice(&1u16.to_be_bytes()); // itemCount = 1
    hvar.extend_from_slice(&1u16.to_be_bytes()); // wordDeltaCount = 1
    hvar.extend_from_slice(&1u16.to_be_bytes()); // regionIndexCount = 1
    hvar.extend_from_slice(&0u16.to_be_bytes()); // regionIndexes[0] = 0
    hvar.extend_from_slice(&300i16.to_be_bytes()); // delta = +300 font units
    hvar.extend_from_slice(&0u16.to_be_bytes()); // pad
    // DeltaSetIndexMap at 54:
    hvar.push(0u8); // format = 0
    hvar.push(0u8); // entryFormat = 0 (1 byte per entry, outer=0, inner=0)
    hvar.extend_from_slice(&1u16.to_be_bytes()); // mapCount = 1
    hvar.push(0u8); // entry 0 -> (0, 0)
    hvar.push(0u8); // pad

    let mut tables: Vec<([u8; 4], Vec<u8>)> = font
        .table_directory()
        .table_records()
        .iter()
        .map(|rec| {
            let tag = rec.tag().to_be_bytes();
            let data = font
                .table_data(rec.tag())
                .expect("table data")
                .as_bytes()
                .to_vec();
            (tag, data)
        })
        .collect();
    tables.push((*b"fvar", fvar));
    tables.push((*b"HVAR", hvar));
    tables.sort_by_key(|(tag, _)| *tag);

    let count = tables.len();
    let entry_selector = count.max(1).ilog2();
    let search_range = (1u32 << entry_selector) * 16;
    let range_shift = (count as u32 * 16).wrapping_sub(search_range);

    let mut out = Vec::new();
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&(count as u16).to_be_bytes());
    out.extend_from_slice(&(search_range as u16).to_be_bytes());
    out.extend_from_slice(&(entry_selector as u16).to_be_bytes());
    out.extend_from_slice(&(range_shift as u16).to_be_bytes());

    let dir_end = out.len() + count * 16;
    out.resize(dir_end, 0);

    for (i, (tag, data)) in tables.iter().enumerate() {
        let offset = out.len() as u32;
        let length = data.len() as u32;
        out.extend_from_slice(data);
        while !out.len().is_multiple_of(4) {
            out.push(0);
        }
        let mut checksum = 0u32;
        for chunk in out[offset as usize..].chunks(4) {
            let mut word = [0u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            checksum = checksum.wrapping_add(u32::from_be_bytes(word));
        }
        let slot = 12 + i * 16;
        out[slot..slot + 4].copy_from_slice(tag);
        out[slot + 4..slot + 8].copy_from_slice(&checksum.to_be_bytes());
        out[slot + 8..slot + 12].copy_from_slice(&offset.to_be_bytes());
        out[slot + 12..slot + 16].copy_from_slice(&length.to_be_bytes());
    }
    out
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn project_with_fonts(fonts: Value, font_vendor: Value, elements: &[Value], line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    let document = json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "background": "#000000",
        "fonts": fonts,
        "fontVendor": font_vendor,
        "tracks": [{"name": "titles", "layer": 1, "elements": elements}],
    });
    write_project(&dir, "p.montagent.json", &canonical(&document.to_string()))
}

fn inter_project(elements: &[Value], line: u32) -> PathBuf {
    let inter_path = workspace(INTER).display().to_string();
    let oswald_path = workspace(OSWALD).display().to_string();
    let naskh_path = workspace(NASKH).display().to_string();
    let inter_sha = montagent_core::fonts::sha256_hex(&std::fs::read(workspace(INTER)).unwrap());
    project_with_fonts(
        json!({
            "inter": [{"file": inter_path}],
            "oswald": [{"file": oswald_path}],
            "arabic": [{"file": oswald_path}, {"file": naskh_path}],
        }),
        json!({
            inter_path: {
                "licence": "OFL-1.1",
                "source": "rsms/inter Inter-Bold.ttf",
                "sha256": inter_sha,
            },
            oswald_path: {
                "licence": "OFL-1.1",
                "source": "google/fonts ofl/oswald, instanced wght=600",
                "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9",
            },
            naskh_path: {
                "licence": "OFL-1.1",
                "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21",
            },
        }),
        elements,
        line,
    )
}

fn text_el(id: &str, font: &str, runs: Value) -> Value {
    json!({
        "id": id,
        "type": "text",
        "start": 0,
        "end": 2000,
        "x": 540,
        "y": 400,
        "width": 1000,
        "height": 200,
        "font": font,
        "size": 100,
        "align": "center",
        "color": "#FFFFFF",
        "runs": runs,
        "caption": false,
    })
}

// ---------------------------------------------------------------------------
// 1. Variable font axes (`FontFile.axes`, `E-FONT-AXIS`).
// ---------------------------------------------------------------------------

#[test]
fn variable_font_axes_shape_distinct_advances_and_render_distinct_frames_from_one_file() {
    let dir = common::tempdir(line!());
    let var_bytes = build_variable_ttf(&workspace(INTER));
    let var_sha = montagent_core::fonts::sha256_hex(&var_bytes);
    let var_path = dir.join("InterVar.ttf");
    std::fs::write(&var_path, &var_bytes).unwrap();
    let var_str = var_path.display().to_string();

    let document = json!({
        "frame": {"width": 600, "height": 300},
        "fps": 25,
        "background": "#000000",
        "fonts": {
            "regular": [{"file": var_str, "axes": {"wght": 400}}],
            "heavy": [{"file": var_str, "axes": {"wght": 900}}],
        },
        "fontVendor": {
            var_str: {
                "licence": "OFL-1.1",
                "source": "synthetic variable Inter",
                "sha256": var_sha,
            },
        },
        "tracks": [{"name": "titles", "layer": 1, "elements": [
            {
                "id": "both",
                "type": "text",
                "start": 0,
                "end": 1000,
                "x": 300,
                "y": 150,
                "width": 560,
                "height": 200,
                "font": "regular",
                "size": 60,
                "color": "#FFFFFF",
                "runs": [
                    {"text": "ABCD\n", "font": "regular"},
                    {"text": "ABCD", "font": "heavy"},
                ],
                "caption": false,
            }
        ]}],
    });
    let path = write_project(&dir, "p.montagent.json", &canonical(&document.to_string()));
    let report = validate(&path);
    assert!(
        !codes(&report).iter().any(|c| c.starts_with("E-")),
        "{:?}",
        report.findings
    );

    // Measure both lines: line 2 (`heavy`, wght=900) must have a wider advance than line 1 (`regular`, wght=400).
    let el = document["tracks"][0]["elements"][0].clone();
    let measured = montagent_core::verbs::measure::measure(
        &path,
        &montagent_core::verbs::measure::Ask {
            element: Some(el),
            ..Default::default()
        },
    )
    .to_json();
    let line0_adv = measured["measure"]["lines"][0]["advance_width"]
        .as_f64()
        .unwrap();
    let line1_adv = measured["measure"]["lines"][1]["advance_width"]
        .as_f64()
        .unwrap();
    assert!(
        line1_adv > line0_adv + 20.0,
        "expected heavy axis (wght=900) to widen advance over regular (wght=400): {line0_adv} vs {line1_adv}"
    );
}

#[test]
fn font_axis_check_rejects_missing_and_out_of_range_axes_and_schema_rejects_empty_axes() {
    let dir = common::tempdir(line!());
    let var_bytes = build_variable_ttf(&workspace(INTER));
    let var_sha = montagent_core::fonts::sha256_hex(&var_bytes);
    let var_path = dir.join("InterVar.ttf");
    std::fs::write(&var_path, &var_bytes).unwrap();
    let var_str = var_path.display().to_string();

    // 1. Out-of-range `wght: 950` (range is 100..=900) and unsupported `opsz: 14` -> two `E-FONT-AXIS` findings.
    let doc = json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "fonts": {
            "bad": [{"file": var_str, "axes": {"wght": 950, "opsz": 14}}],
        },
        "fontVendor": {
            var_str.clone(): {
                "licence": "OFL-1.1",
                "source": "synthetic variable Inter",
                "sha256": var_sha,
            },
        },
        "tracks": [],
    });
    let path = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let report = validate(&path);
    let axis_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-FONT-AXIS")
        .collect();
    assert_eq!(axis_findings.len(), 2, "{:?}", report.findings);

    // 2. Empty `axes: {}` is rejected by the schema (`E-SCHEMA`).
    let empty_axes = json!({
        "frame": {"width": 1080, "height": 1920},
        "fps": 25,
        "fonts": {
            "empty": [{"file": var_str, "axes": {}}],
        },
        "tracks": [],
    });
    let empty_path = write_project(
        &dir,
        "empty.montagent.json",
        &canonical(&empty_axes.to_string()),
    );
    let empty_report = validate(&empty_path);
    assert!(
        codes(&empty_report).contains(&"E-SCHEMA"),
        "{:?}",
        codes(&empty_report)
    );
}

// ---------------------------------------------------------------------------
// 2. OpenType `features` (`TextElement.features`, `Run.features`, `E-FONT-FEATURE`).
// ---------------------------------------------------------------------------

#[test]
fn tnum_equalizes_digit_advances_and_run_features_override_or_clear_element_features() {
    use montagent_core::verbs::measure::{Ask, measure};
    let path = inter_project(&[], line!());

    let measure_text = |el: Value| -> f64 {
        let ans = measure(
            &path,
            &Ask {
                element: Some(el),
                ..Ask::default()
            },
        )
        .to_json();
        ans["measure"]["advance_width"].as_f64().unwrap()
    };

    // Without `tnum`, "1111" in Inter-Bold is narrower than "8888".
    let ones_proportional = measure_text(text_el("a", "inter", json!([{"text": "1111"}])));
    let eights_proportional = measure_text(text_el("b", "inter", json!([{"text": "8888"}])));
    assert!(
        (eights_proportional - ones_proportional).abs() > 10.0,
        "expected proportional 1111 ({ones_proportional}) to differ from 8888 ({eights_proportional})"
    );

    // With element-level `features: ["tnum"]`, "1111" and "8888" have identical advance width.
    let mut ones_tnum = text_el("a", "inter", json!([{"text": "1111"}]));
    ones_tnum["features"] = json!(["tnum"]);
    let mut eights_tnum = text_el("b", "inter", json!([{"text": "8888"}]));
    eights_tnum["features"] = json!(["tnum"]);
    assert!(
        (measure_text(ones_tnum) - measure_text(eights_tnum)).abs() < 0.01,
        "tnum must equalize digit advances"
    );

    // `Run.features: []` clears element-level `features: ["tnum"]` back to proportional digits.
    let mut cleared = text_el("a", "inter", json!([{"text": "1111", "features": []}]));
    cleared["features"] = json!(["tnum"]);
    assert!(
        (measure_text(cleared) - ones_proportional).abs() < 0.01,
        "Run.features: [] must clear element features"
    );
}

#[test]
fn schema_rejects_mutually_exclusive_duplicate_and_unknown_feature_tags() {
    for bad in [
        json!(["tnum", "onum"]),
        json!(["sups", "subs"]),
        json!(["tnum", "tnum"]),
        json!(["liga"]),
    ] {
        let mut on_el = text_el("a", "inter", json!([{"text": "123"}]));
        on_el["features"] = bad.clone();
        let report = validate(&inter_project(&[on_el], line!()));
        assert!(
            codes(&report).contains(&"E-SCHEMA"),
            "element features {bad} should fail E-SCHEMA: {:?}",
            codes(&report)
        );

        let on_run = text_el("a", "inter", json!([{"text": "123", "features": bad}]));
        let report = validate(&inter_project(&[on_run], line!()));
        assert!(
            codes(&report).contains(&"E-SCHEMA"),
            "run features {bad} should fail E-SCHEMA: {:?}",
            codes(&report)
        );
    }

    // Empty `features: []` is rejected on `TextElement`, whereas `Run.features: []` is allowed (resets element features).
    let mut empty_on_el = text_el("a", "inter", json!([{"text": "123"}]));
    empty_on_el["features"] = json!([]);
    let report = validate(&inter_project(&[empty_on_el], line!()));
    assert!(
        codes(&report).contains(&"E-SCHEMA"),
        "empty element features [] should fail E-SCHEMA: {:?}",
        codes(&report)
    );
}

#[test]
fn e_font_feature_fires_when_a_shaping_face_lacks_a_requested_feature() {
    // Inter-Bold has `tnum` and `zero`, but lacks `smcp`.
    let mut supported = text_el("ok", "inter", json!([{"text": "0123"}]));
    supported["features"] = json!(["tnum", "zero"]);

    let mut unsupported = text_el("bad", "inter", json!([{"text": "Hello"}]));
    unsupported["features"] = json!(["smcp"]);

    let report = validate(&inter_project(&[supported, unsupported], line!()));
    let found: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-FONT-FEATURE")
        .collect();
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].location.element.as_deref(), Some("bad"));
    assert_eq!(found[0].fields["feature"], "smcp");
}

// ---------------------------------------------------------------------------
// 3. `Run.baseline_shift`.
// ---------------------------------------------------------------------------

#[test]
fn baseline_shift_zero_is_rejected_by_schema() {
    let bad = text_el("a", "inter", json!([{"text": "H", "baseline_shift": 0}]));
    let report = validate(&inter_project(&[bad], line!()));
    assert!(codes(&report).contains(&"E-SCHEMA"), "{:?}", codes(&report));
}

#[test]
fn baseline_shift_moves_glyph_ink_without_changing_slot_height_and_feeds_collision_and_overflow() {
    use montagent_core::verbs::measure::{Ask, measure};
    use montagent_core::verbs::query::{Ask as QAsk, query};

    let base = text_el("base", "inter", json!([{"text": "H"}]));
    let shifted_up = text_el("up", "inter", json!([{"text": "H", "baseline_shift": 25}]));
    let path = inter_project(&[base.clone(), shifted_up.clone()], line!());

    let m_base = measure(
        &path,
        &Ask {
            element: Some(base),
            ..Ask::default()
        },
    )
    .to_json();
    let m_up = measure(
        &path,
        &Ask {
            element: Some(shifted_up),
            ..Ask::default()
        },
    )
    .to_json();

    // Vertical slot height (`block_height`) is unchanged by `baseline_shift`.
    assert_eq!(
        m_base["measure"]["block_height"],
        m_up["measure"]["block_height"]
    );
    // Ink top moves upward by 25 px (`y - baseline_shift`).
    let ink_top_base = m_base["measure"]["lines"][0]["ink_top"].as_f64().unwrap();
    let ink_top_up = m_up["measure"]["lines"][0]["ink_top"].as_f64().unwrap();
    assert!(
        (ink_top_base - ink_top_up - 25.0).abs() < 0.05,
        "ink_top should move up by 25px: {ink_top_base} vs {ink_top_up}"
    );

    // `query --at` `ink_box.y` also moves up by 25 px.
    let q = query(
        &path,
        &QAsk {
            at: Some(500),
            ..QAsk::default()
        },
    )
    .to_json()["query"]
        .clone();
    let stack = q["stack"].as_array().unwrap();
    let y_base = stack.iter().find(|e| e["id"] == "base").unwrap()["ink_box"]["y"]
        .as_f64()
        .unwrap();
    let y_up = stack.iter().find(|e| e["id"] == "up").unwrap()["ink_box"]["y"]
        .as_f64()
        .unwrap();
    assert!(
        (y_base - y_up - 25.0).abs() < 0.05,
        "query ink_box.y should move up by 25px: {y_base} vs {y_up}"
    );

    // A large upward `baseline_shift` on line 2 causes `R-LINE-INK-COLLISION` with line 1.
    let colliding = text_el(
        "collide",
        "inter",
        json!([
            {"text": "TOP\n"},
            {"text": "BOT", "baseline_shift": 90},
        ]),
    );
    let report = validate(&inter_project(&[colliding], line!()));
    assert!(
        codes(&report).contains(&"R-LINE-INK-COLLISION"),
        "{:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------
// 4. `Run.letter_spacing` and `R-SPACING-SUPPRESSED`.
// ---------------------------------------------------------------------------

#[test]
fn run_letter_spacing_overrides_element_spacing_and_controls_arabic_suppression_per_run() {
    use montagent_core::verbs::measure::{Ask, measure};
    let path = inter_project(&[], line!());

    // Element has `letter_spacing: 200`, but Run 1 overrides `letter_spacing: 0`.
    let mut reset_run = text_el(
        "reset",
        "oswald",
        json!([{"text": "TRACKED", "letter_spacing": 0}]),
    );
    reset_run["letter_spacing"] = json!(200);
    let plain = text_el("plain", "oswald", json!([{"text": "TRACKED"}]));

    let adv_reset = measure(
        &path,
        &Ask {
            element: Some(reset_run),
            ..Ask::default()
        },
    )
    .to_json()["measure"]["advance_width"]
        .as_f64()
        .unwrap();
    let adv_plain = measure(
        &path,
        &Ask {
            element: Some(plain),
            ..Ask::default()
        },
    )
    .to_json()["measure"]["advance_width"]
        .as_f64()
        .unwrap();
    assert!(
        (adv_reset - adv_plain).abs() < 0.01,
        "Run.letter_spacing: 0 must override element letter_spacing: 200 back to unspaced advance"
    );

    // Arabic run with `Run.letter_spacing: 150` triggers `R-SPACING-SUPPRESSED`.
    let arabic_spaced = text_el(
        "ar_spaced",
        "arabic",
        json!([{"text": "سلام", "letter_spacing": 150}]),
    );
    let report = validate(&inter_project(&[arabic_spaced], line!()));
    assert!(
        codes(&report).contains(&"R-SPACING-SUPPRESSED"),
        "{:?}",
        codes(&report)
    );

    // Element has `letter_spacing: 150`, Latin run inherits it, Arabic run resets `letter_spacing: 0` -> no `R-SPACING-SUPPRESSED`.
    let mut arabic_reset = text_el(
        "ar_reset",
        "arabic",
        json!([
            {"text": "HELLO "},
            {"text": "سلام", "letter_spacing": 0},
        ]),
    );
    arabic_reset["letter_spacing"] = json!(150);
    let report = validate(&inter_project(&[arabic_reset], line!()));
    assert!(
        !codes(&report).contains(&"R-SPACING-SUPPRESSED"),
        "{:?}",
        codes(&report)
    );
}

// ---------------------------------------------------------------------------
// 5. 1-painter vs K-painters byte-identity across all four ADR-0223 features.
// ---------------------------------------------------------------------------

#[test]
fn adr_0223_typography_paints_identical_frames_on_k_painters() {
    use montagent_core::verbs::render::{
        Ask, Forced, Progress, force_painting, render, tap_frames,
    };
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let var_bytes = build_variable_ttf(&workspace(INTER));
    let var_sha = montagent_core::fonts::sha256_hex(&var_bytes);
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::write(dir.join("fonts/InterVar.ttf"), &var_bytes).unwrap();

    let document = json!({
        "frame": {"width": 320, "height": 180},
        "fps": 25,
        "background": "#101418",
        "duration": 600,
        "output": "out/typography.mp4",
        "fonts": {
            "light": [{"file": "fonts/InterVar.ttf", "axes": {"wght": 300}}],
            "heavy": [{"file": "fonts/InterVar.ttf", "axes": {"wght": 850}}],
        },
        "fontVendor": {
            "fonts/InterVar.ttf": {
                "licence": "OFL-1.1",
                "source": "synthetic variable Inter",
                "sha256": var_sha,
            },
        },
        "tracks": [{"name": "titles", "layer": 1, "elements": [{
            "id": "title",
            "type": "text",
            "start": 0,
            "end": 600,
            "x": 160,
            "y": 90,
            "width": 300,
            "height": 100,
            "font": "light",
            "size": 32,
            "color": "#FFFFFF",
            "align": "center",
            "features": ["tnum"],
            "runs": [
                {"text": "1111 ", "font": "light", "letter_spacing": 100},
                {"text": "2nd", "font": "heavy", "size": 20, "baseline_shift": 8, "features": ["ordn"], "letter_spacing": 0},
            ],
            "caption": false,
        }]}],
    });
    let path = write_project(
        &dir,
        "typography.montagent.json",
        &canonical(&document.to_string()),
    );
    let rendered = |forced: Forced| {
        let path = path.clone();
        std::thread::spawn(move || {
            let _forced = force_painting(forced);
            let tap = tap_frames();
            let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
            assert_eq!(
                answer.report().exit_code(),
                ExitCode::Ok,
                "{}",
                answer.to_json()
            );
            tap.hashes()
        })
        .join()
        .expect("render thread")
    };
    let one = rendered(Forced::OnePainter);
    let many = rendered(Forced::Chunks {
        painters: 3,
        chunk: 2,
        window_bytes: None,
    });
    assert_eq!(one, many);
}

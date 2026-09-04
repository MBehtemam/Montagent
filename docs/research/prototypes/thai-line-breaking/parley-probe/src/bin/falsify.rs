// Try to break the hard-break finding. Every plausible way the harness could be
// wrong or the API misused, run against the same Thai string.
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, LayoutContext, OverflowWrap,
    StyleProperty, TextWrapMode, WordBreak,
};

const THAI: &str = "ฉันกำลังถือแมงมุม\nตัวใหญ่ไว้ในมือ";
const LATIN: &str = "AAA\nBBB";

fn run(label: &str, text: &str, family: &str, width: Option<f32>, f: &dyn Fn(&mut parley::RangedBuilder<'_, [u8; 4]>)) {
    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();
    let mut b = lcx.ranged_builder(&mut fcx, text, 1.0, false);
    b.push_default(StyleProperty::FontSize(57.0));
    b.push_default(StyleProperty::FontFamily(FontFamily::named(family)));
    f(&mut b);
    let mut l = b.build(text);
    l.break_all_lines(width);
    l.align(Alignment::Start, AlignmentOptions::default());
    let expect = text.matches('\n').count() + 1;
    let n = l.lines().count();
    let reasons: Vec<String> = l.lines().map(|x| format!("{:?}", x.break_reason())).collect();
    let segs: Vec<String> = l.lines().map(|x| {
        let r = x.text_range();
        format!("{:?}", text[r.start..r.end].trim_end_matches('\n'))
    }).collect();
    println!("{label:<44} {n}/{expect} lines  reasons {}  {}",
             reasons.join(","), if n == expect { "ok" } else { "**DROPPED**" });
    if std::env::var("SEGS").is_ok() { println!("{:>46} {}", "", segs.join("  |  ")); }
}

fn main() {
    println!("# control: the same harness on Latin must pass every row");
    run("latin, break_all_lines(None)", LATIN, "Ayuthaya", None, &|_| {});
    println!("\n# is `None` the problem? try real widths");
    for w in [None, Some(f32::MAX), Some(10_000.0), Some(984.0), Some(400.0), Some(1.0)] {
        run(&format!("thai, width {w:?}"), THAI, "Ayuthaya", w, &|_| {});
    }
    println!("\n# do the wrap-policy styles change it?");
    run("thai, TextWrapMode::NoWrap", THAI, "Ayuthaya", Some(984.0),
        &|b| b.push_default(StyleProperty::TextWrapMode(TextWrapMode::NoWrap)));
    run("thai, TextWrapMode::Wrap", THAI, "Ayuthaya", Some(984.0),
        &|b| b.push_default(StyleProperty::TextWrapMode(TextWrapMode::Wrap)));
    run("thai, OverflowWrap::Anywhere", THAI, "Ayuthaya", Some(984.0),
        &|b| b.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere)));
    run("thai, WordBreak::BreakAll", THAI, "Ayuthaya", Some(984.0),
        &|b| b.push_default(StyleProperty::WordBreak(WordBreak::BreakAll)));
    run("thai, WordBreak::KeepAll", THAI, "Ayuthaya", Some(984.0),
        &|b| b.push_default(StyleProperty::WordBreak(WordBreak::KeepAll)));
    println!("\n# is it the font? same Thai text through fonts that are not Thai-specific");
    for f in ["Ayuthaya", "Thonburi", "Helvetica", "Arial Unicode MS"] {
        run(&format!("thai text, font {f}"), THAI, f, Some(984.0), &|_| {});
    }
    println!("\n# does splitting on \\n and laying out paragraphs separately work?");
    let parts: Vec<&str> = THAI.split('\n').collect();
    println!("thai, caller splits into {} paragraphs -> that is the workaround, not a fix", parts.len());
}

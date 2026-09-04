// The jury's falsification list, run. Their sharpest objection: both parley
// columns are identical INCLUDING with the dictionary off, which is the
// signature of a harness that built the same configuration twice.
use parley::{FontContext, FontFamily, LayoutContext, StyleProperty};

fn lines(text: &str, family: &str, width: Option<f32>) -> usize {
    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();
    let mut b = lcx.ranged_builder(&mut fcx, text, 1.0, false);
    b.push_default(StyleProperty::FontSize(57.0));
    b.push_default(StyleProperty::FontFamily(FontFamily::named(family)));
    let mut l = b.build(text);
    l.break_all_lines(width);
    l.lines().count()
}

fn main() {
    println!("# 1. FEATURE-ACTIVATION PROBE (jury C's first objection)");
    println!("#    A Thai string with dictionary-only opportunities in a finite box.");
    println!("#    If the feature is live, this wraps; if not, it cannot.");
    let thai_nowrap = "การเดินทางไปประเทศไทยเป็นประสบการณ์ที่ดีมากสำหรับนักท่องเที่ยวทุกคน";
    let n = lines(thai_nowrap, "Ayuthaya", Some(420.0));
    println!("   unspaced Thai in a 420px box -> {n} line(s)   complex-scripts={}",
             cfg!(feature = "complex-scripts"));
    println!("   (1 line = feature OFF and overflowing; >1 = feature ON and segmenting)\n");

    println!("# 2. BYTES ON THE WIRE (jury B's #2, jury C's #3)");
    for (id, s) in [("thai", "ฉันกำลังถือแมงมุม\nตัวใหญ่ไว้ในมือ"), ("latin", "AAA\nBBB")] {
        let nl: Vec<usize> = s.bytes().enumerate().filter(|(_, b)| *b == 0x0A).map(|(i, _)| i).collect();
        let u2028 = s.contains('\u{2028}');
        println!("   {id:<6} 0x0A at {nl:?}   contains U+2028: {u2028}   len {} bytes", s.len());
    }
    println!();

    println!("# 3. MINIMAL-PAIR ISOLATION OF THE TRIGGER (jury C's #4)");
    println!("#    Is it the character immediately before the LF, or any SA in the run?");
    for (id, s) in [
        ("Thai...Latin then LF   \"กกA\\nB\"", "กกA\nB"),
        ("Latin...Thai then LF   \"Aก\\nB\"",  "Aก\nB"),
        ("pure latin             \"AB\\nC\"",   "AB\nC"),
        ("pure thai              \"กก\\nกก\"",  "กก\nกก"),
        ("space before LF        \"กก \\nกก\"", "กก \nกก"),
        ("LF first               \"\\nกก\"",    "\nกก"),
    ] {
        let n = lines(s, "Arial Unicode MS", None);
        let expect = s.matches('\n').count() + 1;
        println!("   {id:<34} -> {n}/{expect}  {}", if n == expect { "ok" } else { "**DROPPED**" });
    }
    println!();

    println!("# 4. MINIMAL REPRO (jury B's #1) -- smallest failing input");
    let m = "กก\nกก";
    println!("   text {m:?}  bytes {:?}", m.as_bytes());
    println!("   parley::break_all_lines(None) -> {} line(s), expected 2", lines(m, "Arial Unicode MS", None));
}

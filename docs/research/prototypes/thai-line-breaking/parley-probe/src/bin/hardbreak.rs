// UAX #14 gives LF class BK -- a MANDATORY break. ADR-0007 makes `\n` the only
// line-break mechanism in the whole format, so a stack that drops one is not
// "worse at Thai", it is unable to render the format at all.
use parley::{FontContext, FontFamily, LayoutContext, StyleProperty};

const CASES: &[(&str, &str, &str)] = &[
    ("latin",       "SF Pro Rounded", "I'm holding a big\nspider in my hand."),
    ("thai",        "Ayuthaya",       "ฉันกำลังถือแมงมุม\nตัวใหญ่ไว้ในมือ"),
    ("thai-2",      "Ayuthaya",       "ฉันแขวนโครงกระดูก\nไว้ข้างประตู"),
    ("thai-latin-neighbours", "Ayuthaya", "AAA\nBBB"),
    ("thai-then-latin", "Ayuthaya",   "ฉันแขวน\nBBB"),
    ("latin-then-thai", "Ayuthaya",   "AAA\nไว้ข้างประตู"),
    ("khmer",       "Khmer Sangam MN","ការធ្វើដំណើរ\nទៅកាន់ប្រទេស"),
    ("lao",         "Lao Sangam MN",  "ການເດີນທາງ\nໄປປະເທດລາວ"),
    ("japanese",    "Hiragino Sans",  "大きなクモを\n手に持っています。"),
    ("thai-crlf",   "Ayuthaya",       "ฉันกำลังถือแมงมุม\r\nตัวใหญ่ไว้ในมือ"),
    ("thai-double", "Ayuthaya",       "ฉันกำลังถือแมงมุม\n\nตัวใหญ่ไว้ในมือ"),
];

fn main() {
    println!("# parley 0.11.1  complex-scripts={}", cfg!(feature = "complex-scripts"));
    let mut fcx = FontContext::new();
    let mut lcx: LayoutContext<[u8; 4]> = LayoutContext::new();
    let mut broken = 0;
    for (id, family, text) in CASES {
        let mut b = lcx.ranged_builder(&mut fcx, text, 1.0, false);
        b.push_default(StyleProperty::FontSize(57.0));
        b.push_default(StyleProperty::FontFamily(FontFamily::named(family)));
        let mut l = b.build(text);
        l.break_all_lines(None);          // no box at all: only mandatory breaks apply
        let n = l.lines().count();
        let expect = text.matches('\n').count() + 1;
        let ok = if n == expect { "ok" } else { broken += 1; "**DROPPED**" };
        println!("{id:<24} \\n x{}  ->  {n} line(s)  expected {expect}  {ok}",
                 text.matches('\n').count());
    }
    println!("\nmandatory breaks dropped in {broken} of {} cases", CASES.len());
}

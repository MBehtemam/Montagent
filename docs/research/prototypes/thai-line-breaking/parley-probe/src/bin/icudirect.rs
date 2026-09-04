// Is the dropped mandatory break parley's or ICU4X's? Ask icu_segmenter directly.
// UAX #14 gives LF class BK: a mandatory break, unconditional.
use icu_segmenter::LineSegmenter;
use icu_segmenter::options::LineBreakOptions;

const CASES: &[(&str, &str)] = &[
    ("latin",    "AAA\nBBB"),
    ("thai",     "ฉันกำลังถือแมงมุม\nตัวใหญ่ไว้ในมือ"),
    ("thai-then-latin", "ฉันแขวน\nBBB"),
    ("latin-then-thai", "AAA\nไว้ข้างประตู"),
    ("khmer",    "ការធ្វើដំណើរ\nទៅកាន់ប្រទេស"),
    ("japanese", "大きなクモを\n手に持っています。"),
];

fn main() {
    let seg = LineSegmenter::new_dictionary(LineBreakOptions::default());
    for (id, text) in CASES {
        let nl = text.find('\n').unwrap();
        let breaks: Vec<usize> = seg.segment_str(text).collect();
        // A mandatory break at the LF means a boundary at nl+1.
        let honoured = breaks.contains(&(nl + 1));
        println!("{id:<18} LF at byte {nl:>3}  boundary at {}? {}   breaks {:?}",
                 nl + 1, if honoured { "YES" } else { "**NO**" }, breaks);
    }
}

// Same mandatory-break test as the parley side, through cosmic-text.
#[path = "../../../render.rs"]
mod render;

use std::sync::Arc;
use render::PlacedGlyph;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};

const CASES: &[(&str, &str, &str)] = &[
    ("latin",       "Helvetica",      "I'm holding a big\nspider in my hand."),
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
    println!("# cosmic-text 0.19.0  wrap=None (only mandatory breaks apply)");
    let mut fs = FontSystem::new();
    let mut dropped = 0;
    for (id, family, text) in CASES {
        let mut buf = Buffer::new(&mut fs, Metrics::new(57.0, 62.7));
        buf.set_wrap(Wrap::None);
        buf.set_size(None, None);
        buf.set_text(text, &Attrs::new().family(Family::Name(family)), Shaping::Advanced, None);
        buf.shape_until_scroll(&mut fs, false);
        let n = buf.layout_runs().count();
        let expect = text.matches('\n').count() + 1;
        let ok = if n == expect { "ok" } else { dropped += 1; "**DROPPED**" };
        println!("{id:<24} \\n x{}  ->  {n} line(s)  expected {expect}  {ok}",
                 text.matches('\n').count());

        if std::env::args().nth(1).is_some() && (*id == "thai" || *id == "latin") {
            const PAD: f32 = 30.0;
            let mut placed = Vec::new();
            let mut bottom = 0.0f32;
            for run in buf.layout_runs() {
                bottom = bottom.max(run.line_top + run.line_height);
                for g in run.glyphs {
                    if let Some(d) = fs.db().with_face_data(g.font_id, |data, index| {
                        (Arc::new(data.to_vec()), index)
                    }) {
                        placed.push(PlacedGlyph { font_data: d.0, font_index: d.1,
                            glyph_id: g.glyph_id, x: g.x + PAD,
                            y: run.line_y + g.y + PAD, size_px: g.font_size });
                    }
                }
            }
            render::render_png(&placed, (984.0 + PAD * 2.0) as u32,
                (bottom + PAD * 2.0) as u32, 984.0 + PAD,
                &format!("hardbreak-cosmic-{id}.png")).unwrap();
        }
    }
    println!("\nmandatory breaks dropped in {dropped} of {} cases", CASES.len());
}

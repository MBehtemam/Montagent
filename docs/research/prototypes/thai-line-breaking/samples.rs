//! The one shared input for both probes. Neither binary defines its own text,
//! font, size or box width, so a difference in the output is the text stack and
//! not the scene.

pub struct Sample {
    /// Stable id, used as the output key.
    pub id: &'static str,
    /// Requested font family. Both stacks resolve this through system font
    /// discovery on the same machine; each probe prints the family it actually
    /// resolved so a mismatch is visible rather than assumed.
    pub family: &'static str,
    /// Font size in px.
    pub size: f32,
    /// Fixed box width in px. The whole point: same box, same text, both stacks.
    pub width: f32,
    pub text: &'static str,
    /// What correct output looks like, written before running anything.
    /// `|` marks an expected break opportunity (not a forced break).
    pub expect: &'static str,
}

pub const SAMPLES: &[Sample] = &[
    Sample {
        id: "thai",
        family: "Ayuthaya",
        size: 32.0,
        width: 420.0,
        text: "การเดินทางไปประเทศไทยเป็นประสบการณ์ที่ดีมากสำหรับนักท่องเที่ยวทุกคน",
        expect: "การ|เดิน|ทาง|ไป|ประเทศ|ไทย|เป็น|ประสบการณ์|ที่|ดี|มาก|สำหรับ|นัก|ท่องเที่ยว|ทุก|คน \
                 -- Thai writes without spaces, so every break here needs dictionary or LSTM \
                 segmentation. A stack that resolves SA to AL sees one unbreakable 67-char word.",
    },
    Sample {
        id: "thai-with-spaces",
        family: "Ayuthaya",
        size: 32.0,
        width: 420.0,
        text: "ฉันชอบอาหารไทย เพราะรสชาติอร่อยมาก และราคาไม่แพงเกินไปสำหรับนักเรียน",
        expect: "Thai uses spaces at clause boundaries only (2 here). A stack with no Thai \
                 segmentation still breaks at those 2 spaces -- which is what makes the failure \
                 plausible instead of obvious: the output looks laid out, just wrongly.",
    },
    Sample {
        id: "thai-2",
        family: "Ayuthaya",
        size: 32.0,
        width: 420.0,
        text: "เด็กนักเรียนทุกคนต้องอ่านหนังสือก่อนสอบปลายภาคเรียนนี้",
        expect: "A second unspaced Thai paragraph, added after the first one exposed \
                 mid-word boundaries, to tell a bad sample apart from a bad segmenter.",
    },
    Sample {
        id: "thai-3",
        family: "Ayuthaya",
        size: 32.0,
        width: 420.0,
        text: "รัฐบาลประกาศมาตรการใหม่เพื่อช่วยเหลือผู้ประกอบการขนาดเล็กในปีหน้า",
        expect: "A third unspaced Thai paragraph, news register rather than tourism.",
    },
    Sample {
        id: "khmer",
        family: "Khmer Sangam MN",
        size: 32.0,
        width: 420.0,
        text: "ការធ្វើដំណើរទៅកាន់ប្រទេសកម្ពុជាគឺជាបទពិសោធន៍ដ៏អស្ចារ្យសម្រាប់អ្នកទេសចរណ៍",
        expect: "ការ|ធ្វើ|ដំណើរ|ទៅ|កាន់|ប្រទេស|កម្ពុជា|គឺជា|បទពិសោធន៍|ដ៏|អស្ចារ្យ|សម្រាប់|អ្នក|ទេសចរណ៍ \
                 -- same shape as Thai. Also the case where icu4x#7218 (Khmer spacing) would bite.",
    },
    Sample {
        id: "khmer-with-spaces",
        family: "Khmer Sangam MN",
        size: 32.0,
        width: 420.0,
        text: "អស់ នឹង មាន ចន្លោះ អស់ នឹង មាន ចន្លោះ អស់ នឹង មាន ចន្លោះ។",
        expect: "The exact shape of icu4x#7218 (open, T-bug, filed 2025-11-04): the segmenter \
                 offers a break BEFORE each space as well as after, isolating the space. \
                 Upstream's own triage says a layout engine trims trailing spaces so it does \
                 not surface -- so the check is whether parley trims. A line that STARTS with \
                 a space is the bug reaching the frame.",
    },
    Sample {
        id: "lao",
        family: "Lao Sangam MN",
        size: 32.0,
        width: 420.0,
        text: "ການເດີນທາງໄປປະເທດລາວແມ່ນປະສົບການທີ່ດີຫຼາຍສຳລັບນັກທ່ອງທ່ຽວທຸກຄົນ",
        expect: "Same shape as Thai; named in parley's complex-scripts feature text.",
    },
    Sample {
        id: "japanese",
        family: "Hiragino Sans",
        size: 32.0,
        width: 420.0,
        text: "日本語のテキストは単語の区切りに空白を使わないので、行の折り返しは文字単位で行われます。",
        expect: "UAX #14 gives ideographs class ID and kana CJ, both breakable on either side \
                 with no dictionary at all. So BOTH stacks should break this acceptably, and \
                 CJK should NOT discriminate. Kinsoku is the only open question: the line must \
                 not begin with the trailing 。 or 、.",
    },
    Sample {
        id: "chinese",
        family: "Hiragino Sans GB",
        size: 32.0,
        width: 420.0,
        text: "中文文本在排版时不使用空格分词，因此换行可以发生在几乎任意两个汉字之间。",
        expect: "All class ID. Same prediction as japanese: no dictionary needed, no discrimination.",
    },
    Sample {
        id: "latin-control",
        family: "Helvetica",
        size: 32.0,
        width: 420.0,
        text: "The quick brown fox jumps over the lazy dog near the riverbank at dawn.",
        expect: "The control. Both stacks must agree here; if they do not, the harness itself \
                 is measuring something other than line breaking.",
    },
];

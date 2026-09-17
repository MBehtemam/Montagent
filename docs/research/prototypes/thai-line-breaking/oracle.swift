// Independent word-boundary oracle: macOS CFStringTokenizer, the system's own
// Thai/Khmer/Lao/CJK segmenter. Used to check parley's dictionary output against
// something that is not parley and not this prototype's author.
import Foundation

let samples: [(String, String, String)] = [
  ("thai", "th", "การเดินทางไปประเทศไทยเป็นประสบการณ์ที่ดีมากสำหรับนักท่องเที่ยวทุกคน"),
  ("thai-with-spaces", "th", "ฉันชอบอาหารไทย เพราะรสชาติอร่อยมาก และราคาไม่แพงเกินไปสำหรับนักเรียน"),
  ("thai-2", "th", "เด็กนักเรียนทุกคนต้องอ่านหนังสือก่อนสอบปลายภาคเรียนนี้"),
  ("thai-3", "th", "รัฐบาลประกาศมาตรการใหม่เพื่อช่วยเหลือผู้ประกอบการขนาดเล็กในปีหน้า"),
  ("khmer", "km", "ការធ្វើដំណើរទៅកាន់ប្រទេសកម្ពុជាគឺជាបទពិសោធន៍ដ៏អស្ចារ្យសម្រាប់អ្នកទេសចរណ៍"),
  ("khmer-with-spaces", "km", "អស់ នឹង មាន ចន្លោះ អស់ នឹង មាន ចន្លោះ អស់ នឹង មាន ចន្លោះ។"),
  ("lao", "lo", "ການເດີນທາງໄປປະເທດລາວແມ່ນປະສົບການທີ່ດີຫຼາຍສຳລັບນັກທ່ອງທ່ຽວທຸກຄົນ"),
  ("myanmar", "my", "သီဟိုဠ်မှဉာဏ်ကြီးရှင်သည်အာယုဝဍ္ဎနဆေးညွှန်းစာကိုဇလွန်ဈေးဘေးဗာဒံပင်ထက်အဓိဋ္ဌာန်လျက်ဂဃနဏဖတ်ခဲ့သည်။"),
  ("myanmar-with-spaces", "my", "သီဟိုဠ်မှ ဉာဏ်ကြီးရှင်သည် အာယုဝဍ္ဎနဆေးညွှန်းစာကို ဇလွန်ဈေးဘေး ဗာဒံပင်ထက် အဓိဋ္ဌာန်လျက် ဂဃနဏဖတ်ခဲ့သည်။"),
  ("japanese", "ja", "日本語のテキストは単語の区切りに空白を使わないので、行の折り返しは文字単位で行われます。"),
]

for (id, locale, text) in samples {
  let cf = text as CFString
  let range = CFRangeMake(0, CFStringGetLength(cf))
  let tok = CFStringTokenizerCreate(nil, cf, range,
              kCFStringTokenizerUnitWordBoundary,
              Locale(identifier: locale) as CFLocale)
  var words: [String] = []
  var starts: [Int] = []
  let utf16 = Array(text.utf16)
  while CFStringTokenizerAdvanceToNextToken(tok) != [] {
    let r = CFStringTokenizerGetCurrentTokenRange(tok)
    let piece = String(utf16CodeUnits: Array(utf16[r.location..<(r.location + r.length)]),
                       count: r.length)
    // Convert the UTF-16 start offset to a UTF-8 byte offset so it lines up
    // with what the Rust probes print.
    let prefix = String(utf16CodeUnits: Array(utf16[0..<r.location]), count: r.location)
    starts.append(prefix.utf8.count)
    words.append(piece)
  }
  print("## \(id) (locale \(locale))")
  print("segmentation: \(words.joined(separator: "|"))")
  print("starts: \(starts)")
  print("")
}

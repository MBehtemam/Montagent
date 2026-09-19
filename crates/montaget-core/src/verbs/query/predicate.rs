//! `--where`'s predicate: the smallest language that expresses the question the ADR asks.
//!
//! ADR-0011 specifies `query --where <predicate> [--census <field>]` and does not say what
//! a predicate looks like. **ADR-0070 does**, and this file implements it: the grammar is
//! spec, so a change here amends that ADR. It was chosen against ADR-0011's own worked
//! example — *"four of five siblings agree and one does not"* — and against its cost
//! argument for the whole verb: a predicate an agent has to look up is a predicate it will
//! write a `jq` filter instead of.
//!
//! ```text
//! predicate := term (`and` term)*
//! term      := path op literal | path `exists` | path `missing`
//! op        := `=` | `!=` | `<` | `<=` | `>` | `>=`
//! path      := segment (`.` segment)*
//! segment   := a key, an array index, or `*` for every member of an array
//! ```
//!
//! Three deliberate absences (ADR-0070 ratifies each, with the cost of each stated):
//!
//! - **No `or`.** Two calls answer it, and a disjunction is the first step towards an
//!   expression language whose grammar has to be published as a resource. `and` is kept
//!   because the ADR's own example — *siblings*, a set narrowed twice — needs it.
//! - **No resolution of any kind.** A path names what the document *writes*, so
//!   `--where 'y = 1597'` matches an element whose `y` is the number 1597 and not one
//!   whose `y` is a keyframe list passing through 1597 at some instant. #196's acceptance
//!   criterion is that this mode works with no resolver present, and a predicate that
//!   interpolated would make that untrue by construction. It is also why `layer` compares
//!   against the written value rather than the integer ADR-0019's one hop would produce:
//!   an anchor is `{"below": "card"}` in the file, and that is what a document-literal
//!   predicate sees.
//! - **No regular expressions or substring matching.** `=` on a whole value is what a
//!   census groups by, and a half-matching predicate would make the distribution beneath
//!   it unreadable.
//!
//! ## Where the argument is had
//!
//! Every decision above is a real format-surface decision rather than implementation
//! detail — it is what an agent has to learn, and it is as good as permanent once an
//! agent's prompts contain it. ADR-0011 owned none of it, which
//! [#249](https://github.com/MBehtemam/Montaget/issues/249) raised in the same spirit as
//! #241; [ADR-0070](../../../../../docs/adr/0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)
//! settled it. A reader who disagrees with the shape argues with that ADR, not with this
//! file.
//!
//! ## The one reserved name
//!
//! **`track`** is the containing track's `name`, always. An element does not carry the
//! name of the track it sits in — the nesting is the statement — and filtering by track is
//! the most ordinary question there is. The reservation is unconditional rather than a
//! fallback for elements that write no `track` key, because a rule with an exception is a
//! rule an agent has to test: ADR-0017's closed schema gives no element type a `track`
//! field, so there is nothing for the reservation to shadow — checked, not assumed, by
//! `docs/adr/predicate_reserved_names_scan.py`, which also holds the schema to the
//! no-all-digit-key premise the index segment above rests on.

use serde_json::Value;

/// One `--where` expression, as a conjunction of terms. Every term must hold.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    terms: Vec<Term>,
}

/// One comparison, or one presence claim.
#[derive(Debug, Clone, PartialEq)]
struct Term {
    path: Path,
    test: Test,
}

#[derive(Debug, Clone, PartialEq)]
enum Test {
    /// A comparison against a literal.
    Compare(Op, Value),
    /// The path reaches at least one value.
    Exists,
    /// The path reaches none. ADR-0030 makes a field's *presence* content, so this is a
    /// question about the document rather than a convenience.
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    fn spelling(self) -> &'static str {
        match self {
            Op::Eq => "=",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
        }
    }
}

/// A dotted path into an element.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Key(String),
    Index(usize),
    /// `*` — every member of an array, which is what makes `runs.*.font` a question about
    /// an element rather than about its first run.
    Every,
}

/// The reserved first segment: the name of the track the element sits in.
pub const TRACK: &str = "track";

impl Path {
    /// Parse a dotted path, or say what is wrong with it.
    pub fn parse(text: &str) -> Result<Path, String> {
        if text.is_empty() {
            return Err("a field path is empty".into());
        }
        let mut segments = Vec::new();
        for part in text.split('.') {
            if part.is_empty() {
                return Err(format!("`{text}` has an empty segment"));
            }
            segments.push(match part {
                "*" => Segment::Every,
                // An all-digit segment indexes an array. Not because anything asks for
                // `runs.0.font`, but because a digit segment has to mean *something*: read
                // as a key it would reach nothing on every array in the format, and a path
                // that silently matches nothing is the worse of the two answers. A JSON
                // object may legally have an all-digit key and the format has none — every
                // object key the schema declares is an identifier.
                digits if digits.chars().all(|c| c.is_ascii_digit()) => Segment::Index(
                    digits
                        .parse()
                        .map_err(|_| format!("`{part}` is not an array index"))?,
                ),
                key => Segment::Key(key.to_string()),
            });
        }
        Ok(Path { segments })
    }

    /// The path as it was written, which is what the answer names the census field by.
    pub fn spelling(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Key(key) => key.clone(),
                Segment::Index(index) => index.to_string(),
                Segment::Every => "*".to_string(),
            })
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Whether this path is the reserved `track`.
    fn is_track(&self) -> bool {
        matches!(self.segments.as_slice(), [Segment::Key(key)] if key == TRACK)
    }

    /// Every value this path reaches on one element, in document order.
    ///
    /// Empty where the path reaches nothing — which is the *absent* case ADR-0030 makes
    /// content, and is why this returns a list rather than an `Option<&Value>` with a
    /// `Value::Null` standing in for both a written `null` and a key that was never
    /// written.
    pub fn values<'a>(&self, element: &'a Value, track: Option<&'a Value>) -> Vec<&'a Value> {
        if self.is_track() {
            return track.into_iter().collect();
        }
        let mut here = vec![element];
        for segment in &self.segments {
            let mut next = Vec::new();
            for value in here {
                match segment {
                    Segment::Key(key) => next.extend(value.get(key)),
                    Segment::Index(index) => next.extend(value.get(index)),
                    Segment::Every => {
                        if let Some(items) = value.as_array() {
                            next.extend(items);
                        }
                    }
                }
            }
            if next.is_empty() {
                return Vec::new();
            }
            here = next;
        }
        here
    }
}

impl Predicate {
    /// Parse a `--where` expression, or say what is wrong with it in one sentence.
    ///
    /// The error is a sentence rather than a position-and-caret, because unlike a parse
    /// failure in a *file* there is no line to point at: the whole predicate is one
    /// argument, and it is in front of whoever typed it.
    pub fn parse(text: &str) -> Result<Predicate, String> {
        let mut scanner = Scanner::new(text);
        let mut terms = vec![scanner.term()?];
        while scanner.keyword("and")? {
            terms.push(scanner.term()?);
        }
        scanner.end()?;
        Ok(Predicate { terms })
    }

    /// Does this element satisfy every term?
    ///
    /// `track` is the containing track's name, as a JSON value, or `None` where the
    /// element sits in a track that writes none.
    pub fn matches(&self, element: &Value, track: Option<&Value>) -> bool {
        self.terms.iter().all(|term| term.matches(element, track))
    }
}

impl Term {
    fn matches(&self, element: &Value, track: Option<&Value>) -> bool {
        let values = self.path.values(element, track);
        match &self.test {
            Test::Exists => !values.is_empty(),
            Test::Missing => values.is_empty(),
            // *Any* value at the path satisfies it. For a single-valued path — which is
            // every path with no `*` in it — this is ordinary comparison; for `runs.*.font`
            // it reads as "has a run whose font is …", which is the question an element-set
            // query is asking. It also means `!=` is "has a value that is not", never "has
            // no value that is": a path reaching nothing matches neither `=` nor `!=`, so
            // absence is asked about with `missing` rather than falling out of a comparison.
            Test::Compare(op, literal) => values.iter().any(|value| compare(*op, value, literal)),
        }
    }
}

/// One comparison, over the two orderings JSON actually has.
///
/// An ordering comparison between values of different kinds — `<` against a string on one
/// element and a number on another — is **no match** rather than an error. The permissive
/// tree is the spine every read goes through (ADR-0042), so a query is routinely run over a
/// document holding an `fps` of `"25"`, and a verb that aborted on the first such element
/// would answer nothing about the other fifty-nine.
fn compare(op: Op, value: &Value, literal: &Value) -> bool {
    match op {
        Op::Eq => equal(value, literal),
        Op::Ne => !equal(value, literal),
        Op::Lt | Op::Le | Op::Gt | Op::Ge => match order(value, literal) {
            Some(ordering) => match op {
                Op::Lt => ordering.is_lt(),
                Op::Le => ordering.is_le(),
                Op::Gt => ordering.is_gt(),
                Op::Ge => ordering.is_ge(),
                Op::Eq | Op::Ne => unreachable!("handled above"),
            },
            None => false,
        },
    }
}

/// JSON equality, with numbers compared **numerically**.
///
/// `serde_json` holds `1` and `1.0` as different numbers and they compare unequal, which
/// would make `--where 'opacity = 1'` miss an element whose document writes `1.0`. Every
/// other kind compares structurally.
fn equal(value: &Value, literal: &Value) -> bool {
    match (value.as_f64(), literal.as_f64()) {
        (Some(a), Some(b)) if value.is_number() && literal.is_number() => a == b,
        _ => value == literal,
    }
}

/// The ordering of two values, where they have one: numbers numerically, strings by their
/// code points. Everything else — a boolean, an object, two kinds that differ — has none.
fn order(value: &Value, literal: &Value) -> Option<std::cmp::Ordering> {
    if let (Some(a), Some(b)) = (value.as_f64(), literal.as_f64())
        && value.is_number()
        && literal.is_number()
    {
        return a.partial_cmp(&b);
    }
    match (value.as_str(), literal.as_str()) {
        (Some(a), Some(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

/// The hand-written scanner. Small enough to read in one sitting, which is the point: the
/// grammar above is the whole language, and a parser generator would hide that.
struct Scanner<'a> {
    text: &'a str,
    at: usize,
}

/// The characters a path segment may use. `*` is the wildcard and `.` the separator; the
/// rest are what the format's own keys and this scanner's array indices are made of.
fn is_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '*')
}

impl<'a> Scanner<'a> {
    fn new(text: &'a str) -> Self {
        Scanner { text, at: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.text[self.at..]
    }

    fn skip_space(&mut self) {
        let trimmed = self.rest().trim_start();
        self.at = self.text.len() - trimmed.len();
    }

    /// One term: a path, then either a presence word or an operator and a literal.
    fn term(&mut self) -> Result<Term, String> {
        self.skip_space();
        let start = self.at;
        let taken: String = self
            .rest()
            .chars()
            .take_while(|c| is_path_char(*c))
            .collect();
        if taken.is_empty() {
            return Err(match self.rest().chars().next() {
                Some(c) => format!(
                    "expected a field name at `{}`, found `{c}`",
                    &self.text[start..]
                ),
                None => format!("`{}` ends where a field name was expected", self.text),
            });
        }
        self.at += taken.len();
        let path = Path::parse(&taken)?;

        self.skip_space();
        if self.keyword("exists")? {
            return Ok(Term {
                path,
                test: Test::Exists,
            });
        }
        if self.keyword("missing")? {
            return Ok(Term {
                path,
                test: Test::Missing,
            });
        }

        let op = self.op(&taken)?;
        self.skip_space();
        let literal = self.literal(&taken)?;
        Ok(Term {
            path,
            test: Test::Compare(op, literal),
        })
    }

    /// The longer spellings first, so `!=` and `<=` are never read as `!` and `<`.
    fn op(&mut self, path: &str) -> Result<Op, String> {
        for op in [Op::Ne, Op::Le, Op::Ge, Op::Eq, Op::Lt, Op::Gt] {
            if let Some(rest) = self.rest().strip_prefix(op.spelling()) {
                self.at = self.text.len() - rest.len();
                return Ok(op);
            }
        }
        Err(format!(
            "`{path}` is followed by `{}`, which is not one of `=`, `!=`, `<`, `<=`, `>`, \
             `>=`, `exists` or `missing`",
            self.rest().trim_end()
        ))
    }

    /// A quoted string, or a bare word read as the JSON scalar it spells.
    fn literal(&mut self, path: &str) -> Result<Value, String> {
        let mut chars = self.rest().chars();
        match chars.next() {
            Some(quote @ ('\'' | '"')) => {
                let body: String = chars.by_ref().take_while(|c| *c != quote).collect();
                // `take_while` consumed the closing quote if it found one; if it ran to the
                // end instead, the literal never closed.
                let consumed = quote.len_utf8() + body.len();
                if self.rest().len() == consumed {
                    return Err(format!(
                        "`{path}`'s value opens with {quote} and never closes"
                    ));
                }
                self.at += consumed + quote.len_utf8();
                Ok(Value::String(body))
            }
            Some(_) => {
                let taken: String = self
                    .rest()
                    .chars()
                    .take_while(|c| !c.is_whitespace())
                    .collect();
                self.at += taken.len();
                Ok(scalar(&taken))
            }
            None => Err(format!("`{path}` is compared against nothing")),
        }
    }

    /// Consume `word` if it is the next whole word.
    fn keyword(&mut self, word: &str) -> Result<bool, String> {
        self.skip_space();
        let Some(rest) = self.rest().strip_prefix(word) else {
            return Ok(false);
        };
        // A whole word, so that a path beginning `android` is never read as `and` followed
        // by `roid`.
        if rest.chars().next().is_some_and(is_path_char) {
            return Ok(false);
        }
        self.at = self.text.len() - rest.len();
        Ok(true)
    }

    fn end(&mut self) -> Result<(), String> {
        self.skip_space();
        if self.rest().is_empty() {
            return Ok(());
        }
        let left_over = self.rest().trim_end().to_string();
        // `or` is absent by decision (ADR-0070) rather than by oversight, so the refusal
        // names the decision and the move that answers it. Reported as leftover text, the
        // caller's next guess is a spelling — `||`, `OR` — and the turn is spent on syntax
        // rather than on learning that the language has no disjunction at all.
        if self.keyword("or")? {
            return Err(format!(
                "`{left_over}` is left over: there is no `or` — two calls answer a \
                 disjunction, one per term, and the union of their matched sets is the answer"
            ));
        }
        Err(format!(
            "`{left_over}` is left over; terms are joined with `and`"
        ))
    }
}

/// A bare word as the JSON scalar it spells.
///
/// Quoting forces a string, so `--where 'id = "true"'` reaches an element whose id is the
/// word and `--where 'loop = true'` reaches the boolean. Without the distinction one of
/// those two is unaskable.
fn scalar(word: &str) -> Value {
    match word {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" => Value::Null,
        _ => serde_json::from_str::<Value>(word)
            .ok()
            .filter(Value::is_number)
            .unwrap_or_else(|| Value::String(word.to_string())),
    }
}

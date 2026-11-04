//! ADR-0244 / Issue #1141: Skill verification engine.
//!
//! Validates installed skills against the active binary's commands, flags,
//! registered finding codes, published resources, and project schemas.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::finding::Finding;
use crate::report::Report;

/// Arguments configuring skill inspection.
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// Specific skill file or directory to inspect.
    pub target: Option<PathBuf>,
    /// Root workspace directory for default discovery.
    pub root: Option<PathBuf>,
}

/// A node in the CLI command grammar tree.
#[derive(Debug, Clone)]
struct CommandDef {
    name: &'static str,
    aliases: &'static [&'static str],
    subcommands: &'static [CommandDef],
    flags: &'static [&'static str],
}

static CACHE_SUBCOMMANDS: &[CommandDef] = &[CommandDef {
    name: "clear",
    aliases: &[],
    subcommands: &[],
    flags: &["--json", "-h", "--help"],
}];

static FONTS_SUBCOMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "acquire",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--url",
            "--google-family",
            "--google-file",
            "--revision",
            "--weight",
            "--style",
            "--family",
            "--face-index",
            "--expected-sha256",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "list",
        aliases: &[],
        subcommands: &[],
        flags: &["--root", "--json", "-h", "--help"],
    },
    CommandDef {
        name: "vendor",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--licence",
            "--source",
            "--as",
            "--destination",
            "--receipt",
            "--resolution",
            "--face-index",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
];

static SKILLS_SUBCOMMANDS: &[CommandDef] = &[CommandDef {
    name: "check",
    aliases: &[],
    subcommands: &[],
    flags: &["--json", "--verbose", "-h", "--help"],
}];

static COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "validate",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--quiet",
            "--quiet-review",
            "--quiet-reviews",
            "--no-quiet",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "create-project",
        aliases: &["create_project"],
        subcommands: &[],
        flags: &[
            "--width",
            "--height",
            "--fps",
            "--background",
            "--duration",
            "--output",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "fmt",
        aliases: &[],
        subcommands: &[],
        flags: &["--check", "--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "cache",
        aliases: &[],
        subcommands: CACHE_SUBCOMMANDS,
        flags: &["-h", "--help"],
    },
    CommandDef {
        name: "probe",
        aliases: &[],
        subcommands: &[],
        flags: &["--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "timeline",
        aliases: &[],
        subcommands: &[],
        flags: &["--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "query",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--at",
            "--from",
            "--to",
            "--where",
            "--census",
            "--range",
            "--kind",
            "--id",
            "--property",
            "--order",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "frame",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--at",
            "--from",
            "--to",
            "--keyframes",
            "--infill-ceiling",
            "--crop",
            "--png",
            "--span",
            "--full",
            "--sheet",
            "--output",
            "--out",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "measure",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--at",
            "--span",
            "--element",
            "--elements",
            "--all",
            "--scopes",
            "--layout",
            "--ranges",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "render",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--from",
            "--to",
            "--output",
            "--out",
            "--no-clobber",
            "--progress-file",
            "--profile",
            "--stems-only",
            "--preset",
            "--crf",
            "--estimate",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "verify",
        aliases: &[],
        subcommands: &[],
        flags: &["--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "preview",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--from",
            "--to",
            "--output",
            "--out",
            "--full",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "shift",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--at",
            "--delta",
            "--scope",
            "--delete",
            "--release",
            "--by",
            "--dry-run",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "compare",
        aliases: &[],
        subcommands: &[],
        flags: &["--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "fonts",
        aliases: &[],
        subcommands: FONTS_SUBCOMMANDS,
        flags: &["-h", "--help"],
    },
    CommandDef {
        name: "import",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--format",
            "--output",
            "-o",
            "--width",
            "--height",
            "--fps",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "export",
        aliases: &[],
        subcommands: &[],
        flags: &[
            "--format",
            "--output",
            "-o",
            "--json",
            "--verbose",
            "-h",
            "--help",
        ],
    },
    CommandDef {
        name: "skills",
        aliases: &[],
        subcommands: SKILLS_SUBCOMMANDS,
        flags: &["-h", "--help"],
    },
    CommandDef {
        name: "skills-check",
        aliases: &["skills_check"],
        subcommands: &[],
        flags: &["--json", "--verbose", "-h", "--help"],
    },
    CommandDef {
        name: "mcp",
        aliases: &[],
        subcommands: &[],
        flags: &["-h", "--help"],
    },
];

/// Inspect skills according to `options` and return a structured report.
pub fn check(options: &CheckOptions) -> Report {
    let root = options
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let target_dir = match &options.target {
        Some(t) => {
            if !t.exists() {
                return Report::bad_invocation(format!(
                    "target path does not exist: {}",
                    t.display()
                ));
            }
            t.clone()
        }
        None => {
            let candidates = [
                root.join(".agents/skills"),
                root.join(".claude/skills"),
                root.join("skills"),
            ];
            let found = candidates.into_iter().find(|c| c.is_dir());
            match found {
                Some(dir) => dir,
                None => {
                    return Report::bad_invocation(
                        "no skills directory detected under .agents/skills, .claude/skills, or skills",
                    );
                }
            }
        }
    };

    let files = collect_skill_files(&target_dir);
    if files.is_empty() {
        return Report::bad_invocation(format!(
            "no skill markdown files found in {}",
            target_dir.display()
        ));
    }

    let mut report = Report::new("skills check", None);

    for file in files {
        check_file(&file, &mut report);
    }

    report
}

fn collect_skill_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    let mut result = Vec::new();
    let mut dirs = vec![path.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs.push(p);
            } else if p.is_file()
                && (p.file_name() == Some(std::ffi::OsStr::new("SKILL.md"))
                    || p.extension() == Some(std::ffi::OsStr::new("md")))
            {
                result.push(p);
            }
        }
    }

    result.sort();
    result
}

struct Block {
    line: usize,
    text: String,
    opted_out: BTreeSet<String>,
}

struct Fence {
    line: usize,
    info: String,
    body: String,
}

fn prose_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current: Option<Block> = None;
    let mut in_fence = false;

    for (n, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            blocks.extend(current.take());
            continue;
        }
        if in_fence || line.trim().is_empty() {
            blocks.extend(current.take());
            continue;
        }
        let block = current.get_or_insert_with(|| Block {
            line: n + 1,
            text: String::new(),
            opted_out: BTreeSet::new(),
        });
        block.text.push_str(line);
        block.text.push('\n');
    }
    blocks.extend(current);

    for block in &mut blocks {
        for (at, _) in block.text.match_indices("guard-ok:") {
            let rest = &block.text[at + "guard-ok:".len()..];
            let words = &rest[..rest.find("-->").unwrap_or(rest.len())];
            block
                .opted_out
                .extend(words.split_whitespace().map(str::to_string));
        }
    }
    blocks
}

fn fences(text: &str) -> Vec<Fence> {
    let mut result = Vec::new();
    let mut open: Option<Fence> = None;

    for (n, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        match (&mut open, trimmed.strip_prefix("```")) {
            (None, Some(info)) => {
                open = Some(Fence {
                    line: n + 1,
                    info: info.trim().to_string(),
                    body: String::new(),
                })
            }
            (Some(_), Some(_)) => result.extend(open.take()),
            (Some(fence), None) => {
                fence.body.push_str(line);
                fence.body.push('\n');
            }
            (None, None) => {}
        }
    }
    result
}

fn code_spans(text: &str) -> Vec<String> {
    let mut text = text.to_string();
    while let Some(start) = text.find("<!--") {
        let end = text[start..]
            .find("-->")
            .map_or(text.len(), |e| start + e + 3);
        text.replace_range(start..end, "");
    }
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_code(token: &str) -> bool {
    let mut parts = token.split('-');
    let prefix = parts.next().unwrap_or_default();
    prefix.len() == 1
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && token.contains('-')
        && parts.all(|p| {
            !p.is_empty()
                && p.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
}

fn uris(text: &str) -> Vec<String> {
    text.match_indices("montagent://")
        .map(|(at, _)| {
            let uri: String = text[at..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "._/:-".contains(*c))
                .collect();
            uri.trim_end_matches('.').to_string()
        })
        .collect()
}

fn check_file(file: &Path, report: &mut Report) {
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };
    let file_str = file.to_string_lossy().to_string();

    // 1. Prose blocks: check finding codes, URIs, commands/flags.
    for block in prose_blocks(&text) {
        // Resource URIs
        for uri in uris(&block.text) {
            if crate::resources::find(&uri).is_none() {
                report.push(
                    Finding::new("E-SKILL-RESOURCE-UNKNOWN")
                        .at_file(&file_str)
                        .at_line(block.line as u32)
                        .field("uri", json!(uri)),
                );
            }
        }

        // Code spans
        for span in code_spans(&block.text) {
            let tokens: Vec<&str> = span.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            // Check finding codes: e.g. `E-PARSE`
            let first = tokens[0];
            if is_code(first) && tokens.len() == 1 {
                if block.opted_out.contains(first) {
                    continue;
                }
                if crate::registry::spec(first).is_none() {
                    report.push(
                        Finding::new("E-SKILL-CODE-UNKNOWN")
                            .at_file(&file_str)
                            .at_line(block.line as u32)
                            .field("code", json!(first)),
                    );
                }
                continue;
            }

            // Check commands: e.g. `montagent validate`
            if first == "montagent" && tokens.len() > 1 {
                check_command(
                    &file_str,
                    block.line,
                    &tokens[1..],
                    &block.opted_out,
                    report,
                );
            }
        }
    }

    // 2. Fenced JSON blocks: validate project snippets.
    for fence in fences(&text) {
        let mut info_words = fence.info.split_whitespace();
        if info_words.next() != Some("json") || info_words.any(|w| w == "fragment") {
            continue;
        }

        validate_snippet(&file_str, fence.line, &fence.body, report);
    }
}

fn check_command(
    file: &str,
    line: usize,
    tokens: &[&str],
    opted_out: &BTreeSet<String>,
    report: &mut Report,
) {
    if tokens.is_empty() {
        return;
    }

    let mut current_cmds = COMMANDS;
    let mut path: Vec<&str> = Vec::new();
    let mut active_flags: &'static [&'static str] = &["--version", "-V", "--help", "-h"];
    let mut rest = tokens;

    while let Some((&token, tail)) = rest.split_first() {
        if token.starts_with('-') || token.starts_with('<') || current_cmds.is_empty() {
            break;
        }

        // Match subcommand
        if let Some(cmd) = current_cmds
            .iter()
            .find(|c| c.name == token || c.aliases.contains(&token))
        {
            path.push(cmd.name);
            current_cmds = cmd.subcommands;
            active_flags = cmd.flags;
            rest = tail;
        } else {
            if opted_out.contains(token) {
                rest = tail;
                continue;
            }
            let parent_cmd = if path.is_empty() {
                "montagent".to_string()
            } else {
                format!("montagent {}", path.join(" "))
            };
            report.push(
                Finding::new("E-SKILL-COMMAND-UNKNOWN")
                    .at_file(file)
                    .at_line(line as u32)
                    .field("command", json!(parent_cmd))
                    .field("subcommand", json!(token)),
            );
            return;
        }
    }

    // Verify flags from the remainder
    for raw_flag in rest {
        if !raw_flag.starts_with('-') {
            continue;
        }
        let flag = raw_flag.split('=').next().unwrap_or(raw_flag);
        if !flag.starts_with("--") {
            // Short flags or positional placeholders
            continue;
        }
        if opted_out.contains(flag) {
            continue;
        }
        if !active_flags.contains(&flag) {
            let parent_cmd = if path.is_empty() {
                "montagent".to_string()
            } else {
                format!("montagent {}", path.join(" "))
            };
            let suggested = suggest_flag(flag, active_flags);
            let suggested_str = match &suggested {
                Some(s) => format!(" (did you mean `{s}`?)"),
                None => String::new(),
            };
            let mut finding = Finding::new("E-SKILL-FLAG-UNKNOWN")
                .at_file(file)
                .at_line(line as u32)
                .field("command", json!(parent_cmd))
                .field("flag", json!(flag))
                .field("suggested", json!(suggested_str));
            if let Some(s) = suggested {
                finding = finding.field("suggestion", json!(s));
            }
            report.push(finding);
        }
    }
}

fn suggest_flag(flag: &str, valid_flags: &[&str]) -> Option<String> {
    let clean = flag.trim_start_matches('-');
    let mut best: Option<(&str, usize)> = None;

    for &valid in valid_flags {
        let clean_valid = valid.trim_start_matches('-');
        if clean_valid.contains(clean) || clean.contains(clean_valid) {
            return Some(valid.to_string());
        }
        let dist = levenshtein(clean, clean_valid);
        if dist <= 3 {
            match best {
                Some((_, best_dist)) if dist < best_dist => best = Some((valid, dist)),
                None => best = Some((valid, dist)),
                _ => {}
            }
        }
    }

    best.map(|(s, _)| s.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate() {
        *cell = j;
    }
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }
    matrix[a.len()][b.len()]
}

static SNIPPET_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn validate_snippet(file: &str, line: usize, body: &str, report: &mut Report) {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(body);
    let Ok(_) = parsed else {
        let err = parsed.unwrap_err();
        report.push(
            Finding::new("E-SKILL-SNIPPET-INVALID")
                .at_file(file)
                .at_line(line as u32)
                .field("detail", json!(format!("invalid JSON: {err}"))),
        );
        return;
    };

    let counter = SNIPPET_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_dir = std::env::temp_dir().join(format!(
        "montagent_skill_snippet_{}_{}",
        std::process::id(),
        counter
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let temp_file = temp_dir.join("snippet.montagent.json");
    if std::fs::write(&temp_file, body).is_err() {
        return;
    }

    let validation = crate::verbs::validate::validate(&temp_file);
    let _ = std::fs::remove_dir_all(&temp_dir);

    let errors: Vec<&Finding> = validation
        .findings
        .iter()
        .filter(|f| {
            f.class == crate::finding::Class::Error
                && !matches!(
                    f.code.as_str(),
                    "E-SOURCE-MISSING" | "E-FONT-MISSING" | "E-LUT-MISSING"
                )
        })
        .collect();

    if !errors.is_empty() {
        let error_codes: Vec<String> = errors
            .iter()
            .map(|f| format!("{}: {}", f.code, f.location.file))
            .collect();
        let detail = format!("{} error(s): {}", errors.len(), error_codes.join(", "));
        report.push(
            Finding::new("E-SKILL-SNIPPET-INVALID")
                .at_file(file)
                .at_line(line as u32)
                .field("detail", json!(detail)),
        );
    }
}

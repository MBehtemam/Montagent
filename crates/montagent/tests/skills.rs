//! The drift guard for `skills/` (#450, #513): the PR that renames or removes something
//! a skill names, or that breaks a recipe a skill teaches, fails here.
//!
//! Three checks, each against the binary this build produced rather than against a list
//! kept by hand:
//!
//! - **Names.** Finding codes in code spans (against the registry), `montagent://` URIs
//!   (against the published resources), `` `montagent <subcommand> --flag` `` (against the
//!   clap tree, read from `--help`), and a residual rule for every other bare word in a code
//!   span: it has to be an MCP tool, a CLI verb, a schema word, an id or name the skill's own
//!   snippets declare, or a skill. Keys and `$defs` are not scanned as such — snippets cover
//!   them — but a bare word that is none of the above fails, so a renamed verb does too.
//! - **Snippets.** Every fenced `json` block is a project, validated against the committed
//!   fixture set in `fixtures/skills/`. Errors fail, warnings don't, nothing is rendered.
//!   A fence marked `json fragment` is exempt.
//! - **Scripts.** Every `skills/*/scripts/*.py` is run on the Python its header names, with
//!   the arguments its header names, inside a copy of the fixture set, and what it prints
//!   is validated the same way.
//!
//! A code span the names rule should not judge is opted out in its own block with
//! `<!-- guard-ok: word -->`. The workaround markers and their lint are the other half of
//! the guard, and live in `ci/skills_workarounds.py` because they need the issue tracker.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

#[test]
fn skills_name_only_what_montagent_still_has() {
    let problems = check_names(&repo().join("skills"));
    assert!(
        problems.is_empty(),
        "stale names in skills/:\n{}",
        problems.join("\n")
    );
}

#[test]
fn skill_snippets_validate_against_the_fixture_set() {
    let problems = check_snippets(&repo().join("skills"), "snippets");
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn skill_scripts_print_projects_that_validate() {
    let problems = check_scripts(&repo().join("skills"), "scripts");
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn the_capability_map_names_every_element_type_and_effect() {
    let problems = check_completeness(&capability_map(&router_skill()));
    assert!(
        problems.is_empty(),
        "the capability map in skills/montagent/SKILL.md is missing:\n{}",
        problems.join("\n")
    );
}

#[test]
fn every_adr_a_not_by_design_line_cites_is_still_accepted() {
    let problems = check_refusals(&capability_map(&router_skill()), &repo().join("docs/adr"));
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Each check, seen to fail on a deliberately stale skill — so a guard that has quietly
/// stopped looking cannot pass by finding nothing.
#[test]
fn the_guard_fails_on_a_stale_skill() {
    let skills = scratch_dir("stale-skill");
    let dir = skills.join("stale");
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(
        dir.join("SKILL.md"),
        "# Stale\n\n\
         Skip `E-NO-SUCH-CODE`, read `montagent://gone.md`, run `montagent frobnicate <p>`,\n\
         then `montagent fonts unvendor`, then `measure --no-such-flag`, then call `retired_tool`.\n\n\
         A `word` opted out. <!-- guard-ok: word -->\n\n\
         ```json\n{\"frame\": {\"width\": 64, \"height\": 64}, \"fps\": 30, \"duration\": 1000, \
         \"output\": \"out/x.mp4\", \"tracks\": [{\"name\": \"t\", \"layer\": 0, \"elements\": \
         [{\"id\": \"i\", \"type\": \"image\", \"start\": 0, \"end\": 1000, \"source\": \"media/missing.png\"}]}]}\n```\n\n\
         ```json fragment\n{\"not\": \"a project\"}\n```\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("scripts/broken.py"),
        "\"\"\"Prints a broken project.\n\nRequires Python >= 3.9\ndrift-guard: media/bed.words.json\n\"\"\"\n\
         import sys\nif '--help' in sys.argv: print('Requires Python >= 3.9'); sys.exit(0)\n\
         print('{\"frame\": {\"width\": 64}, \"tracks\": []}')\n",
    )
    .unwrap();

    let names = check_names(&skills).join("\n");
    for stale in [
        "E-NO-SUCH-CODE",
        "montagent://gone.md",
        "frobnicate",
        "unvendor",
        "--no-such-flag",
        "retired_tool",
    ] {
        assert!(
            names.contains(stale),
            "the names check missed `{stale}`:\n{names}"
        );
    }
    assert!(
        !names.contains("`word`"),
        "an opt-out was ignored:\n{names}"
    );

    let snippets = check_snippets(&skills, "stale-snippets");
    assert_eq!(
        snippets.len(),
        1,
        "one broken snippet, one fragment:\n{snippets:?}"
    );

    let scripts = check_scripts(&skills, "stale-scripts");
    assert_eq!(scripts.len(), 1, "{scripts:?}");
    assert!(scripts[0].contains("does not validate"), "{scripts:?}");

    // The capability map, with an effect taken out of it and a refusal citing a
    // superseded ADR.
    let map = capability_map(&router_skill());
    let without_blur = map.replace("`blur`", "blur");
    let missing = check_completeness(&without_blur).join("\n");
    assert!(
        missing.contains("`blur`"),
        "the completeness check missed a removed effect:\n{missing}"
    );

    let adrs = scratch_dir("stale-adrs");
    std::fs::write(
        adrs.join("0001-old.md"),
        "---\nstatus: superseded by 0002\n---\n# Old\n",
    )
    .unwrap();
    let stale = check_refusals("### Not by design\n\n- A thing. [ADR-0001](x)\n", &adrs).join("\n");
    assert!(
        stale.contains("ADR-0001"),
        "the refusals check missed a superseded ADR:\n{stale}"
    );
    let gone = check_refusals("### Not by design\n\n- A thing. [ADR-0002](x)\n", &adrs).join("\n");
    assert!(
        gone.contains("ADR-0002"),
        "the refusals check missed a missing ADR:\n{gone}"
    );
}

// ---- The capability map ---------------------------------------------------------------

fn router_skill() -> String {
    std::fs::read_to_string(repo().join("skills/montagent/SKILL.md")).unwrap()
}

/// The capability map: the router skill's "What Montagent does" section, down to the next
/// `##` heading.
fn capability_map(skill: &str) -> String {
    let start = skill
        .find("## What Montagent does")
        .expect("the router skill has a capability map section");
    let rest = &skill[start + 3..];
    let end = rest.find("\n## ").map_or(rest.len(), |e| e + 1);
    rest[..end].to_string()
}

/// Every element type and every effect the schema index lists appears as a code span in the
/// map.
///
/// Plain fields (`blend`, `letter_spacing`) are not covered: the index lists no such thing,
/// so each spec that adds one carries its own acceptance line saying the map names it.
fn check_completeness(map: &str) -> Vec<String> {
    let index: serde_json::Value = serde_json::from_str(
        &montagent_core::resources::read("montagent://schema/index.json").unwrap(),
    )
    .unwrap();
    let spans: BTreeSet<String> = code_spans(map).into_iter().collect();
    let mut problems = Vec::new();
    for (section, kind) in [("elements", "element type"), ("effects", "effect")] {
        for name in index[section]
            .as_object()
            .expect("the index section")
            .keys()
        {
            if !spans.contains(name) {
                problems.push(format!("the {kind} `{name}`"));
            }
        }
    }
    problems
}

/// Every ADR a "Not by design" line cites exists and is still accepted, the way
/// `every_adr_the_format_docs_cite_exists_and_is_still_accepted` checks the format docs.
fn check_refusals(map: &str, adrs: &Path) -> Vec<String> {
    let Some(at) = map.find("Not by design") else {
        return vec!["the map has no \"Not by design\" list".to_string()];
    };
    let section = &map[at..];
    let mut cited: Vec<&str> = section
        .match_indices("ADR-")
        .filter_map(|(at, _)| section.get(at + 4..at + 8))
        .filter(|number| number.chars().all(|c| c.is_ascii_digit()))
        .collect();
    cited.sort();
    cited.dedup();
    let mut problems = Vec::new();
    if cited.is_empty() {
        problems.push("the \"Not by design\" list cites no ADR".to_string());
    }
    for number in cited {
        let file = std::fs::read_dir(adrs)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&format!("{number}-")))
            });
        let Some(file) = file else {
            problems.push(format!(
                "a refusal cites ADR-{number}, which does not exist"
            ));
            continue;
        };
        let body = std::fs::read_to_string(&file).unwrap();
        let status = body.lines().take(12).find_map(|line| {
            line.trim()
                .strip_prefix("status:")
                .map(|rest| rest.trim().to_lowercase())
        });
        if !status.as_deref().is_some_and(|s| s.starts_with("accepted")) {
            problems.push(format!(
                "a refusal cites ADR-{number}, whose status is {status:?}"
            ));
        }
    }
    problems
}

// ---- Names --------------------------------------------------------------------------

fn check_names(skills: &Path) -> Vec<String> {
    let allowed = residual_words(skills);
    let mut problems = Vec::new();
    for file in files(skills, "md") {
        let text = std::fs::read_to_string(&file).unwrap();
        for block in prose_blocks(&text) {
            let at = format!("{}:{}", rel(&file), block.line);
            for uri in uris(&block.text) {
                if montagent_core::resources::find(&uri).is_none() {
                    problems.push(format!("{at}: `{uri}` is not a published resource"));
                }
            }
            for span in code_spans(&block.text) {
                if block.opted_out.contains(&span) {
                    continue;
                }
                if let Some(problem) = judge_span(&span, &allowed) {
                    problems.push(format!("{at}: {problem}"));
                }
            }
        }
    }
    problems
}

fn judge_span(span: &str, allowed: &BTreeSet<String>) -> Option<String> {
    let tokens: Vec<&str> = span.split_whitespace().collect();
    let first = *tokens.first()?;
    if is_code(first) && tokens.len() == 1 {
        return montagent_core::registry::spec(first)
            .is_none()
            .then(|| format!("`{first}` is not a registered finding code"));
    }
    if first == "montagent" && tokens.len() > 1 {
        return judge_command(&tokens[1..]);
    }
    // `measure --at`: a verb and its flags.
    if tokens.len() > 1
        && cli().contains_key(&vec![first.to_string()])
        && tokens[1..]
            .iter()
            .all(|t| t.starts_with('-') || t.starts_with('<'))
    {
        return judge_command(&tokens);
    }
    let bare = tokens.len() == 1
        && first
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && first.chars().any(|c| c.is_ascii_lowercase());
    (bare && !allowed.contains(first)).then(|| {
        format!(
            "`{first}` is not an MCP tool, a CLI verb, a schema word, a snippet id or a skill \
             (opt out with <!-- guard-ok: {first} --> if it is meant)"
        )
    })
}

/// `montagent fonts vendor <p> --flag`: walk the subcommands, then check every flag
/// against the `--help` of the command they belong to.
fn judge_command(tokens: &[&str]) -> Option<String> {
    let mut path: Vec<String> = Vec::new();
    let mut rest = tokens;
    while let Some((&token, tail)) = rest.split_first() {
        let Some(node) = cli().get(&path) else { break };
        if token.starts_with('-') || token.starts_with('<') || node.subcommands.is_empty() {
            break;
        }
        if !node.subcommands.contains(token) {
            let parent = std::iter::once("montagent").chain(path.iter().map(String::as_str));
            return Some(format!(
                "`{}` has no subcommand `{token}`",
                parent.collect::<Vec<_>>().join(" ")
            ));
        }
        path.push(token.to_string());
        rest = tail;
    }
    let help = &cli().get(&path)?.help;
    rest.iter()
        .filter(|t| t.starts_with("--"))
        .map(|t| t.split('=').next().unwrap())
        .find(|flag| !help.lines().any(|line| declares_flag(line, flag)))
        .map(|flag| format!("`montagent {}` has no flag `{flag}`", path.join(" ")))
}

fn declares_flag(line: &str, flag: &str) -> bool {
    // `  -V, --version` or `      --at <MS>`: the flag is the line's first or second word.
    line.trim_start()
        .split([',', ' '])
        .take(3)
        .any(|word| word == flag)
}

/// Every word a bare code span may be: the residual rule's allowlist, all of it read off the
/// binary or the skills themselves.
fn residual_words(skills: &Path) -> BTreeSet<String> {
    let mut words: BTreeSet<String> = mcp_tools().clone();
    words.extend(cli().keys().filter_map(|path| path.last().cloned()));
    let schema = montagent_core::resources::read("montagent://schema.json").unwrap();
    schema_words(&serde_json::from_str(&schema).unwrap(), &mut words);
    for file in files(skills, "md") {
        for fence in fences(&std::fs::read_to_string(&file).unwrap()) {
            if let Ok(json) = serde_json::from_str(&fence.body) {
                declared_names(&json, &mut words);
            }
        }
    }
    if let Ok(dirs) = std::fs::read_dir(skills) {
        words.extend(
            dirs.flatten()
                .filter(|d| d.path().is_dir())
                .map(|d| d.file_name().to_string_lossy().into_owned()),
        );
    }
    words
}

fn schema_words(value: &serde_json::Value, words: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                // Every key: property and `$defs` names, and the schema's own keywords
                // (`properties`, `description`), which a skill may name.
                words.insert(key.clone());
                if key == "const" || key == "enum" {
                    for s in std::iter::once(child).chain(child.as_array().into_iter().flatten()) {
                        if let Some(s) = s.as_str() {
                            words.insert(s.to_string());
                        }
                    }
                }
                schema_words(child, words);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|v| schema_words(v, words)),
        _ => {}
    }
}

/// The ids, track names and font keys a snippet declares: the names prose points at.
fn declared_names(value: &serde_json::Value, words: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                match (key.as_str(), child) {
                    ("id" | "name", serde_json::Value::String(s)) => {
                        words.insert(s.clone());
                    }
                    ("fonts", serde_json::Value::Object(fonts)) => {
                        words.extend(fonts.keys().cloned())
                    }
                    _ => {}
                }
                declared_names(child, words);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|v| declared_names(v, words)),
        _ => {}
    }
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

// ---- Snippets -----------------------------------------------------------------------

fn check_snippets(skills: &Path, scratch: &str) -> Vec<String> {
    let dir = fixture_copy(scratch);
    let mut problems = Vec::new();
    for file in files(skills, "md") {
        for fence in fences(&std::fs::read_to_string(&file).unwrap()) {
            let mut info = fence.info.split_whitespace();
            if info.next() != Some("json") || info.any(|word| word == "fragment") {
                continue;
            }
            let at = format!("{}:{}", rel(&file), fence.line);
            let project = dir.join(format!(
                "{}.montagent.json",
                at.replace(['/', ':', '.'], "_")
            ));
            std::fs::write(&project, &fence.body).unwrap();
            if let Some(report) = validation_errors(&project) {
                problems.push(format!("{at}: the snippet does not validate\n{report}"));
            }
        }
    }
    problems
}

/// `None` when the project has no errors; otherwise the text report, for the failure.
fn validation_errors(project: &Path) -> Option<String> {
    let json = montagent(&["validate", project.to_str().unwrap(), "--json"]);
    let errors = serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .and_then(|v| v["summary"]["error"].as_u64());
    (errors != Some(0)).then(|| montagent(&["validate", project.to_str().unwrap()]))
}

// ---- Scripts ------------------------------------------------------------------------

fn check_scripts(skills: &Path, scratch: &str) -> Vec<String> {
    let dir = fixture_copy(scratch);
    let mut problems = Vec::new();
    for script in files(skills, "py")
        .into_iter()
        .filter(|p| p.parent().is_some_and(|d| d.ends_with("scripts")))
    {
        let at = rel(&script);
        let text = std::fs::read_to_string(&script).unwrap();
        let Some(version) = header(&text, "Requires Python >=") else {
            problems.push(format!(
                "{at}: no `Requires Python >= X.Y` line in its header"
            ));
            continue;
        };
        let Some(args) = header(&text, "drift-guard:") else {
            problems.push(format!(
                "{at}: no `drift-guard: <fixture args>` line in its header"
            ));
            continue;
        };
        let args: Vec<&str> = args.split_whitespace().collect();

        let help = python(&version, &script, &["--help"], &dir);
        if !help.stdout.contains(&version) {
            problems.push(format!(
                "{at}: `--help` does not state Python {version}\n{}",
                help.stderr
            ));
        }
        let run = python(&version, &script, &args, &dir);
        if run.code != Some(0) {
            problems.push(format!(
                "{at}: exited {:?} on Python {version}\n{}",
                run.code, run.stderr
            ));
            continue;
        }
        let project = dir.join(format!("{}.montagent.json", at.replace(['/', '.'], "_")));
        std::fs::write(&project, &run.stdout).unwrap();
        if let Some(report) = validation_errors(&project) {
            problems.push(format!("{at}: what it prints does not validate\n{report}"));
        }
    }
    problems
}

/// A header line `<label> <value>`, anywhere in the script's leading docstring or comments.
fn header(text: &str, label: &str) -> Option<String> {
    text.lines()
        .take(40)
        .find_map(|line| line.trim_start_matches(['#', ' ']).strip_prefix(label))
        .map(|value| value.trim().to_string())
}

/// The script on exactly the Python it claims, through `uv`. Without `uv` a developer's
/// machine falls back to `python3` and says so; CI has `uv`, and refuses to fall back.
fn python(version: &str, script: &Path, args: &[&str], cwd: &Path) -> Output {
    let uv = Command::new("uv")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    let mut command = if uv {
        let mut c = Command::new("uv");
        c.args(["run", "--no-project", "--python", version, "python"]);
        c
    } else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must run skill scripts on their minimum Python: install uv"
        );
        eprintln!(
            "uv not found: running {} on python3, not on Python {version}",
            script.display()
        );
        Command::new("python3")
    };
    // A script may call `montagent` (a typing script measures its letters): this build's.
    let path = std::env::join_paths(
        std::iter::once(binary().parent().unwrap().to_path_buf()).chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )),
    )
    .unwrap();
    let out = command
        .arg(script)
        .args(args)
        .env("PATH", path)
        .current_dir(cwd)
        .output()
        .expect("run a skill script");
    Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

// ---- Reading markdown ---------------------------------------------------------------

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

/// Prose split at blank lines, with fenced blocks left out.
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
    let mut fences = Vec::new();
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
            (Some(_), Some(_)) => fences.extend(open.take()),
            (Some(fence), None) => {
                fence.body.push_str(line);
                fence.body.push('\n');
            }
            (None, None) => {}
        }
    }
    fences
}

/// Inline code spans, with HTML comments stripped first so a marker's text is not prose.
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

// ---- The binary's own answers -------------------------------------------------------

struct CliNode {
    help: String,
    subcommands: BTreeSet<String>,
}

/// The clap tree, keyed by subcommand path, read from `--help` all the way down.
fn cli() -> &'static BTreeMap<Vec<String>, CliNode> {
    static TREE: OnceLock<BTreeMap<Vec<String>, CliNode>> = OnceLock::new();
    TREE.get_or_init(|| {
        let mut tree = BTreeMap::new();
        let mut pending = vec![Vec::<String>::new()];
        while let Some(path) = pending.pop() {
            let mut args: Vec<&str> = path.iter().map(String::as_str).collect();
            args.push("--help");
            let help = montagent(&args);
            let subcommands = commands_section(&help);
            for sub in subcommands.iter().filter(|s| *s != "help") {
                pending.push(path.iter().cloned().chain([sub.clone()]).collect());
            }
            tree.insert(path, CliNode { help, subcommands });
        }
        tree
    })
}

fn commands_section(help: &str) -> BTreeSet<String> {
    help.lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next().map(str::to_string))
        .collect()
}

fn mcp_tools() -> &'static BTreeSet<String> {
    static TOOLS: OnceLock<BTreeSet<String>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        let mut child = Command::new(binary())
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn the MCP server");
        let mut stdin = child.stdin.take().unwrap();
        for message in [
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2026-07-28", "capabilities": {},
                "clientInfo": {"name": "montagent-skills-guard", "version": "0"}}}),
            serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        ] {
            writeln!(stdin, "{message}").unwrap();
        }
        stdin.flush().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        let tools = reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(&line).ok())
            .find(|value| value["id"] == 2)
            .expect("an answer to tools/list");
        drop(stdin);
        let _ = child.kill();
        let _ = child.wait();
        tools["result"]["tools"]
            .as_array()
            .expect("a tool list")
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect()
    })
}

// ---- Plumbing -----------------------------------------------------------------------

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

/// Stdout of one `montagent` run, against a probe cache no other test shares.
fn montagent(args: &[&str]) -> String {
    static CACHE: OnceLock<PathBuf> = OnceLock::new();
    let cache = CACHE.get_or_init(|| scratch_dir(&format!("cache-{}", std::process::id())));
    let out = Command::new(binary())
        .args(args)
        .env(montagent_core::media::sidecar::CACHE_DIR_VAR, cache)
        .output()
        .expect("run montagent");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn rel(path: &Path) -> String {
    let path = path.strip_prefix(repo()).unwrap_or(path);
    path.to_string_lossy().replace('\\', "/")
}

fn files(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == extension) {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// A fresh copy of `fixtures/skills/`, so snippets and scripts resolve their relative paths
/// against it and nothing they write lands in the tree.
fn fixture_copy(name: &str) -> PathBuf {
    let dir = scratch_dir(name);
    copy_tree(&repo().join("fixtures/skills"), &dir);
    dir
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-skills-tests/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

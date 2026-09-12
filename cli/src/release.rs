//! Native `aps release` — new / status / notes / close (REL-005).
//!
//! Release records are **prose Markdown** (`plans/releases/v<version>.md`),
//! not a structured sidecar: APS is markdown-first, and the index constraint
//! "no runtime dependencies" rules out a JSON companion. Every command here
//! therefore reads the narrative itself — its header table, its
//! `## What Ships` module links and work-item mentions, and its ship-evidence
//! prose — using the shared document model in `parser`.
//!
//! `close` is the load-bearing one: after a cut ships, work items sitting in
//! the terminal-ish lifecycle states (`Merged` / `Released` / `Shipped`)
//! advance to `Complete` with a release-evidence line, and the module and
//! index rows that reflect them advance too. It is a dry run unless `--apply`
//! is passed, and both paths report from the same plan, so the preview and the
//! write can never disagree.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::date;
use crate::next::plan_roots;
use crate::orchestrate::{RewriteMode, rewrite_work_item};
use crate::parser::{self, PlanFile};

/// Fallback for `aps release new` when the project has no
/// `plans/releases/.release.template.md` of its own.
const EMBEDDED_TEMPLATE: &str = include_str!("../templates/release.template.md");

/// Lifecycle states that `close` advances. `parser::normalize_status` reports
/// these as `Unknown` by design — they are terminal *compaction* markers
/// (plans/aps-rules.md § Status Vocabulary), not canonical states — so the
/// release lifecycle recognises them here and nowhere else.
const RELEASE_STATES: [&str; 3] = ["merged", "released", "shipped"];

/// Record `Status` values that mean the cut is out the door, so advancing
/// work items against it is safe.
const SHIPPED_STATES: [&str; 3] = ["shipped", "released", "archived"];

// --- Versions ----------------------------------------------------------------

/// `v0.5.0` and `0.5.0` name the same release.
pub fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed)
        .to_string()
}

/// Ordering key for a version string: numeric components first, then a
/// pre-release marker (`1.0.0-beta` sorts *before* `1.0.0`), then the
/// pre-release text.
fn version_key(version: &str) -> (Vec<u64>, u8, String) {
    let (core, pre) = match version.find(['-', '+']) {
        Some(index) => (&version[..index], &version[index + 1..]),
        None => (version, ""),
    };
    let numbers = core
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect();
    (numbers, u8::from(pre.is_empty()), pre.to_ascii_lowercase())
}

fn releases_dir(plans: &Path) -> PathBuf {
    plans.join("releases")
}

fn record_path(plans: &Path, version: &str) -> PathBuf {
    releases_dir(plans).join(format!("v{version}.md"))
}

/// Every release record under `<plans>/releases/`, ascending by version.
/// Mirrors the lint notion of a release filename: `v<digit>….md`.
fn list_records(plans: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let Ok(entries) = fs::read_dir(releases_dir(plans)) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        let Some(rest) = stem.strip_prefix('v') else {
            continue;
        };
        if !rest.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        out.push((rest.to_string(), entry.path()));
    }
    out.sort_by_key(|left| version_key(&left.0));
    out
}

// --- Markdown helpers --------------------------------------------------------

/// A table row that carries no data — only `|`, `:`, `-` and spaces.
fn is_separator_row(line: &str) -> bool {
    line.starts_with('|') && line.chars().all(|c| matches!(c, '|' | ':' | '-' | ' '))
}

/// Two-column header-table lookup: `| Target | v0.5.0 |` → `v0.5.0`. Scans
/// the whole document so a record with a long guidance comment still resolves.
fn header_field(plan: &PlanFile, name: &str) -> String {
    for line in &plan.lines {
        if !line.starts_with('|') || is_separator_row(line) {
            continue;
        }
        let mut cells = line.split('|').skip(1);
        let Some(label) = cells.next() else {
            continue;
        };
        if label.trim() != name {
            continue;
        }
        if let Some(value) = cells.next() {
            return value.trim().to_string();
        }
    }
    String::new()
}

/// `split('|')` index of the cell whose text is `name`, for a table header row.
fn header_column(line: &str, name: &str) -> Option<usize> {
    if !line.starts_with('|') || is_separator_row(line) {
        return None;
    }
    line.split('|').position(|cell| cell.trim() == name)
}

/// Replace cell `col` (a `split('|')` index) of a markdown table row,
/// preserving the cell's original width when the new value is shorter so the
/// surrounding table stays aligned and the row keeps its column count.
fn replace_cell(line: &str, col: usize, value: &str) -> String {
    let parts: Vec<&str> = line.split('|').collect();
    if col == 0 || col >= parts.len() {
        return line.to_string();
    }
    let width = parts[col].chars().count();
    let mut cell = format!(" {value} ");
    let len = cell.chars().count();
    if len < width {
        cell.push_str(&" ".repeat(width - len));
    }
    let mut out = String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        if index == col {
            out.push_str(&cell);
        } else {
            out.push_str(part);
        }
    }
    out
}

/// Blank-line-separated blocks with their lines joined by a space. Markdown
/// prose is hard-wrapped, so evidence like ``tag `v0.5.0` on `408f8cf``` can
/// straddle a line break; blocks make the scan wrap-insensitive.
fn prose_blocks(lines: &[String]) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        current.push(line.trim());
    }
    if !current.is_empty() {
        blocks.push(current.join(" "));
    }
    blocks
}

/// Backtick-quoted spans of `text`, in order.
fn backticked(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let start = index + 1;
        match text[start..].find('`') {
            Some(offset) => {
                out.push(&text[start..start + offset]);
                index = start + offset + 1;
            }
            None => break,
        }
    }
    out
}

/// A git object name: 7–40 hex digits.
fn looks_like_sha(token: &str) -> bool {
    (7..=40).contains(&token.len()) && token.chars().all(|c| c.is_ascii_hexdigit())
}

/// Every `YYYY-MM-DD` in `text`, in order.
fn dates_in(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index + 10 <= chars.len() {
        let window: String = chars[index..index + 10].iter().collect();
        if date::parse_civil_date(&window).is_some() {
            out.push(window);
            index += 10;
        } else {
            index += 1;
        }
    }
    out
}

/// Each date in a `Date` row paired with the lower-cased `(…)` label that
/// follows it, or an empty label when the date is unlabelled. The label runs
/// to the next date, so `2026-07-28 (cut); 2026-07-29 (shipped)` yields
/// `[("2026-07-28", "cut"), ("2026-07-29", "shipped")]`.
///
/// The labels actually in use across this repo's records are `planning`,
/// `cut`, `shipped`, the combined `planning + shipped`, and none at all
/// (v0.3.0). Anything else is left unrecognised on purpose: reporting no date
/// is always better than reporting a wrong one.
fn labelled_dates(row: &str) -> Vec<(String, String)> {
    let dates = dates_in(row);
    let mut spans: Vec<(usize, String)> = Vec::new();
    let mut cursor = 0;
    for date in dates {
        if let Some(offset) = row[cursor..].find(&date) {
            let at = cursor + offset;
            cursor = at + date.len();
            spans.push((at + date.len(), date));
        }
    }
    let mut out = Vec::new();
    for (index, (after, date)) in spans.iter().enumerate() {
        let end = spans
            .get(index + 1)
            .map(|(_, next)| row.find(next.as_str()).unwrap_or(row.len()))
            .unwrap_or(row.len());
        let window = &row[(*after).min(end)..end.max(*after)];
        let label = match (window.find('('), window.find(')')) {
            (Some(open), Some(close)) if close > open => {
                window[open + 1..close].trim().to_lowercase()
            }
            _ => String::new(),
        };
        out.push((date.clone(), label));
    }
    out
}

/// The first date whose label mentions `keyword`.
fn date_labelled(dates: &[(String, String)], keyword: &str) -> Option<String> {
    dates
        .iter()
        .find(|(_, label)| label.contains(keyword))
        .map(|(date, _)| date.clone())
}

// --- The prose release record ------------------------------------------------

/// A release narrative, read as prose. Every field is best-effort: a record is
/// hand-written Markdown, so a missing row degrades to a warning and a
/// documented fallback rather than a parse failure.
#[derive(Debug, Default)]
pub struct ReleaseRecord {
    pub path: String,
    /// Version from the `Target` row, falling back to the filename.
    pub version: String,
    /// `Status` row verbatim (`Shipped`, `Planning`, …).
    pub status: String,
    /// `Previous release` row, version only (`v0.4.0 (2026-06-22)` → `0.4.0`).
    pub previous: String,
    pub previous_date: String,
    /// `Date` row entry labelled `(planning)`, empty when absent.
    pub planning_date: String,
    /// `Date` row entry labelled `(cut)`, empty when absent.
    pub cut_date: String,
    /// Shipped date: the `Date` row entry labelled `(shipped)` (or
    /// `(planning + shipped)`), else a `Ship evidence (…)` date, else — only
    /// for a terminal record whose `Date` row carries no labels at all — the
    /// last date in that row. Empty otherwise: an unshipped release has no
    /// ship date, and inventing one asserts something false.
    pub ship_date: String,
    /// Tag commit from the ship-evidence prose.
    pub tag_commit: String,
    /// Module plan files linked from `## What Ships` (resolved, existing).
    pub module_links: Vec<String>,
    /// Bare module names from the `**APS module(s):**` lines in
    /// `## What Ships`. Records in the wild write those references both as
    /// links and as plain text, so both are collected.
    pub module_refs: Vec<String>,
    /// Work-item IDs named in `## What Ships`, ranges expanded.
    pub named_items: Vec<String>,
    /// Non-fatal observations about the record itself.
    pub warnings: Vec<String>,
}

/// Read a release record. Fails only when the file cannot be read or carries
/// no recognisable release header table (lint R002).
pub fn parse_record(path: &Path) -> Result<ReleaseRecord, String> {
    let path_str = path.to_string_lossy().into_owned();
    let plan = PlanFile::load(&path_str)
        .map_err(|err| format!("cannot read release record {path_str}: {err}"))?;

    let mut record = ReleaseRecord {
        path: path_str.clone(),
        ..ReleaseRecord::default()
    };

    let target = header_field(&plan, "Target");
    record.status = header_field(&plan, "Status");
    if target.is_empty() && record.status.is_empty() {
        return Err(format!(
            "{path_str}: no release header table with Target and Status rows (lint R002) — cannot read this record as a release narrative"
        ));
    }
    if target.is_empty() {
        record.warnings.push(
            "header table has no Target row (lint R002); using the filename for the version"
                .to_string(),
        );
    }
    if record.status.is_empty() {
        record
            .warnings
            .push("header table has no Status row (lint R002)".to_string());
    }

    let file_version = path
        .file_stem()
        .map(|stem| normalize_version(&stem.to_string_lossy()))
        .unwrap_or_default();
    record.version = if target.is_empty() {
        file_version.clone()
    } else {
        normalize_version(&target)
    };
    if !target.is_empty() && !file_version.is_empty() && record.version != file_version {
        record.warnings.push(format!(
            "header table Target is v{} but the filename says v{file_version}",
            record.version
        ));
    }

    let previous = header_field(&plan, "Previous release");
    record.previous = normalize_version(previous.split_whitespace().next().unwrap_or(""));
    record.previous_date = dates_in(&previous).first().cloned().unwrap_or_default();

    let date_row = header_field(&plan, "Date");
    let dates = labelled_dates(&date_row);
    let unlabelled = !dates.is_empty() && dates.iter().all(|(_, label)| label.is_empty());
    let terminal = SHIPPED_STATES
        .iter()
        .any(|state| record.status.to_lowercase().starts_with(state));

    record.planning_date = date_labelled(&dates, "planning")
        .or_else(|| unlabelled.then(|| dates[0].0.clone()))
        .unwrap_or_default();
    record.cut_date = date_labelled(&dates, "cut").unwrap_or_default();
    // A `shipped` label is authoritative. Failing that, an entirely unlabelled
    // Date row on a terminal record (v0.3.0's bare `2026-05-20`) is a ship
    // date. A labelled row that never says shipped — v0.4.0's
    // `(planning), (cut)`, v0.9.0's `(planning)` — has none, and says so.
    record.ship_date = date_labelled(&dates, "ship")
        .or_else(|| date_labelled(&dates, "released"))
        .or_else(|| (unlabelled && terminal).then(|| dates[dates.len() - 1].0.clone()))
        .unwrap_or_default();

    let blocks = prose_blocks(&plan.lines);
    if let Some(evidence) = blocks
        .iter()
        .find(|block| block.to_lowercase().starts_with("ship evidence"))
        && let Some(date) = dates_in(evidence).first()
    {
        record.ship_date = date.clone();
    }
    if record.ship_date.is_empty() && terminal && !dates.is_empty() {
        record.warnings.push(format!(
            "Status is '{}' but the Date row labels no date as shipped ({date_row:?}) — `close` needs --date",
            record.status
        ));
    }
    record.tag_commit = find_tag_commit(&blocks, &record.version);

    // `## What Ships` is the template's designated inventory of what the cut
    // delivers, so it — and not the whole document — scopes the closeout.
    // Risks, Out of Scope and Related deliberately name work that is *not*
    // shipping. A record with no such section falls back to the whole file.
    let mut ships: Vec<&str> = plan.section_content("## What Ships");
    if ships.is_empty() {
        record.warnings.push(
            "no ## What Ships section (lint R004); scoping from the whole record instead"
                .to_string(),
        );
        ships = plan.lines.iter().map(String::as_str).collect();
    }
    let base = path.parent().unwrap_or(Path::new("."));
    record.module_links = module_links(&ships, base);
    record.module_refs = module_references(&ships);
    record.named_items = named_item_ids(&ships.join("\n"));

    // Documented fallback: when `## What Ships` names no modules at all, the
    // `**Module specs:**` bullet under `## Related` carries the same links in
    // most records. Only reached when the primary scope is empty, and the
    // caller warns that it was used.
    if record.module_links.is_empty() && record.module_refs.is_empty() {
        let related = plan.section_content("## Related");
        let specs: Vec<&str> = related
            .iter()
            .copied()
            .filter(|line| line.contains("**Module specs:**"))
            .chain(continuation_lines(&related, "**Module specs:**"))
            .collect();
        record.module_links = module_links(&specs, base);
        if !record.module_links.is_empty() {
            record.warnings.push(
                "## What Ships names no modules; scoped from the **Module specs:** list under ## Related instead"
                    .to_string(),
            );
        }
    }

    Ok(record)
}

/// The indented continuation lines of a wrapped `- **Field:** …` bullet.
fn continuation_lines<'a>(lines: &[&'a str], marker: &str) -> Vec<&'a str> {
    let Some(start) = lines.iter().position(|line| line.contains(marker)) else {
        return Vec::new();
    };
    lines[start + 1..]
        .iter()
        .take_while(|line| {
            let trimmed = line.trim_start();
            !trimmed.is_empty() && trimmed.len() != line.len() && !trimmed.starts_with("- **")
        })
        .copied()
        .collect()
}

/// Bare module names from the `**APS module:**` / `**APS modules:**` lines the
/// template puts under each `## What Ships` area. Records write these both as
/// markdown links (picked up by `module_links`) and as plain text, e.g.
/// `**APS modules:** cli-redesign (6/7 work items done — CLI-002 remains)`.
///
/// Parenthesised asides and link targets are dropped, then every remaining
/// lowercase word is offered as a candidate. Resolution against the plan tree
/// is what actually filters them, so prose like `Closes` or `Satisfies` simply
/// never matches and needs no grammar of its own.
fn module_references(ships: &[&str]) -> Vec<String> {
    let owned: Vec<String> = ships.iter().map(|line| (*line).to_string()).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::new();
    for block in prose_blocks(&owned) {
        let lower = block.to_lowercase();
        let Some(at) = lower.find("**aps module") else {
            continue;
        };
        let Some(colon) = block[at..].find(":**") else {
            continue;
        };
        let text = &block[at + colon + 3..];

        // Drop `(…)` asides and rewrite `[label](target)` down to `label`.
        let mut flat = String::new();
        let mut depth = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ']' => {
                    // Skip the `](target)` that follows a link label.
                    if chars.peek() == Some(&'(') {
                        for inner in chars.by_ref() {
                            if inner == ')' {
                                break;
                            }
                        }
                    }
                }
                '[' => {}
                _ if depth == 0 => flat.push(c),
                _ => {}
            }
        }

        for token in flat.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
            let token = token.trim_matches('-');
            if token.len() < 3
                || token.chars().any(|c| c.is_ascii_uppercase())
                || !token.starts_with(|c: char| c.is_ascii_lowercase())
            {
                continue;
            }
            if seen.insert(token.to_string()) {
                out.push(token.to_string());
            }
        }
    }
    out
}

/// Tag commit from ship-evidence prose. Prefers the conventional
/// ``tag `v<version>` on `<sha>``` phrasing, then any block that mentions a
/// tag and carries a hex object name.
fn find_tag_commit(blocks: &[String], version: &str) -> String {
    let anchor = format!("tag `v{version}`");
    for block in blocks {
        if let Some(at) = block.find(&anchor)
            && let Some(sha) = backticked(&block[at + anchor.len()..])
                .into_iter()
                .find(|token| looks_like_sha(token))
        {
            return sha.to_string();
        }
    }
    for block in blocks {
        let lower = block.to_lowercase();
        let Some(at) = lower.find("tag") else {
            continue;
        };
        if let Some(sha) = backticked(&block[at..])
            .into_iter()
            .find(|token| looks_like_sha(token))
        {
            return sha.to_string();
        }
    }
    String::new()
}

/// Markdown link targets ending in `.aps.md`, resolved against `base` and
/// lexically normalised. Order-preserving and deduped; does not touch disk.
fn plan_link_targets(lines: &[&str], base: &Path) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::new();
    for line in lines {
        let mut rest = *line;
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            let Some(close) = after.find(')') else {
                break;
            };
            let target = after[..close].split('#').next().unwrap_or("");
            rest = &after[close + 1..];
            if !target.ends_with(".aps.md") {
                continue;
            }
            let resolved = parser::normalize_path(&format!("{}/{target}", base.to_string_lossy()));
            if seen.insert(resolved.clone()) {
                out.push(resolved);
            }
        }
    }
    out
}

/// `plan_link_targets` filtered to links that name a file that really exists —
/// what a release record's module references have to clear.
fn module_links(lines: &[&str], base: &Path) -> Vec<String> {
    plan_link_targets(lines, base)
        .into_iter()
        .filter(|link| Path::new(link).is_file())
        .collect()
}

/// Work-item IDs named in prose, with the collapsed forms release narratives
/// actually use expanded: `MONO-007/008` → both items, `MONO-001…006` → the
/// whole run. The prefix must be two or more uppercase letters, which keeps
/// decision refs (`D-039`) out, and the number two to four digits, which keeps
/// prose like `UTF-8` out. Prefixes unknown to the plan tree are filtered by
/// the caller, so an incidental `ISO-8601` never becomes a finding.
fn named_item_ids(text: &str) -> Vec<String> {
    const RANGE_MARKS: [&str; 5] = ["…", "...", "--", "–", "—"];
    let chars: Vec<char> = text.chars().collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::new();
    let mut index = 0;

    fn push(id: String, out: &mut Vec<String>, seen: &mut BTreeSet<String>) {
        if seen.insert(id.clone()) {
            out.push(id);
        }
    }

    while index < chars.len() {
        if !chars[index].is_ascii_uppercase() {
            index += 1;
            continue;
        }
        // A letter run that continues past uppercase (`Mixed`) is prose.
        let letters_start = index;
        while index < chars.len() && chars[index].is_ascii_uppercase() {
            index += 1;
        }
        let letters: String = chars[letters_start..index].iter().collect();
        if letters.len() < 2 || index >= chars.len() || chars[index] != '-' {
            continue;
        }
        let digits_start = index + 1;
        let mut cursor = digits_start;
        while cursor < chars.len() && chars[cursor].is_ascii_digit() {
            cursor += 1;
        }
        let width = cursor - digits_start;
        if !(2..=4).contains(&width) {
            index = cursor.max(index + 1);
            continue;
        }
        let first: String = chars[digits_start..cursor].iter().collect();
        push(format!("{letters}-{first}"), &mut out, &mut seen);
        index = cursor;

        // `/008` repetitions share the prefix and the digit width.
        while index < chars.len() && chars[index] == '/' {
            let mut peek = index + 1;
            while peek < chars.len() && chars[peek].is_ascii_digit() {
                peek += 1;
            }
            if peek == index + 1 {
                break;
            }
            let number: String = chars[index + 1..peek].iter().collect();
            index = peek;
            let Ok(number) = number.parse::<u32>() else {
                break;
            };
            push(format!("{letters}-{number:0width$}"), &mut out, &mut seen);
        }

        // `…006` / `...006` / `--006` closes a run from the first ID.
        let tail: String = chars[index..].iter().collect();
        let Some(mark) = RANGE_MARKS.iter().find(|mark| tail.starts_with(**mark)) else {
            continue;
        };
        let after = index + mark.chars().count();
        let mut peek = after;
        while peek < chars.len() && chars[peek].is_ascii_digit() {
            peek += 1;
        }
        if peek == after {
            continue;
        }
        let last: String = chars[after..peek].iter().collect();
        index = peek;
        let (Ok(from), Ok(to)) = (first.parse::<u32>(), last.parse::<u32>()) else {
            continue;
        };
        // A sane run, not a typo that would enumerate thousands of IDs.
        if to <= from || to - from > 200 {
            continue;
        }
        for number in from + 1..=to {
            push(format!("{letters}-{number:0width$}"), &mut out, &mut seen);
        }
    }
    out
}

// --- The plan tree -----------------------------------------------------------

/// Lifecycle bucket for a raw status string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Life {
    /// `Merged` / `Released` / `Shipped` — awaiting release closeout.
    Releasable,
    Complete,
    Open,
}

/// True when `text` begins with `word` as a whole word.
fn starts_with_word(text: &str, word: &str) -> bool {
    text.starts_with(word)
        && !text[word.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// Bucket a raw status. The canonical vocabulary and its documented aliases
/// (`Proposed` → `Draft`, `Done` → `Complete`) come from
/// `parser::normalize_status`; the release lifecycle states it reports as
/// `Unknown` are recognised here.
fn lifecycle(raw: &str) -> Life {
    let letters: String = raw
        .chars()
        .skip_while(|c| !c.is_ascii_alphabetic())
        .collect();
    let lower = letters.to_lowercase();
    if RELEASE_STATES
        .iter()
        .any(|state| starts_with_word(&lower, state))
    {
        return Life::Releasable;
    }
    if parser::normalize_status(&letters, "") == "Complete" {
        return Life::Complete;
    }
    Life::Open
}

/// A terminal suffix on a work-item header (`### X-001: Title — Merged`),
/// used only when the item carries no `- **Status:**` field. Mirrors lint's
/// `header_has_terminal_suffix`.
fn header_terminal_suffix(header: &str) -> String {
    let mut search = header;
    let mut best = String::new();
    loop {
        let (at, dash) = match (search.find('—'), search.find("--")) {
            (Some(em), Some(dd)) if em <= dd => (em, '—'.len_utf8()),
            (Some(em), None) => (em, '—'.len_utf8()),
            (_, Some(dd)) => (dd, 2),
            (None, None) => return best,
        };
        let rest = search[at + dash..].trim_start_matches(' ');
        if lifecycle(rest) != Life::Open {
            best = rest.to_string();
        }
        search = &search[at + dash..];
    }
}

#[derive(Debug, Clone)]
struct ModuleInfo {
    id: String,
    file: String,
    index_file: Option<String>,
    raw_status: String,
}

#[derive(Debug, Clone)]
struct TreeItem {
    id: String,
    title: String,
    file: String,
    raw_status: String,
    module: String,
}

#[derive(Debug, Default)]
struct Tree {
    modules: Vec<ModuleInfo>,
    items: Vec<TreeItem>,
}

impl Tree {
    fn module_by_file(&self, file: &str) -> Option<&ModuleInfo> {
        self.modules.iter().find(|module| module.file == file)
    }

    fn items_in(&self, file: &str) -> impl Iterator<Item = &TreeItem> {
        self.items.iter().filter(move |item| item.file == file)
    }

    fn prefixes(&self) -> BTreeSet<String> {
        self.items
            .iter()
            .filter_map(|item| {
                item.id
                    .split_once('-')
                    .map(|(prefix, _)| prefix.to_string())
            })
            .collect()
    }

    /// Resolve a bare module name written in a record: the module file's stem
    /// (`monorepo` → `monorepo.aps.md`), its metadata ID (`MONO`), or — for
    /// the abbreviations records actually use — a *unique* stem whose name
    /// continues at a `-` boundary, so `continuous-improvement` finds
    /// `continuous-improvement-backlog.aps.md`. Ambiguity resolves to nothing.
    fn module_by_name(&self, name: &str) -> Option<&ModuleInfo> {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return None;
        }
        if let Some(exact) = self
            .modules
            .iter()
            .find(|module| module_label(&module.file).to_lowercase() == name)
        {
            return Some(exact);
        }
        if let Some(by_id) = self
            .modules
            .iter()
            .find(|module| module.id.to_lowercase() == name)
        {
            return Some(by_id);
        }
        if name.len() < 4 {
            return None;
        }
        let boundary = format!("{name}-");
        let mut matches = self.modules.iter().filter(|module| {
            module_label(&module.file)
                .to_lowercase()
                .starts_with(&boundary)
        });
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }
}

/// What a release record says shipped, resolved against the plan tree. Shared
/// by `status`, `notes` and `close` so the three commands can never disagree
/// about a release's scope.
#[derive(Debug, Default)]
struct Scope {
    /// Module plan files the record says shipped, in record order.
    module_files: Vec<String>,
    /// Every work item in those modules, plus any item the record names
    /// directly. Deduped, plan-tree order.
    items: Vec<TreeItem>,
    /// IDs the record names whose prefix belongs to a module in this tree but
    /// whose item does not exist.
    unresolved: Vec<String>,
    warnings: Vec<String>,
}

fn build_scope(record: &ReleaseRecord, tree: &Tree) -> Scope {
    let mut scope = Scope::default();
    let mut files: BTreeSet<String> = BTreeSet::new();

    for link in &record.module_links {
        if tree.module_by_file(link).is_some() {
            if files.insert(link.clone()) {
                scope.module_files.push(link.clone());
            }
        } else {
            scope.warnings.push(format!(
                "{link} is linked as a module but is not a module plan in this tree"
            ));
        }
    }
    for name in &record.module_refs {
        if let Some(module) = tree.module_by_name(name)
            && files.insert(module.file.clone())
        {
            scope.module_files.push(module.file.clone());
        }
    }

    let prefixes = tree.prefixes();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for file in &scope.module_files {
        for item in tree.items_in(file) {
            if seen.insert(item.id.clone()) {
                scope.items.push(item.clone());
            }
        }
    }
    for id in &record.named_items {
        // A prefix unknown to the tree is prose, not a work item, so an
        // incidental `ISO-8601` never becomes a finding.
        if !id
            .split_once('-')
            .is_some_and(|(prefix, _)| prefixes.contains(prefix))
        {
            continue;
        }
        match tree.items.iter().find(|item| &item.id == id) {
            Some(item) => {
                if seen.insert(item.id.clone()) {
                    scope.items.push(item.clone());
                }
            }
            None => scope.unresolved.push(id.clone()),
        }
    }
    scope
}

/// The hint every command gives when a record maps to nothing.
const NO_SCOPE_HINT: &str = "add a markdown link to ../modules/<name>.aps.md (or an `**APS modules:** <name>` line) under ## What Ships";

/// Load every module file and work item in the plan tree, keeping **raw**
/// status text. `aps next`'s `PlanGraph` normalises status, which folds the
/// release lifecycle states into `Unknown` — exactly the information the
/// closeout needs — so this walks the same roots (`next::plan_roots`, so a
/// federated tree is covered) with the same `parser` primitives instead.
fn load_tree(plans: &Path) -> Tree {
    let mut tree = Tree::default();
    for root in plan_roots(plans) {
        let index = root.join("index.aps.md");
        let index_file = index
            .is_file()
            .then(|| parser::normalize_path(&index.to_string_lossy()));
        let module_dir = root.join("modules");
        if !module_dir.is_dir() {
            continue;
        }
        for file in parser::find_aps_files(&module_dir) {
            if !file.ends_with(".aps.md") {
                continue;
            }
            let Ok(plan) = PlanFile::load(&file) else {
                continue;
            };
            let file = parser::normalize_path(&file);
            let module_id = plan.module_id().unwrap_or_else(|| {
                Path::new(&file)
                    .file_stem()
                    .map(|stem| {
                        stem.to_string_lossy()
                            .trim_end_matches(".aps")
                            .to_uppercase()
                    })
                    .unwrap_or_default()
            });
            tree.modules.push(ModuleInfo {
                id: module_id.clone(),
                file: file.clone(),
                index_file: index_file.clone(),
                raw_status: plan.status().unwrap_or_default(),
            });
            for header in plan.work_items() {
                let Some(id) = parser::parse_work_item_id(&header.header) else {
                    continue;
                };
                let content = plan.item_content(header.line);
                let mut raw_status = parser::field_value(&content, "Status");
                if raw_status.is_empty() {
                    raw_status = header_terminal_suffix(&header.header);
                }
                tree.items.push(TreeItem {
                    id: id.to_string(),
                    title: parser::work_item_title(&header.header),
                    file: file.clone(),
                    raw_status,
                    module: module_id.clone(),
                });
            }
        }
    }
    tree
}

// --- Table cell locators -----------------------------------------------------

/// The metadata-table `Status` cell of a module plan: `(line index,
/// split index, current value)`. Matches `PlanFile::status()` — the first
/// table whose header row names both `ID` and `Status`.
fn module_status_cell(plan: &PlanFile) -> Option<(usize, usize, String)> {
    let mut column = None;
    for (index, line) in plan.lines.iter().enumerate() {
        match column {
            None => {
                if header_column(line, "ID").is_some()
                    && let Some(col) = header_column(line, "Status")
                {
                    column = Some(col);
                }
            }
            Some(col) => {
                if !line.starts_with('|') || is_separator_row(line) {
                    continue;
                }
                let value = line.split('|').nth(col).unwrap_or("").trim().to_string();
                return Some((index, col, value));
            }
        }
    }
    None
}

/// The `Status` cell of the index row that links to `module_file`. Index
/// tables are `| Module | Purpose | Status |`, but the column is located by
/// name from the nearest preceding header row so a different shape still works.
fn index_status_cell(plan: &PlanFile, module_file: &str) -> Option<(usize, usize, String)> {
    let base = Path::new(&plan.path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut column = None;
    for (index, line) in plan.lines.iter().enumerate() {
        if let Some(col) = header_column(line, "Status") {
            column = Some(col);
            continue;
        }
        if !line.starts_with('|') || is_separator_row(line) {
            continue;
        }
        let links = plan_link_targets(&[line.as_str()], &base);
        if !links.iter().any(|link| link == module_file) {
            continue;
        }
        let col = column?;
        let value = line.split('|').nth(col).unwrap_or("").trim().to_string();
        return Some((index, col, value));
    }
    None
}

// --- Work-item mutation ------------------------------------------------------

/// Insert or replace a `- **Field:** value` line immediately after a work
/// item's `- **Status:**` line (and any indented continuation of it). Status
/// itself is rewritten through `orchestrate::rewrite_work_item`, so the two
/// edits share the plan tree's one canonical notion of an item block.
fn set_item_field(path: &str, id: &str, field: &str, value: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|err| format!("cannot read {path}: {err}"))?;
    let lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    let header = format!("### {id}:");
    let field_prefix = format!("- **{field}:**");
    let new_line = format!("- **{field}:** {value}");

    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 1);
    let mut in_target = false;
    let mut done = false;
    let mut pending = false;

    for line in lines {
        if pending {
            // Skip over indented continuation of the Status line, then insert.
            let trimmed = line.trim_start();
            if !trimmed.is_empty() && trimmed.len() != line.len() {
                out.push(line);
                continue;
            }
            out.push(new_line.clone());
            pending = false;
            done = true;
        }
        if line.starts_with("### ") {
            in_target = line.starts_with(&header);
            out.push(line);
            continue;
        }
        if in_target && line.starts_with("## ") {
            in_target = false;
            out.push(line);
            continue;
        }
        if in_target && !done {
            if line.starts_with(&field_prefix) {
                out.push(new_line.clone());
                done = true;
                continue;
            }
            if line.starts_with("- **Status:**") {
                out.push(line);
                pending = true;
                continue;
            }
        }
        out.push(line);
    }
    if pending {
        out.push(new_line.clone());
        done = true;
    }
    if !done {
        return Err(format!(
            "{id}: no **Status:** line to anchor **{field}:** in {path}"
        ));
    }
    fs::write(path, out.join("\n")).map_err(|err| format!("cannot write {path}: {err}"))
}

/// Replace a located table cell in a file, re-reading it so the edit is
/// applied to whatever the earlier work-item rewrites left behind.
fn set_table_cell(path: &str, line_index: usize, col: usize, value: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|err| format!("cannot read {path}: {err}"))?;
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    let line = lines
        .get_mut(line_index)
        .ok_or_else(|| format!("{path}: line {} vanished before the write", line_index + 1))?;
    *line = replace_cell(line, col, value);
    fs::write(path, lines.join("\n")).map_err(|err| format!("cannot write {path}: {err}"))
}

// --- `aps release close` plan ------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advance {
    pub id: String,
    pub module: String,
    pub file: String,
    pub from: String,
    pub to: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowChange {
    /// `module` or `index`.
    pub kind: &'static str,
    pub module: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skip {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct ClosePlan {
    pub version: String,
    pub record: String,
    pub record_status: String,
    pub date: String,
    pub tag: String,
    pub advances: Vec<Advance>,
    pub rows: Vec<RowChange>,
    pub skips: Vec<Skip>,
    pub unresolved: Vec<String>,
    pub warnings: Vec<String>,
}

/// The evidence line stamped beside an advanced item. Deterministic, so the
/// bash and PowerShell ports can reproduce it byte for byte.
fn evidence_line(version: &str, date: &str, tag: &str, from: &str) -> String {
    let mut out = format!("v{version} ({date}");
    if !tag.is_empty() {
        out.push_str(&format!(", tag `{tag}`"));
    }
    out.push(')');
    if !from.is_empty() {
        out.push_str(&format!(" — was {from}"));
    }
    out
}

/// Work out everything `close` would do. Pure: reads the record and the plan
/// tree, writes nothing. The dry run and `--apply` both report from this, so a
/// preview can never disagree with the write.
pub fn plan_close(
    plans: &Path,
    version: &str,
    tag_override: Option<&str>,
    date_override: Option<&str>,
) -> Result<ClosePlan, String> {
    let version = normalize_version(version);
    let path = record_path(plans, &version);
    if !path.is_file() {
        return Err(format!(
            "no release record at {} — run `aps release new {version}` first",
            path.display()
        ));
    }
    let record = parse_record(&path)?;
    let tree = load_tree(plans);

    let mut plan = ClosePlan {
        version: record.version.clone(),
        record: record.path.clone(),
        record_status: record.status.clone(),
        tag: tag_override.unwrap_or(&record.tag_commit).to_string(),
        warnings: record.warnings.clone(),
        ..ClosePlan::default()
    };

    plan.date = match date_override {
        Some(date) => {
            if date::parse_civil_date(date).is_none() {
                return Err(format!("--date {date} is not a YYYY-MM-DD civil date"));
            }
            date.to_string()
        }
        None if !record.ship_date.is_empty() => record.ship_date.clone(),
        None => {
            let today = date::today_utc_ymd();
            plan.warnings.push(format!(
                "record carries no shipped date; stamping today ({today}) — override with --date"
            ));
            today
        }
    };
    if plan.tag.is_empty() {
        plan.warnings.push(
            "record carries no tag commit in its ship evidence; evidence will omit it — override with --tag"
                .to_string(),
        );
    }

    let scope = build_scope(&record, &tree);
    plan.warnings.extend(scope.warnings.iter().cloned());
    plan.unresolved = scope.unresolved.clone();

    if scope.items.is_empty() {
        plan.warnings.push(format!(
            "the record maps to no work items in this plan tree — nothing to close out; {NO_SCOPE_HINT}"
        ));
    }

    for item in &scope.items {
        match lifecycle(&item.raw_status) {
            Life::Releasable => plan.advances.push(Advance {
                id: item.id.clone(),
                module: item.module.clone(),
                file: item.file.clone(),
                from: item.raw_status.clone(),
                to: format!("Complete: {}", plan.date),
                evidence: evidence_line(&plan.version, &plan.date, &plan.tag, &item.raw_status),
            }),
            Life::Complete => plan.skips.push(Skip {
                id: item.id.clone(),
                reason: "already Complete".to_string(),
            }),
            Life::Open => plan.skips.push(Skip {
                id: item.id.clone(),
                reason: format!(
                    "status '{}' is not a release state",
                    if item.raw_status.is_empty() {
                        "(none)"
                    } else {
                        &item.raw_status
                    }
                ),
            }),
        }
    }

    // A module row only advances when this closeout actually advanced items in
    // it *and* every item in it is now terminal — the same judgement the
    // manual sweeps made, and it never touches a module the release did not
    // move.
    let mut touched: BTreeMap<String, ()> = BTreeMap::new();
    for advance in &plan.advances {
        touched.insert(advance.file.clone(), ());
    }
    for file in touched.keys() {
        let advanced: BTreeSet<&str> = plan
            .advances
            .iter()
            .filter(|advance| &advance.file == file)
            .map(|advance| advance.id.as_str())
            .collect();
        let all_terminal = tree.items_in(file).all(|item| {
            advanced.contains(item.id.as_str()) || lifecycle(&item.raw_status) == Life::Complete
        });
        if !all_terminal {
            continue;
        }
        let Some(module) = tree.module_by_file(file) else {
            continue;
        };
        if lifecycle(&module.raw_status) == Life::Complete {
            continue;
        }
        let Ok(module_plan) = PlanFile::load(file) else {
            continue;
        };
        if let Some((line, col, from)) = module_status_cell(&module_plan) {
            plan.rows.push(RowChange {
                kind: "module",
                module: module.id.clone(),
                file: file.clone(),
                line,
                col,
                from,
                to: "Complete".to_string(),
            });
        }
        let Some(index_file) = &module.index_file else {
            continue;
        };
        let Ok(index_plan) = PlanFile::load(index_file) else {
            continue;
        };
        match index_status_cell(&index_plan, file) {
            Some((line, col, from)) if lifecycle(&from) != Life::Complete => {
                plan.rows.push(RowChange {
                    kind: "index",
                    module: module.id.clone(),
                    file: index_file.clone(),
                    line,
                    col,
                    from,
                    to: "Complete".to_string(),
                });
            }
            Some(_) => {}
            None => plan.warnings.push(format!(
                "{index_file} has no row linking {file}; its index status was left alone"
            )),
        }
    }

    plan.advances.sort_by(|a, b| a.id.cmp(&b.id));
    plan.skips.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plan)
}

/// Apply a plan: item statuses and evidence first, then the table rows (whose
/// cells are relocated against the rewritten file).
fn apply_close(plan: &ClosePlan) -> Result<(), String> {
    for advance in &plan.advances {
        rewrite_work_item(&advance.file, &advance.id, RewriteMode::Status, &advance.to)?;
        set_item_field(&advance.file, &advance.id, "Released", &advance.evidence)?;
    }
    for row in &plan.rows {
        // Work-item rewrites can shift lines, so re-locate the cell by content.
        let plan_file = PlanFile::load(&row.file)
            .map_err(|err| format!("cannot re-read {}: {err}", row.file))?;
        let located = if row.kind == "module" {
            module_status_cell(&plan_file)
        } else {
            let module_file = plan
                .rows
                .iter()
                .find(|other| other.kind == "module" && other.module == row.module)
                .map(|other| other.file.clone())
                .unwrap_or_default();
            index_status_cell(&plan_file, &module_file)
        };
        let (line, col, _) = located.ok_or_else(|| {
            format!(
                "{}: the {} Status cell for {} moved before the write",
                row.file, row.kind, row.module
            )
        })?;
        set_table_cell(&row.file, line, col, &row.to)?;
    }
    Ok(())
}

// --- Commands ----------------------------------------------------------------

/// `aps release new <version>` — copy the release template into
/// `plans/releases/v<version>.md` with the version, dates and previous release
/// filled in. Never overwrites an existing record.
pub fn cmd_release_new(plan_root: &str, version: &str) -> i32 {
    let plans = Path::new(plan_root);
    if !plans.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let version = normalize_version(version);
    if version.is_empty() || !version.starts_with(|c: char| c.is_ascii_digit()) {
        eprintln!("error: '{version}' is not a version — expected 0.6.0 or v0.6.0");
        return 1;
    }
    let target = record_path(plans, &version);
    if target.exists() {
        eprintln!(
            "error: {} already exists — refusing to overwrite a release record",
            target.display()
        );
        return 1;
    }

    // A project's own template wins; the scaffolded copy lives beside the
    // records as a dotfile, which `aps lint` skips.
    let local = releases_dir(plans).join(".release.template.md");
    let (template, source) = match fs::read_to_string(&local) {
        Ok(text) => (text, local.display().to_string()),
        Err(_) => (
            EMBEDDED_TEMPLATE.to_string(),
            "templates/release.template.md (embedded)".to_string(),
        ),
    };

    let previous = list_records(plans)
        .into_iter()
        .map(|(found, _)| found)
        .rfind(|found| version_key(found) < version_key(&version))
        .unwrap_or_default();
    let today = date::today_utc_ymd();
    let body = render_template(&template, &version, &previous, &today);

    if let Err(err) = fs::create_dir_all(releases_dir(plans)) {
        eprintln!(
            "error: cannot create {}: {err}",
            releases_dir(plans).display()
        );
        return 1;
    }
    if let Err(err) = fs::write(&target, body) {
        eprintln!("error: cannot write {}: {err}", target.display());
        return 1;
    }

    println!("Created {}", target.display());
    println!("  from:     {source}");
    println!("  version:  v{version}");
    println!(
        "  previous: {}",
        if previous.is_empty() {
            "(none found)".to_string()
        } else {
            format!("v{previous}")
        }
    );
    println!("  planning: {today}");
    println!("Next: fill in ## Release Theme and ## What Ships, then `aps lint`.");
    0
}

/// Fill the template's placeholders. The multi-line guidance comment is
/// dropped — every record in the wild keeps only the one-line APS pointer.
fn render_template(template: &str, version: &str, previous: &str, today: &str) -> String {
    let mut body = String::new();
    let mut skipping = false;
    for line in template.lines() {
        if line.trim() == "<!--" {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == "-->" {
                skipping = false;
            }
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    // Leading blank lines left by the dropped comment.
    let body = body.trim_start_matches('\n').to_string();

    let previous_label = if previous.is_empty() {
        "[previous-version]".to_string()
    } else {
        previous.to_string()
    };
    // The header table's values go through the width-preserving cell writer
    // rather than plain substitution, so filling in a version longer than the
    // placeholder does not leave the table ragged. Applied to the original
    // line, before any substitution has changed the cell's width.
    let cells = [
        ("Target", format!("v{version}")),
        ("Previous release", format!("v{previous_label}")),
        ("Date", format!("{today} (planning), YYYY-MM-DD (shipped)")),
    ];
    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        let label = line
            .starts_with('|')
            .then(|| line.split('|').nth(1).map(str::trim).unwrap_or(""))
            .filter(|_| !is_separator_row(line))
            .unwrap_or("");
        let rendered = match cells.iter().find(|(name, _)| *name == label) {
            Some((_, value)) => replace_cell(line, 2, value),
            None => line
                .replace("v[previous-version]", &format!("v{previous_label}"))
                .replace("v[version]", &format!("v{version}"))
                .replace("[version]", version),
        };
        out.push_str(&rendered);
        out.push('\n');
    }
    out
}

/// `aps release status [version]` — work-item completion for a release.
pub fn cmd_release_status(plan_root: &str, version: Option<&str>) -> i32 {
    let plans = Path::new(plan_root);
    if !plans.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let path = match version {
        Some(version) => {
            let version = normalize_version(version);
            let path = record_path(plans, &version);
            if !path.is_file() {
                eprintln!("error: no release record at {}", path.display());
                return 1;
            }
            path
        }
        None => match list_records(plans).pop() {
            Some((_, path)) => path,
            None => {
                eprintln!(
                    "error: no release records under {} — run `aps release new <version>`",
                    releases_dir(plans).display()
                );
                return 1;
            }
        },
    };

    let record = match parse_record(&path) {
        Ok(record) => record,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };
    let tree = load_tree(plans);

    println!(
        "Release v{} — {}",
        record.version,
        if record.status.is_empty() {
            "(no Status row)"
        } else {
            &record.status
        }
    );
    println!("Record:   {}", record.path);
    // Each milestone is reported only when the record labels it. An unshipped
    // release has no ship date, and a blank is the honest rendering.
    println!(
        "Planning: {}   Cut: {}   Shipped: {}",
        blank_as_dash(&record.planning_date),
        blank_as_dash(&record.cut_date),
        blank_as_dash(&record.ship_date)
    );
    println!(
        "Previous: {}   Tag: {}",
        if record.previous.is_empty() {
            "-".to_string()
        } else {
            format!("v{}", record.previous)
        },
        blank_as_dash(&record.tag_commit)
    );

    let scope = build_scope(&record, &tree);
    let mut complete = 0usize;
    let mut releasable = 0usize;
    let mut open = 0usize;
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for file in &scope.module_files {
        let Some(module) = tree.module_by_file(file) else {
            continue;
        };
        let mut module_complete = 0usize;
        let mut module_total = 0usize;
        let mut module_releasable = 0usize;
        for item in tree.items_in(file) {
            module_total += 1;
            match lifecycle(&item.raw_status) {
                Life::Complete => {
                    module_complete += 1;
                    complete += 1;
                }
                Life::Releasable => {
                    module_releasable += 1;
                    releasable += 1;
                }
                Life::Open => open += 1,
            }
        }
        let note = if module_releasable > 0 {
            format!(
                "module: {}, {module_releasable} awaiting closeout",
                blank_as_dash(&module.raw_status)
            )
        } else {
            format!("module: {}", blank_as_dash(&module.raw_status))
        };
        rows.push((
            module.id.clone(),
            format!("{module_complete}/{module_total} complete"),
            note,
        ));
    }
    // Items named directly but outside any scoped module still count.
    for item in &scope.items {
        if scope.module_files.contains(&item.file) {
            continue;
        }
        match lifecycle(&item.raw_status) {
            Life::Complete => complete += 1,
            Life::Releasable => releasable += 1,
            Life::Open => open += 1,
        }
    }

    println!();
    if rows.is_empty() {
        println!("Modules: none — this record names no module plans.");
        println!("  To map it to the plan tree, {NO_SCOPE_HINT}.");
    } else {
        println!("Modules ({}):", rows.len());
        let width = rows.iter().map(|row| row.0.len()).max().unwrap_or(0);
        let counts = rows.iter().map(|row| row.1.len()).max().unwrap_or(0);
        for (id, count, note) in &rows {
            println!("  {id:<width$}  {count:<counts$}  {note}");
        }
    }
    let total = complete + releasable + open;
    println!();
    println!(
        "Work items: {total} in scope — {complete} Complete, {releasable} awaiting closeout, {open} open"
    );
    if releasable > 0 {
        println!(
            "Run `aps release close {}` to preview the closeout.",
            record.version
        );
    }
    if !scope.unresolved.is_empty() {
        println!(
            "Unresolved: {} named but absent from the plan tree",
            scope.unresolved.join(", ")
        );
    }
    for warning in record.warnings.iter().chain(scope.warnings.iter()) {
        eprintln!("warning: {warning}");
    }
    0
}

fn blank_as_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

/// `aps release notes <version>` — a markdown draft of the Complete items
/// since the previous release, grouped by module.
pub fn cmd_release_notes(plan_root: &str, version: &str) -> i32 {
    let plans = Path::new(plan_root);
    if !plans.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let version = normalize_version(version);
    let path = record_path(plans, &version);
    if !path.is_file() {
        eprintln!(
            "error: no release record at {} — run `aps release new {version}` first",
            path.display()
        );
        return 1;
    }
    let record = match parse_record(&path) {
        Ok(record) => record,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };
    let tree = load_tree(plans);

    // Lower bound: the previous release's date from this record, else that
    // record's own shipped date. Without one, every dated Complete item counts.
    let mut since = record.previous_date.clone();
    if since.is_empty() && !record.previous.is_empty() {
        let previous_path = record_path(plans, &record.previous);
        if previous_path.is_file()
            && let Ok(previous) = parse_record(&previous_path)
        {
            since = previous.ship_date;
        }
    }
    let since_days = date::parse_civil_date(&since);
    let until_days = date::parse_civil_date(&record.ship_date);

    // Scope to the modules the record says shipped. A record fresh from the
    // template names none, so notes — which only reads — falls back to the
    // whole tree rather than emitting nothing. `close`, which writes, does not.
    let resolved = build_scope(&record, &tree);
    let scoped_to_record = !resolved.module_files.is_empty();
    let scope: Vec<String> = if scoped_to_record {
        resolved.module_files.clone()
    } else {
        tree.modules
            .iter()
            .map(|module| module.file.clone())
            .collect()
    };

    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut undated: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in &scope {
        let Some(module) = tree.module_by_file(file) else {
            continue;
        };
        let heading = format!("{} ({})", module_label(file), module.id);
        for item in tree.items_in(file) {
            if lifecycle(&item.raw_status) != Life::Complete {
                continue;
            }
            let date = dates_in(&item.raw_status).first().cloned();
            let Some(date) = date else {
                undated
                    .entry(heading.clone())
                    .or_default()
                    .push(format!("- **{}:** {}", item.id, item.title));
                continue;
            };
            let days = date::parse_civil_date(&date);
            if let (Some(days), Some(lower)) = (days, since_days)
                && days <= lower
            {
                continue;
            }
            if let (Some(days), Some(upper)) = (days, until_days)
                && days > upper
            {
                continue;
            }
            groups
                .entry(heading.clone())
                .or_default()
                .push(format!("- **{}:** {} ({date})", item.id, item.title));
        }
    }

    println!(
        "## v{}{}",
        record.version,
        if record.ship_date.is_empty() {
            String::new()
        } else {
            format!(" — {}", record.ship_date)
        }
    );
    println!();
    let window = match (record.previous.is_empty(), since.is_empty()) {
        (false, false) => format!("since v{} ({since})", record.previous),
        (false, true) => format!("since v{}", record.previous),
        _ => "to date".to_string(),
    };
    println!("Complete work items {window}.");

    if groups.is_empty() && undated.is_empty() {
        println!();
        println!("_No Complete work items in scope._");
    }
    for (heading, mut items) in groups {
        items.sort();
        println!();
        println!("### {heading}");
        println!();
        for item in items {
            println!("{item}");
        }
    }
    if !undated.is_empty() {
        println!();
        println!("### Undated (verify manually)");
        println!();
        println!(
            "<!-- These items are Complete but carry no date, so they could not be filtered to this release window. -->"
        );
        for (heading, mut items) in undated {
            items.sort();
            println!();
            println!("**{heading}**");
            println!();
            for item in items {
                println!("{item}");
            }
        }
    }

    if !scoped_to_record {
        eprintln!(
            "warning: this record names no module plans, so notes cover the whole plan tree; {NO_SCOPE_HINT}"
        );
    }
    for warning in record.warnings.iter().chain(resolved.warnings.iter()) {
        eprintln!("warning: {warning}");
    }
    0
}

/// `plans/modules/monorepo.aps.md` → `monorepo`.
fn module_label(file: &str) -> String {
    Path::new(file)
        .file_name()
        .map(|name| {
            name.to_string_lossy()
                .trim_end_matches(".aps.md")
                .to_string()
        })
        .unwrap_or_else(|| file.to_string())
}

/// `aps release close <version>` — advance released work. Dry run by default.
pub fn cmd_release_close(
    plan_root: &str,
    version: &str,
    apply: bool,
    force: bool,
    tag: Option<&str>,
    date_override: Option<&str>,
) -> i32 {
    let plans = Path::new(plan_root);
    if !plans.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let plan = match plan_close(plans, version, tag, date_override) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let shipped = SHIPPED_STATES
        .iter()
        .any(|state| plan.record_status.to_lowercase().starts_with(state));

    println!(
        "Release v{} closeout — {} ({})",
        plan.version,
        if plan.record_status.is_empty() {
            "(no Status row)"
        } else {
            &plan.record_status
        },
        plan.record
    );
    println!(
        "Release date: {}   Tag commit: {}",
        plan.date,
        blank_as_dash(&plan.tag)
    );
    println!(
        "Mode: {}",
        if apply {
            "apply — files will be written"
        } else {
            "dry run — no files written"
        }
    );

    println!();
    if plan.advances.is_empty() {
        println!("Advancing 0 work items — nothing is in a release state.");
    } else {
        println!("Advancing {} work item(s):", plan.advances.len());
        let width = plan
            .advances
            .iter()
            .map(|advance| advance.id.len())
            .max()
            .unwrap_or(0);
        for advance in &plan.advances {
            println!(
                "  {:<width$}  {}  {} -> {}",
                advance.id, advance.module, advance.from, advance.to
            );
            println!("    + - **Released:** {}", advance.evidence);
            println!("    in {}", advance.file);
        }
    }

    if !plan.rows.is_empty() {
        println!();
        println!("Advancing {} status row(s):", plan.rows.len());
        for row in &plan.rows {
            println!(
                "  {:<6} {}  {} -> {}  ({})",
                row.kind, row.module, row.from, row.to, row.file
            );
        }
    }

    if !plan.skips.is_empty() {
        println!();
        let complete = plan
            .skips
            .iter()
            .filter(|skip| skip.reason == "already Complete")
            .count();
        println!(
            "Skipped {} work item(s) — {complete} already Complete:",
            plan.skips.len()
        );
        for skip in &plan.skips {
            if skip.reason != "already Complete" {
                println!("  {}: {}", skip.id, skip.reason);
            }
        }
    }

    if !plan.unresolved.is_empty() {
        println!();
        println!(
            "Unresolved {} work item(s) named by the record but absent from the plan tree:",
            plan.unresolved.len()
        );
        for id in &plan.unresolved {
            println!("  {id}");
        }
    }

    for warning in &plan.warnings {
        eprintln!("warning: {warning}");
    }

    println!();
    if !apply {
        println!(
            "Dry run — no files changed. Re-run with --apply to advance {} item(s) and {} row(s).",
            plan.advances.len(),
            plan.rows.len()
        );
        if !shipped && !plan.advances.is_empty() {
            eprintln!(
                "warning: record Status is '{}', not Shipped/Released/Archived — --apply will refuse without --force",
                blank_as_dash(&plan.record_status)
            );
        }
        return 0;
    }

    if !shipped && !force {
        eprintln!(
            "error: record Status is '{}' — refusing to advance work items for a release that has not shipped (pass --force to override)",
            blank_as_dash(&plan.record_status)
        );
        return 1;
    }
    if plan.advances.is_empty() && plan.rows.is_empty() {
        println!("Nothing to apply — no files changed.");
        return 0;
    }
    if let Err(err) = apply_close(&plan) {
        eprintln!("error: {err}");
        return 1;
    }
    println!(
        "Closeout complete — advanced {} work item(s) and {} status row(s) for v{}.",
        plan.advances.len(),
        plan.rows.len(),
        plan.version
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Fixture: the plan tree as it stood before the 2026-07-13 v0.5.0
    // sweep (recovered with `git show d9cb739^:<path>`). MONO-007 and MONO-008
    // are the two Merged items the manual sweep advanced; MONO-001…006 were
    // already Complete, the monorepo module row and its index row said
    // In Progress.

    const PRE_SWEEP_MONOREPO: &str = r#"# Monorepo Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| MONO | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-01

## Purpose

Nested index.aps.md plans, federated lint + orchestration.

## Work Items

### MONO-001: Decision table for tags vs nested plans

- **Intent:** Document the two tiers
- **Expected Outcome:** A decision table
- **Validation:** `true`
- **Status:** Complete

### MONO-002: Federated lint traversal

- **Intent:** Lint a federation
- **Expected Outcome:** Transitive traversal
- **Validation:** `true`
- **Status:** Complete

### MONO-003: Federated orchestration

- **Intent:** Orchestrate a federation
- **Expected Outcome:** next/start/complete traverse
- **Validation:** `true`
- **Status:** Complete

### MONO-004: Root roll-up view

- **Intent:** Roll up child completion
- **Expected Outcome:** `aps rollup`
- **Validation:** `true`
- **Status:** Complete

### MONO-005: Scaffold nested layouts

- **Intent:** Scaffold a nested tree
- **Expected Outcome:** `--scope nested`
- **Validation:** `true`
- **Status:** Complete

### MONO-006: Nested-plans guide and worked example

- **Intent:** Document the tier
- **Expected Outcome:** Guide plus storefront example
- **Validation:** `true`
- **Status:** Complete

### MONO-007: Rust CLI parity for nested-plan lint

- **Intent:** Keep the three CLIs in lockstep
- **Expected Outcome:** Rust matches bash for federated lint
- **Validation:** `true`
- **Confidence:** high
- **Dependencies:** MONO-002 (complete)
- **Status:** Merged
- **Notes:** Completes MONO-002's parity contract (index D-039).

### MONO-008: Child-scoped module statuses

- **Intent:** Scope module statuses by child
- **Expected Outcome:** Statuses keyed by child in bash and Rust
- **Validation:** `true`
- **Confidence:** medium
- **Dependencies:** MONO-003 (complete)
- **Status:** Merged: 2026-07-10
- **Notes:** `ORCH_MODULE_STATUSES` / `PlanGraph.module_statuses`.

## Decisions

- D-040: Nested plans are opt-in.
"#;

    const PRE_SWEEP_INDEX: &str = r#"# APS Roadmap

| Field   | Value      |
| ------- | ---------- |
| Status  | Active     |
| Owner   | @aneki     |
| Created | 2025-12-31 |
| Updated | 2026-06-17 |

## Problem

APS needs continued development.

## Modules

### Near Term

| Module                                        | Purpose                                                   | Status   |
| --------------------------------------------- | --------------------------------------------------------- | -------- |
| [prompts](./modules/prompts.aps.md)           | Tool-specific prompt variants                             | Ready    |
| [monorepo](./modules/monorepo.aps.md)         | Nested index.aps.md plans, federated lint + orchestration | In Progress |
| [ci-parity](./modules/ci-parity.aps.md)       | Behavioural pwsh + cross-CLI parity checks in CI          | Complete |
"#;

    const PRE_SWEEP_CI_PARITY: &str = r#"# CI Parity Module

| ID  | Owner  | Priority | Status   |
| --- | ------ | -------- | -------- |
| CIP | @aneki | medium   | Complete |

## Purpose

Behavioural pwsh + cross-CLI parity checks in CI.

## Work Items

### CIP-001: Extend the behavioural PowerShell harness

- **Intent:** Behavioural coverage
- **Expected Outcome:** Fixtures execute under pwsh
- **Validation:** `true`
- **Status:** Complete

### CIP-002: Cross-CLI byte-diff parity harness

- **Intent:** Compare ordered findings
- **Expected Outcome:** CI diffs all three CLIs
- **Validation:** `true`
- **Status:** Complete
"#;

    const V050_RECORD: &str = r#"<!-- APS: See docs/workflow.md → "Release Narrative" for guidance -->

# Release Plan: v0.5.0

| Field            | Value                                  |
| ---------------- | -------------------------------------- |
| Target           | v0.5.0                                 |
| Cut from         | `main`                                 |
| Previous release | v0.4.0 (2026-06-22)                    |
| Status           | Shipped                                |
| Date             | 2026-07-11 (planning), 2026-07-13 (shipped) |

## Release Theme

**Federated Plans & Full-Lifecycle Tooling** — v0.5 lets the workflow scale
beyond one plan tree.

## What Ships

### Federated nested plans (MONO)

| Area        | Detail |
| ----------- | ------ |
| Plan layout | Co-located standalone child plans |

**APS module:** [monorepo](../modules/monorepo.aps.md) (MONO-001…006
Complete; MONO-007/008 merged for this cut)

### Cross-implementation parity (CIP)

| Area        | Detail |
| ----------- | ------ |
| Corpus diff | CI compares ordered lint findings |

**APS modules:** [ci-parity](../modules/ci-parity.aps.md) (CIP-001/002 Complete)

## Success Criteria

- [x] The full CLI suite passes for the release candidate.

Ship evidence (2026-07-13): tag `v0.5.0` on `408f8cf` (re-cut after the
`x86_64-pc-windows-gnu` build failure on the first tag). Release workflow run
29204184841 green across all five targets.

## Out of Scope

- **`aps release` subcommand** (REL-005) remains deferred.
- **Prompt variants** (PROMPTS-001…003) remain Ready.

## Related

- **Previous release:** [plans/releases/v0.4.0.md](./v0.4.0.md)
"#;

    /// A temp plan tree reproducing the pre-sweep state. `test/` is off limits
    /// for fixtures, so the corpus lives here and the tree is built per test.
    fn pre_sweep_tree(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "aps-release-{tag}-{}-{}",
            std::process::id(),
            tag.len()
        ));
        let _ = fs::remove_dir_all(&root);
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("releases")).unwrap();
        fs::write(plans.join("index.aps.md"), PRE_SWEEP_INDEX).unwrap();
        fs::write(plans.join("modules/monorepo.aps.md"), PRE_SWEEP_MONOREPO).unwrap();
        fs::write(plans.join("modules/ci-parity.aps.md"), PRE_SWEEP_CI_PARITY).unwrap();
        fs::write(plans.join("releases/v0.5.0.md"), V050_RECORD).unwrap();
        plans
    }

    /// The whole clap surface must build. `propagate_version = true` hands
    /// every subcommand an auto `--version`, which collides with a positional
    /// argument called `version` — clap only catches that at parse time, so
    /// without this the panic ships to users instead of to CI.
    #[test]
    fn clap_surface_builds() {
        use clap::CommandFactory;
        crate::Cli::command().debug_assert();
    }

    // --- Version handling ---

    #[test]
    fn versions_normalize_and_order() {
        assert_eq!(normalize_version("v0.5.0"), "0.5.0");
        assert_eq!(normalize_version(" 0.5.0 "), "0.5.0");
        assert!(version_key("0.9.0") > version_key("0.8.1"));
        assert!(version_key("0.10.0") > version_key("0.9.0"));
        assert!(version_key("1.0.0-beta") < version_key("1.0.0"));
    }

    // --- Prose parsing ---

    #[test]
    fn record_parses_prose_header_dates_and_tag() {
        let plans = pre_sweep_tree("parse");
        let record = parse_record(&plans.join("releases/v0.5.0.md")).unwrap();
        assert_eq!(record.version, "0.5.0");
        assert_eq!(record.status, "Shipped");
        assert_eq!(record.previous, "0.4.0");
        assert_eq!(record.previous_date, "2026-06-22");
        assert_eq!(record.planning_date, "2026-07-11");
        assert_eq!(record.ship_date, "2026-07-13");
        assert_eq!(record.tag_commit, "408f8cf");
        assert!(record.warnings.is_empty(), "{:?}", record.warnings);
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn record_scopes_to_what_ships_only() {
        let plans = pre_sweep_tree("scope");
        let record = parse_record(&plans.join("releases/v0.5.0.md")).unwrap();
        assert_eq!(record.module_links.len(), 2, "{:?}", record.module_links);
        assert!(
            record
                .module_links
                .iter()
                .any(|l| l.ends_with("monorepo.aps.md"))
        );
        assert!(
            record
                .module_links
                .iter()
                .any(|l| l.ends_with("ci-parity.aps.md"))
        );
        // REL-005 and PROMPTS-001…003 live in ## Out of Scope, so they are not
        // part of the cut and must not appear.
        assert!(!record.named_items.iter().any(|id| id == "REL-005"));
        assert!(
            !record
                .named_items
                .iter()
                .any(|id| id.starts_with("PROMPTS"))
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn collapsed_and_ranged_ids_expand() {
        let ids = named_item_ids("[monorepo](x) (MONO-001…006 Complete; MONO-007/008 merged)");
        for number in 1..=8 {
            assert!(
                ids.contains(&format!("MONO-{number:03}")),
                "MONO-{number:03} missing from {ids:?}"
            );
        }
        assert_eq!(named_item_ids("CIP-001/002"), ["CIP-001", "CIP-002"]);
        assert_eq!(named_item_ids("REL-001, REL-003"), ["REL-001", "REL-003"]);
        // Decision refs and prose numerals are not work items.
        assert!(named_item_ids("see D-039 and UTF-8 and W003").is_empty());
        // A range typo never enumerates a silly number of IDs.
        assert_eq!(named_item_ids("MONO-001…9999"), ["MONO-001"]);
    }

    /// The `Date`-row label vocabulary actually in use across
    /// `plans/releases/*.md`, enumerated from those files rather than guessed.
    #[test]
    fn date_row_labels_in_the_wild() {
        // v0.5.0, v0.7.0 — planning/shipped, comma- or semicolon-separated.
        let dates = labelled_dates("2026-07-11 (planning), 2026-07-13 (shipped)");
        assert_eq!(date_labelled(&dates, "planning").unwrap(), "2026-07-11");
        assert_eq!(date_labelled(&dates, "ship").unwrap(), "2026-07-13");

        // v0.8.0, v0.8.1 — cut/shipped. The cut date is NOT the ship date.
        let dates = labelled_dates("2026-07-28 (cut); 2026-07-29 (shipped)");
        assert_eq!(date_labelled(&dates, "cut").unwrap(), "2026-07-28");
        assert_eq!(date_labelled(&dates, "ship").unwrap(), "2026-07-29");
        assert_eq!(date_labelled(&dates, "planning"), None);

        // v0.6.0 — one date carrying a combined label.
        let dates = labelled_dates("2026-07-16 (planning + shipped)");
        assert_eq!(date_labelled(&dates, "planning").unwrap(), "2026-07-16");
        assert_eq!(date_labelled(&dates, "ship").unwrap(), "2026-07-16");

        // v0.4.0 — planning/cut, and nothing that says shipped.
        let dates = labelled_dates("2026-06-21 (planning), 2026-06-22 (cut)");
        assert_eq!(date_labelled(&dates, "ship"), None);

        // v0.9.0 — planning only.
        let dates = labelled_dates("2026-09-12 (planning)");
        assert_eq!(date_labelled(&dates, "ship"), None);

        // v0.3.0 — a bare date with no label at all.
        let dates = labelled_dates("2026-05-20");
        assert_eq!(dates, [("2026-05-20".to_string(), String::new())]);
    }

    #[test]
    fn unshipped_record_reports_no_ship_date() {
        let root = std::env::temp_dir().join(format!("aps-release-dates-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let releases = root.join("plans/releases");
        fs::create_dir_all(&releases).unwrap();

        let write = |version: &str, status: &str, date_row: &str| {
            fs::write(
                releases.join(format!("v{version}.md")),
                format!(
                    "# Release Plan: v{version}\n\n| Field | Value |\n| --- | --- |\n| Target | v{version} |\n| Status | {status} |\n| Date | {date_row} |\n\n## Release Theme\n\nt\n\n## What Ships\n\nw\n"
                ),
            )
            .unwrap();
            parse_record(&releases.join(format!("v{version}.md"))).unwrap()
        };

        // Cutting, planning date only: nothing shipped, so no ship date.
        let cutting = write("0.9.0", "Cutting", "2026-09-12 (planning)");
        assert_eq!(cutting.planning_date, "2026-09-12");
        assert_eq!(
            cutting.ship_date, "",
            "an unshipped release has no ship date"
        );
        assert_eq!(cutting.cut_date, "");

        // `(cut)` is not `(shipped)`.
        let cut = write("0.8.0", "Shipped", "2026-07-28 (cut); 2026-07-29 (shipped)");
        assert_eq!(cut.cut_date, "2026-07-28");
        assert_eq!(cut.ship_date, "2026-07-29");
        assert_eq!(cut.planning_date, "");

        // Shipped, but the row never labels a shipped date: report none, warn.
        let unlabelled_ship = write(
            "0.4.0",
            "Shipped",
            "2026-06-21 (planning), 2026-06-22 (cut)",
        );
        assert_eq!(unlabelled_ship.ship_date, "");
        assert!(
            unlabelled_ship
                .warnings
                .iter()
                .any(|w| w.contains("labels no date as shipped"))
        );

        // A wholly unlabelled row on a terminal record is the ship date.
        let bare = write("0.3.0", "Shipped", "2026-05-20");
        assert_eq!(bare.ship_date, "2026-05-20");
        assert_eq!(bare.planning_date, "2026-05-20");

        // The same bare row on a non-terminal record is not.
        let bare_planning = write("0.2.0", "Planning", "2026-05-20");
        assert_eq!(bare_planning.ship_date, "");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn plain_text_module_references_resolve() {
        // v0.9.0's original phrasing: names, not links, with asides.
        let refs = module_references(&[
            "**APS modules:** cli-redesign (6/7 work items done — CLI-002 remains, gated on",
            "TEAM).",
        ]);
        assert!(refs.contains(&"cli-redesign".to_string()), "{refs:?}");

        // Link labels survive, targets and prose do not trip it up.
        let refs = module_references(&[
            "**APS modules:** [prompts](../modules/prompts.aps.md) (Complete),",
            "[examples](../modules/examples.aps.md) (Complete). Closes ISS-011.",
        ]);
        assert!(refs.contains(&"prompts".to_string()), "{refs:?}");
        assert!(refs.contains(&"examples".to_string()), "{refs:?}");

        // Lines that are not APS-module references contribute nothing.
        assert!(module_references(&["Some prose about prompts and examples."]).is_empty());
    }

    #[test]
    fn module_names_resolve_by_stem_id_and_unique_prefix() {
        let plans = pre_sweep_tree("names");
        let tree = load_tree(&plans);
        assert_eq!(
            tree.module_by_name("monorepo").map(|m| m.id.as_str()),
            Some("MONO")
        );
        assert_eq!(
            tree.module_by_name("MONO").map(|m| m.id.as_str()),
            Some("MONO")
        );
        assert_eq!(
            tree.module_by_name("ci-parity").map(|m| m.id.as_str()),
            Some("CIP")
        );
        assert!(tree.module_by_name("nope").is_none());
        // Too short to prefix-match, and not an exact stem or ID.
        assert!(tree.module_by_name("mon").is_none());
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn related_module_specs_are_the_documented_fallback() {
        let plans = pre_sweep_tree("fallback");
        let record = plans.join("releases/v0.5.0.md");
        // Strip the module references out of What Ships, leaving them only in
        // a ## Related **Module specs:** bullet.
        let text = fs::read_to_string(&record)
            .unwrap()
            .replace(
                "**APS module:** [monorepo](../modules/monorepo.aps.md) (MONO-001…006",
                "(MONO-001…006",
            )
            .replace(
                "**APS modules:** [ci-parity](../modules/ci-parity.aps.md) (CIP-001/002 Complete)",
                "(CIP-001/002 Complete)",
            )
            .replace(
                "- **Previous release:** [plans/releases/v0.4.0.md](./v0.4.0.md)",
                "- **Module specs:** [monorepo](../modules/monorepo.aps.md),\n  [ci-parity](../modules/ci-parity.aps.md)",
            );
        fs::write(&record, text).unwrap();

        let parsed = parse_record(&record).unwrap();
        assert_eq!(parsed.module_links.len(), 2, "{:?}", parsed.module_links);
        assert!(
            parsed
                .warnings
                .iter()
                .any(|w| w.contains("**Module specs:**"))
        );
        // The closeout still finds the same two Merged items.
        let plan = plan_close(&plans, "0.5.0", None, None).unwrap();
        let ids: Vec<&str> = plan.advances.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["MONO-007", "MONO-008"]);
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn tag_commit_survives_a_wrapped_line() {
        let blocks = prose_blocks(&[
            "Ship evidence (2026-07-13): tag `v0.9.0` on".to_string(),
            "`deadbee` after the re-cut.".to_string(),
        ]);
        assert_eq!(find_tag_commit(&blocks, "0.9.0"), "deadbee");
    }

    #[test]
    fn malformed_record_without_a_header_table_is_rejected() {
        let root = std::env::temp_dir().join(format!("aps-release-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let releases = root.join("plans/releases");
        fs::create_dir_all(&releases).unwrap();
        fs::write(
            releases.join("v9.9.9.md"),
            "# Release Plan: v9.9.9\n\nprose only\n",
        )
        .unwrap();
        let err = parse_record(&releases.join("v9.9.9.md")).unwrap_err();
        assert!(err.contains("R002"), "{err}");
        assert_eq!(
            cmd_release_close(
                &root.join("plans").to_string_lossy(),
                "9.9.9",
                false,
                false,
                None,
                None
            ),
            1
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn record_without_what_ships_falls_back_to_the_whole_file() {
        let root = std::env::temp_dir().join(format!("aps-release-nows-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("releases")).unwrap();
        fs::write(plans.join("modules/monorepo.aps.md"), PRE_SWEEP_MONOREPO).unwrap();
        fs::write(
            plans.join("releases/v0.5.0.md"),
            "# Release Plan: v0.5.0\n\n| Field | Value |\n| --- | --- |\n| Target | v0.5.0 |\n| Status | Shipped |\n| Date | 2026-07-13 (shipped) |\n\nShipped [monorepo](../modules/monorepo.aps.md).\n",
        )
        .unwrap();
        let record = parse_record(&plans.join("releases/v0.5.0.md")).unwrap();
        assert!(record.warnings.iter().any(|w| w.contains("R004")));
        assert_eq!(record.module_links.len(), 1);
        fs::remove_dir_all(&root).ok();
    }

    // --- Lifecycle ---

    #[test]
    fn lifecycle_recognises_release_states_and_canonical_aliases() {
        assert_eq!(lifecycle("Merged"), Life::Releasable);
        assert_eq!(lifecycle("Merged: 2026-07-10"), Life::Releasable);
        assert_eq!(lifecycle("Released in v0.5.0"), Life::Releasable);
        assert_eq!(lifecycle("Shipped"), Life::Releasable);
        assert_eq!(lifecycle("Complete"), Life::Complete);
        assert_eq!(lifecycle("Complete: 2026-06-08"), Life::Complete);
        assert_eq!(lifecycle("Done"), Life::Complete);
        assert_eq!(lifecycle("Ready"), Life::Open);
        assert_eq!(lifecycle("In Progress"), Life::Open);
        assert_eq!(lifecycle("Proposed"), Life::Open);
        assert_eq!(lifecycle(""), Life::Open);
        // Not a release state: a word that merely starts the same way.
        assert_eq!(lifecycle("Mergeable"), Life::Open);
    }

    // --- The v0.5.0 acceptance test ---

    #[test]
    fn close_050_reproduces_the_manual_sweep() {
        let plans = pre_sweep_tree("accept");
        let plan = plan_close(&plans, "0.5.0", None, None).unwrap();

        assert_eq!(plan.date, "2026-07-13");
        assert_eq!(plan.tag, "408f8cf");
        let ids: Vec<&str> = plan.advances.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(
            ids,
            ["MONO-007", "MONO-008"],
            "only the Merged items advance"
        );
        assert_eq!(plan.advances[0].to, "Complete: 2026-07-13");
        assert_eq!(
            plan.advances[0].evidence,
            "v0.5.0 (2026-07-13, tag `408f8cf`) — was Merged"
        );
        assert_eq!(
            plan.advances[1].evidence,
            "v0.5.0 (2026-07-13, tag `408f8cf`) — was Merged: 2026-07-10"
        );
        // MONO-001…006 plus CIP-001/002 are already Complete.
        assert_eq!(plan.skips.len(), 8, "{:?}", plan.skips);
        assert!(plan.skips.iter().all(|s| s.reason == "already Complete"));
        assert!(plan.unresolved.is_empty(), "{:?}", plan.unresolved);

        // Module + index rows advance; ci-parity is untouched (nothing moved).
        let rows: Vec<(&str, &str, &str)> = plan
            .rows
            .iter()
            .map(|r| (r.kind, r.module.as_str(), r.from.as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                ("module", "MONO", "In Progress"),
                ("index", "MONO", "In Progress")
            ]
        );

        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn dry_run_writes_nothing_and_lists_the_same_advances() {
        let plans = pre_sweep_tree("dry");
        let before: Vec<String> = ["modules/monorepo.aps.md", "index.aps.md"]
            .iter()
            .map(|name| fs::read_to_string(plans.join(name)).unwrap())
            .collect();

        let preview = plan_close(&plans, "0.5.0", None, None).unwrap();
        assert_eq!(
            cmd_release_close(&plans.to_string_lossy(), "0.5.0", false, false, None, None),
            0
        );
        for (name, text) in ["modules/monorepo.aps.md", "index.aps.md"]
            .iter()
            .zip(&before)
        {
            assert_eq!(
                &fs::read_to_string(plans.join(name)).unwrap(),
                text,
                "{name} must be untouched by a dry run"
            );
        }

        // The same plan drives both paths, so --apply advances exactly the
        // items the dry run listed.
        assert_eq!(
            cmd_release_close(&plans.to_string_lossy(), "0.5.0", true, false, None, None),
            0
        );
        let after = fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap();
        for advance in &preview.advances {
            assert!(
                after.contains(&format!("- **Released:** {}", advance.evidence)),
                "{} evidence missing",
                advance.id
            );
        }
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn apply_050_stamps_items_and_advances_rows() {
        let plans = pre_sweep_tree("apply");
        assert_eq!(
            cmd_release_close(&plans.to_string_lossy(), "0.5.0", true, false, None, None),
            0
        );

        let monorepo = fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap();
        assert!(!monorepo.contains("- **Status:** Merged"));
        assert_eq!(
            monorepo
                .matches("- **Status:** Complete: 2026-07-13")
                .count(),
            2
        );
        assert!(monorepo.contains(
            "- **Released:** v0.5.0 (2026-07-13, tag `408f8cf`) — was Merged: 2026-07-10"
        ));
        // The already-Complete items keep their exact prior text.
        assert_eq!(monorepo.matches("- **Status:** Complete\n").count(), 6);
        // The evidence line sits directly after the Status line it explains.
        let mono_007 = monorepo.split("### MONO-007:").nth(1).unwrap();
        assert!(mono_007.contains(
            "- **Status:** Complete: 2026-07-13\n- **Released:** v0.5.0 (2026-07-13, tag `408f8cf`) — was Merged\n"
        ));

        // Module metadata row and the index row both read Complete.
        let module_plan =
            PlanFile::load(&plans.join("modules/monorepo.aps.md").to_string_lossy()).unwrap();
        assert_eq!(module_plan.status().as_deref(), Some("Complete"));
        let index = fs::read_to_string(plans.join("index.aps.md")).unwrap();
        let row = index
            .lines()
            .find(|line| line.contains("modules/monorepo.aps.md"))
            .unwrap();
        assert!(row.trim_end().ends_with("| Complete    |"), "{row}");
        // Sibling rows are untouched.
        assert!(
            index.contains(
                "| Tool-specific prompt variants                             | Ready    |"
            )
        );
        // ci-parity had nothing to advance, so its module file is byte-identical.
        assert_eq!(
            fs::read_to_string(plans.join("modules/ci-parity.aps.md")).unwrap(),
            PRE_SWEEP_CI_PARITY
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn apply_is_idempotent() {
        let plans = pre_sweep_tree("idem");
        let plans_str = plans.to_string_lossy().into_owned();
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", true, false, None, None),
            0
        );
        let once = fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap();
        let index_once = fs::read_to_string(plans.join("index.aps.md")).unwrap();

        // Second run: nothing is in a release state any more.
        let second = plan_close(&plans, "0.5.0", None, None).unwrap();
        assert!(second.advances.is_empty());
        assert!(second.rows.is_empty());
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", true, false, None, None),
            0
        );
        assert_eq!(
            fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap(),
            once
        );
        assert_eq!(
            fs::read_to_string(plans.join("index.aps.md")).unwrap(),
            index_once
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn unresolved_items_are_reported_not_fatal() {
        let plans = pre_sweep_tree("unresolved");
        let record = plans.join("releases/v0.5.0.md");
        let text = fs::read_to_string(&record)
            .unwrap()
            .replace("MONO-007/008 merged", "MONO-007/008/099 merged");
        fs::write(&record, text).unwrap();

        let plan = plan_close(&plans, "0.5.0", None, None).unwrap();
        assert_eq!(plan.unresolved, ["MONO-099"]);
        assert_eq!(plan.advances.len(), 2, "the real items still advance");
        assert_eq!(
            cmd_release_close(&plans.to_string_lossy(), "0.5.0", false, false, None, None),
            0,
            "an unresolved mention is a finding, not a failure"
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn unshipped_record_refuses_to_apply_without_force() {
        let plans = pre_sweep_tree("unshipped");
        let record = plans.join("releases/v0.5.0.md");
        let text = fs::read_to_string(&record).unwrap().replace(
            "| Status           | Shipped",
            "| Status           | Cutting",
        );
        fs::write(&record, text).unwrap();
        let plans_str = plans.to_string_lossy().into_owned();
        let before = fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap();

        // Dry run still previews.
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", false, false, None, None),
            0
        );
        // --apply refuses and changes nothing.
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", true, false, None, None),
            1
        );
        assert_eq!(
            fs::read_to_string(plans.join("modules/monorepo.aps.md")).unwrap(),
            before
        );
        // --force overrides.
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", true, true, None, None),
            0
        );
        assert!(
            fs::read_to_string(plans.join("modules/monorepo.aps.md"))
                .unwrap()
                .contains("- **Released:** v0.5.0")
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn tag_and_date_overrides_win_over_the_record() {
        let plans = pre_sweep_tree("override");
        let plan = plan_close(&plans, "0.5.0", Some("abcdef1"), Some("2026-08-01")).unwrap();
        assert_eq!(plan.date, "2026-08-01");
        assert_eq!(
            plan.advances[0].evidence,
            "v0.5.0 (2026-08-01, tag `abcdef1`) — was Merged"
        );
        assert!(plan_close(&plans, "0.5.0", None, Some("not-a-date")).is_err());
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn module_row_waits_for_every_item_to_be_terminal() {
        let plans = pre_sweep_tree("partial");
        // Leave one item Ready: the module is not finished, so its row stays.
        let module = plans.join("modules/monorepo.aps.md");
        let text = fs::read_to_string(&module).unwrap().replacen(
            "- **Status:** Complete",
            "- **Status:** Ready",
            1,
        );
        fs::write(&module, text).unwrap();

        let plan = plan_close(&plans, "0.5.0", None, None).unwrap();
        assert_eq!(plan.advances.len(), 2);
        assert!(plan.rows.is_empty(), "{:?}", plan.rows);
        assert!(
            plan.skips
                .iter()
                .any(|skip| skip.reason.contains("not a release state"))
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn missing_record_is_an_error() {
        let plans = pre_sweep_tree("missing");
        assert!(plan_close(&plans, "9.9.9", None, None).is_err());
        assert_eq!(
            cmd_release_close(&plans.to_string_lossy(), "9.9.9", false, false, None, None),
            1
        );
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    // --- new / status / notes ---

    #[test]
    fn new_renders_the_template_and_refuses_to_overwrite() {
        let plans = pre_sweep_tree("new");
        let plans_str = plans.to_string_lossy().into_owned();
        assert_eq!(cmd_release_new(&plans_str, "v0.6.0"), 0);
        let body = fs::read_to_string(plans.join("releases/v0.6.0.md")).unwrap();
        assert!(body.starts_with("<!-- APS: See docs/workflow.md"));
        assert!(body.contains("# Release Plan: v0.6.0"));
        assert!(body.contains("| Target           | v0.6.0"));
        assert!(body.contains("| Previous release | v0.5.0"));
        assert!(body.contains(&format!("| Date             | {}", date::today_utc_ymd())));
        assert!(!body.contains("[version]"));
        // The long guidance comment is dropped, like every record in the wild.
        assert!(!body.contains("A release narrative is richer than a CHANGELOG"));
        // Lint's structural requirements survive the render.
        assert!(body.contains("## Release Theme") && body.contains("## What Ships"));
        // Filling in a longer version than the placeholder keeps the header
        // table aligned: every row of it is the same width.
        let widths: BTreeSet<usize> = body
            .lines()
            .take_while(|line| !line.starts_with("## "))
            .filter(|line| line.starts_with('|'))
            .map(|line| line.chars().count())
            .collect();
        assert_eq!(widths.len(), 1, "ragged header table: {widths:?}");

        assert_eq!(cmd_release_new(&plans_str, "0.6.0"), 1, "no overwrite");
        assert_eq!(cmd_release_new(&plans_str, "nope"), 1, "not a version");
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn new_prefers_a_project_local_template() {
        let plans = pre_sweep_tree("localtpl");
        fs::write(
            plans.join("releases/.release.template.md"),
            "# Release Plan: v[version]\n\n| Field | Value |\n| --- | --- |\n| Target | v[version] |\n| Status | Planning |\n\nLOCAL\n",
        )
        .unwrap();
        assert_eq!(cmd_release_new(&plans.to_string_lossy(), "0.7.0"), 0);
        let body = fs::read_to_string(plans.join("releases/v0.7.0.md")).unwrap();
        assert!(body.contains("LOCAL"));
        let record = parse_record(&plans.join("releases/v0.7.0.md")).unwrap();
        assert_eq!(record.version, "0.7.0");
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn status_defaults_to_the_most_recent_record() {
        let plans = pre_sweep_tree("status");
        let plans_str = plans.to_string_lossy().into_owned();
        fs::write(
            plans.join("releases/v0.4.0.md"),
            V050_RECORD.replace("0.5.0", "0.4.0"),
        )
        .unwrap();
        assert_eq!(cmd_release_status(&plans_str, None), 0);
        assert_eq!(cmd_release_status(&plans_str, Some("0.5.0")), 0);
        assert_eq!(cmd_release_status(&plans_str, Some("9.9.9")), 1);
        // The newest record is the one picked by default.
        let (version, _) = list_records(&plans).pop().unwrap();
        assert_eq!(version, "0.5.0");
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    #[test]
    fn notes_requires_a_record_and_lists_dated_complete_items() {
        let plans = pre_sweep_tree("notes");
        let plans_str = plans.to_string_lossy().into_owned();
        assert_eq!(cmd_release_notes(&plans_str, "9.9.9"), 1);
        assert_eq!(cmd_release_notes(&plans_str, "0.5.0"), 0);
        // After the closeout the two advanced items carry 2026-07-13 dates and
        // land inside the v0.4.0 → v0.5.0 window.
        assert_eq!(
            cmd_release_close(&plans_str, "0.5.0", true, false, None, None),
            0
        );
        let tree = load_tree(&plans);
        let dated: Vec<&TreeItem> = tree
            .items
            .iter()
            .filter(|item| item.raw_status.contains("2026-07-13"))
            .collect();
        assert_eq!(dated.len(), 2);
        assert_eq!(cmd_release_notes(&plans_str, "0.5.0"), 0);
        fs::remove_dir_all(plans.parent().unwrap()).ok();
    }

    // --- Table cells ---

    #[test]
    fn cell_replacement_keeps_the_table_aligned() {
        let row = "| MONO | @aneki | medium   | In Progress |";
        assert_eq!(
            replace_cell(row, 4, "Complete"),
            "| MONO | @aneki | medium   | Complete    |"
        );
        // A longer value grows the cell rather than corrupting the row.
        assert_eq!(
            replace_cell("| a | b |", 2, "much longer"),
            "| a | much longer |"
        );
        // Column count never changes.
        assert_eq!(
            replace_cell(row, 4, "Complete").matches('|').count(),
            row.matches('|').count()
        );
        // Out-of-range is a no-op.
        assert_eq!(replace_cell(row, 99, "x"), row);
    }

    #[test]
    fn status_cells_are_located_by_column_name() {
        let plan = PlanFile::from_text("m.aps.md", PRE_SWEEP_MONOREPO);
        let (line, col, value) = module_status_cell(&plan).unwrap();
        assert_eq!(value, "In Progress");
        assert_eq!(
            plan.lines[line],
            "| MONO | @aneki | medium   | In Progress |"
        );
        assert_eq!(col, 4);

        let index = PlanFile::from_text("plans/index.aps.md", PRE_SWEEP_INDEX);
        let (_, col, value) = index_status_cell(&index, "plans/modules/monorepo.aps.md").unwrap();
        assert_eq!((col, value.as_str()), (3, "In Progress"));
        assert!(index_status_cell(&index, "plans/modules/nope.aps.md").is_none());
    }
}

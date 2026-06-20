//! Native `aps lint` (TUI-009, D-028).
//!
//! Implements the bash rule set from lib/rules/*.sh with the same E/W
//! codes, message text, output format, and exit behavior. Quirks of the
//! bash implementation (section-relative line numbers in W010/W011,
//! unescaped JSON strings) are preserved deliberately — the parity
//! contract is byte-identical output on the same input.

use std::collections::HashSet;
use std::io::IsTerminal;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::parser::{self, FileType, PlanFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, Default)]
pub struct LintReport {
    /// (path, type) in lint order.
    pub files: Vec<(String, FileType)>,
    pub findings: Vec<Finding>,
}

impl LintReport {
    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    pub fn warnings(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    fn add(
        &mut self,
        path: &str,
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
        line: Option<usize>,
    ) {
        self.findings.push(Finding {
            path: path.to_string(),
            severity,
            code,
            message: message.into(),
            line,
        });
    }
}

/// Lint a target file or directory. Mirrors `cmd_lint` file collection:
/// directories are scanned recursively; the literal target `plans`/`plans/`
/// also picks up a sibling `designs/` directory.
pub fn lint_target(target: &str) -> Result<LintReport, String> {
    let target_path = Path::new(target);
    if !target_path.exists() {
        return Err(format!("Path not found: {target}"));
    }

    let mut files = Vec::new();
    if target_path.is_file() {
        files.push(target.to_string());
    } else {
        files.extend(parser::find_aps_files(target_path));
        if (target == "plans" || target == "plans/") && Path::new("designs").is_dir() {
            files.extend(parser::find_aps_files(Path::new("designs")));
        }
    }

    if files.is_empty() {
        return Err(format!("No APS files found in: {target}"));
    }

    // Cross-file ID index (`build_id_index`). For a single-file target,
    // widen the index to the surrounding plan tree so cross-module
    // dependencies still resolve.
    let mut index_files = files.clone();
    if target_path.is_file()
        && let Some(parent) = target_path.parent()
    {
        let tdir = std::fs::canonicalize(if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        })
        .unwrap_or_else(|_| parent.to_path_buf());
        let tdir_str = tdir.to_string_lossy().into_owned();
        // Climb out of modules/ (including nested subdirectories).
        let troot = match tdir_str.find("/modules") {
            Some(at)
                if tdir_str[at..] == *"/modules"
                    || tdir_str[at + "/modules".len()..].starts_with('/') =>
            {
                tdir_str[..at].to_string()
            }
            _ => tdir_str,
        };
        index_files.extend(parser::find_aps_files(Path::new(&troot)));
    }
    let tree_ids = build_id_index(&index_files);

    let mut report = LintReport::default();
    for file in &files {
        lint_file(&mut report, file, &tree_ids);
    }
    Ok(report)
}

/// Work item and decision IDs from the plan tree (`build_id_index`).
/// Fence-aware: IDs inside ``` / ~~~ code blocks are examples, not
/// definitions.
fn build_id_index(files: &[String]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for file in files {
        let Ok(plan) = PlanFile::load(file) else {
            continue;
        };
        let mut fence = false;
        for line in &plan.lines {
            if line.starts_with("```") || line.starts_with("~~~") {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            // Work item headers: ### AUTH-001: title
            if let Some(id) = parser::parse_work_item_id(line) {
                ids.insert(id.to_string());
            }
            // Decision entries: - **D-026:** text
            if let Some(rest) = line.strip_prefix("- **D-") {
                let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !digits.is_empty() && rest[digits.len()..].starts_with(':') {
                    ids.insert(format!("D-{digits}"));
                }
            }
        }
    }
    ids
}

fn lint_file(report: &mut LintReport, path: &str, tree_ids: &HashSet<String>) {
    let kind = parser::file_type(path);
    report.files.push((path.to_string(), kind));

    let Ok(plan) = PlanFile::load(path) else {
        report.add(path, Severity::Error, "E000", "Cannot read file", None);
        return;
    };

    match kind {
        FileType::Index => lint_index(report, &plan),
        FileType::Module | FileType::Simple => lint_module(report, &plan, tree_ids),
        FileType::Issues => lint_issues(report, &plan),
        FileType::Design => lint_design(report, &plan),
        FileType::Release => lint_release(report, &plan),
        FileType::Actions | FileType::Archive | FileType::Template => {}
        FileType::Unknown => {
            report.add(
                path,
                Severity::Warning,
                "W000",
                "Unknown file type, skipping validation",
                None,
            );
        }
    }
}

// --- Index rules ---------------------------------------------------------------

fn lint_index(report: &mut LintReport, plan: &PlanFile) {
    if !plan.has_section("## Modules") {
        report.add(
            &plan.path,
            Severity::Error,
            "E004",
            "Missing ## Modules section",
            None,
        );
    }
    check_w019_module_links(report, plan);
    check_w006_conductor_index(report, plan);
    for section in ["## Overview", "## Problem & Success Criteria", "## Modules"] {
        check_empty_section(report, plan, section);
    }
}

/// W006: a module listed under a `### Conductor / Crosscutting` index
/// subsection whose file exists but is not marked `Type: Conductor`. Keeps the
/// index's conductor grouping honest — the inverse of COND-003's module check.
/// Missing/non-module link targets are W019's job, so they are skipped here.
fn check_w006_conductor_index(report: &mut LintReport, plan: &PlanFile) {
    let dir = Path::new(&plan.path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    let mut in_conductor = false;
    for (index, line) in plan.lines.iter().enumerate() {
        if let Some(heading) = line.strip_prefix("### ") {
            let h = heading.to_lowercase();
            in_conductor = h.contains("conductor") || h.contains("crosscutting");
            continue;
        }
        // Any other heading ends the subsection.
        if line.starts_with("## ") {
            in_conductor = false;
        }
        if !in_conductor {
            continue;
        }

        for target in link_targets(line) {
            if !target.ends_with(".aps.md") {
                continue;
            }
            let path = dir.join(&target);
            let Ok(module) = PlanFile::load(&path.to_string_lossy()) else {
                continue; // missing file → W019 already covers it
            };
            if !module.is_conductor() {
                report.add(
                    &plan.path,
                    Severity::Warning,
                    "W006",
                    format!(
                        "Module '{target}' is listed under Conductor / Crosscutting but its file is not marked `Type: Conductor`"
                    ),
                    Some(index + 1),
                );
            }
        }
    }
}

/// Resolve every `](target)` link target on a line, stripping titles, anchors,
/// and URI-scheme links. Shared by the index reference checks.
fn link_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find("](") {
        let after = &rest[open + 2..];
        let Some(close) = after.find(')') else {
            break;
        };
        let mut target = after[..close].to_string();
        rest = &after[close + 1..];

        // Strip markdown link titles: ` "title"` / ` 'title'`.
        if let Some(at) = target.find([' ', '\t']) {
            let tail = target[at..].trim_start();
            if tail.starts_with('"') || tail.starts_with('\'') {
                target.truncate(at);
            }
        }
        if target.starts_with('#') || has_uri_scheme(&target) {
            continue;
        }
        let target = target.split('#').next().unwrap_or("");
        if !target.is_empty() {
            targets.push(target.to_string());
        }
    }
    targets
}

/// W019: link in ## Modules points to a non-existent file. Warning, not
/// error — the scaffold seed index intentionally links a placeholder.
fn check_w019_module_links(report: &mut LintReport, plan: &PlanFile) {
    let dir = Path::new(&plan.path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    let mut in_modules = false;
    for (index, line) in plan.lines.iter().enumerate() {
        if line.starts_with("## Modules") {
            in_modules = true;
            continue;
        }
        if in_modules && line.starts_with("## ") {
            in_modules = false;
        }
        if !in_modules {
            continue;
        }

        // Every `](target)` on the line.
        let mut rest = line.as_str();
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            let Some(close) = after.find(')') else {
                break;
            };
            let mut target = after[..close].to_string();
            rest = &after[close + 1..];

            // Strip markdown link titles: ` "title"` / ` 'title'`.
            if let Some(at) = target.find([' ', '\t']) {
                let tail = target[at..].trim_start();
                if tail.starts_with('"') || tail.starts_with('\'') {
                    target.truncate(at);
                }
            }

            // Skip pure anchors and any URI scheme.
            if target.starts_with('#') || has_uri_scheme(&target) {
                continue;
            }
            // Strip anchor fragment.
            let target = target.split('#').next().unwrap_or("");
            if target.is_empty() {
                continue;
            }

            if !dir.join(target).exists() {
                report.add(
                    &plan.path,
                    Severity::Warning,
                    "W019",
                    format!("Module link target not found: {target}"),
                    Some(index + 1),
                );
            }
        }
    }
}

/// `^[A-Za-z][A-Za-z0-9+.-]*:`
fn has_uri_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

fn check_empty_section(report: &mut LintReport, plan: &PlanFile, section: &str) {
    if plan.has_section(section) && !plan.section_has_content(section) {
        report.add(
            &plan.path,
            Severity::Warning,
            "W004",
            format!("Empty section: {section}"),
            plan.section_line(section),
        );
    }
}

// --- Module / simple rules -------------------------------------------------------

fn lint_module(report: &mut LintReport, plan: &PlanFile, tree_ids: &HashSet<String>) {
    if !plan.has_section("## Purpose") {
        report.add(
            &plan.path,
            Severity::Error,
            "E001",
            "Missing ## Purpose section",
            None,
        );
    }
    if !plan.has_section("## Work Items") {
        report.add(
            &plan.path,
            Severity::Error,
            "E002",
            "Missing ## Work Items section",
            None,
        );
    }
    if !plan.has_metadata_table() {
        report.add(
            &plan.path,
            Severity::Error,
            "E003",
            "Missing ID/Status metadata table",
            None,
        );
    }

    for section in ["## Purpose", "## In Scope"] {
        check_empty_section(report, plan, section);
    }

    if plan.status().as_deref() == Some("Ready") && plan.work_items().is_empty() {
        report.add(
            &plan.path,
            Severity::Warning,
            "W005",
            "Status is Ready but no work items defined",
            None,
        );
    }

    check_w017_last_reviewed(report, plan);

    if plan.is_conductor() {
        check_w002_conductor_refs(report, plan, tree_ids);
    }

    if plan.has_section("## Work Items") {
        lint_work_items(report, plan, tree_ids);
    }
}

/// W002: a conductor module's coordination sections reference a work-item ID
/// that resolves nowhere in the plan tree — most likely a typo. Conductor
/// modules legitimately reference IDs owned by other modules (that is the
/// point), so cross-file references are expected here; only unresolved ones
/// are flagged. Vertical-module dependency typos remain W003's job.
fn check_w002_conductor_refs(report: &mut LintReport, plan: &PlanFile, tree_ids: &HashSet<String>) {
    const SECTIONS: [&str; 2] = ["## Coordinated Modules", "## Cross-Module Work Items"];
    for section in SECTIONS {
        let Some(header_line) = plan.section_line(section) else {
            continue;
        };
        let mut in_comment = false;
        for (offset, line) in plan.section_content(section).iter().enumerate() {
            let trimmed = line.trim_start();
            if in_comment {
                if trimmed.contains("-->") {
                    in_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("<!--") {
                in_comment = !trimmed.contains("-->");
                continue;
            }
            for id in extract_dep_ids(line) {
                if !tree_ids.contains(&id) {
                    report.add(
                        &plan.path,
                        Severity::Warning,
                        "W002",
                        format!("Cross-module reference '{id}' not found in plan tree"),
                        Some(header_line + 1 + offset),
                    );
                }
            }
        }
    }
}

/// W017: active module missing or stale `**Last reviewed:**` field.
/// Threshold configurable via APS_STALE_DAYS (default 60).
fn check_w017_last_reviewed(report: &mut LintReport, plan: &PlanFile) {
    let status = plan.status().unwrap_or_default().to_lowercase();
    if !(status.starts_with("ready") || status.starts_with("in progress")) {
        return;
    }

    let reviewed = plan.lines.iter().find_map(|line| {
        let rest = line.strip_prefix("**Last reviewed:**")?;
        let date = rest.trim_start_matches(' ');
        let date = date.get(..10)?;
        parse_civil_date(date).map(|epoch_days| (date.to_string(), epoch_days))
    });

    let Some((date, reviewed_days)) = reviewed else {
        report.add(
            &plan.path,
            Severity::Warning,
            "W017",
            "Active module has no **Last reviewed:** field",
            None,
        );
        return;
    };

    let stale_days: i64 = std::env::var("APS_STALE_DAYS")
        .ok()
        .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    let age_days = today_civil_days() - reviewed_days;
    if age_days > stale_days {
        let line = plan
            .lines
            .iter()
            .position(|l| l.starts_with("**Last reviewed:**"))
            .map(|index| index + 1);
        report.add(
            &plan.path,
            Severity::Warning,
            "W017",
            format!("Last reviewed {date} is {age_days} days old (threshold: {stale_days})"),
            line,
        );
    }
}

/// Today as civil days since the epoch, in the *local* timezone — bash's
/// `date -d "$reviewed" +%s` anchors at local midnight, so its W017 age is
/// the local civil-day difference. std has no timezone access, so ask
/// `date` like bash does; fall back to UTC when unavailable.
fn today_civil_days() -> i64 {
    let local = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| parse_civil_date(String::from_utf8_lossy(&out.stdout).trim()));

    local.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_secs() / 86_400) as i64)
            .unwrap_or(0)
    })
}

/// Days since the Unix epoch for a `YYYY-MM-DD` date (Howard Hinnant's
/// civil-date algorithm). Returns None for malformed dates.
fn parse_civil_date(date: &str) -> Option<i64> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let y: i64 = date[..4].parse().ok()?;
    let m: i64 = date[5..7].parse().ok()?;
    let d: i64 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn lint_work_items(report: &mut LintReport, plan: &PlanFile, tree_ids: &HashSet<String>) {
    // Uppercase-only IDs for the dependency known-set, matching
    // `grep -oE '^### [A-Z]+-[0-9]+:'`.
    let all_ids: Vec<String> = plan
        .work_items()
        .iter()
        .filter_map(|item| {
            let id = parser::parse_work_item_id(&item.header)?;
            id.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
                .then(|| id.to_string())
        })
        .collect();

    // Module status gates W018 (terminal modules are exempt archives).
    let module_status = plan.status().unwrap_or_default();

    for item in plan.work_items() {
        check_w001_id_format(report, plan, &item.header, item.line);
        check_e005_required_fields(report, plan, &item.header, item.line);
        check_w003_dependencies(report, plan, item.line, &all_ids, tree_ids);
        check_w018_terminal_validation(report, plan, &item.header, item.line, &module_status);
    }
}

/// W018: terminal work item missing Validation in an active module —
/// completion that the audit cannot verify. Warning only.
fn check_w018_terminal_validation(
    report: &mut LintReport,
    plan: &PlanFile,
    header: &str,
    line: usize,
    module_status: &str,
) {
    // Skip when the whole module is terminal (archive compaction is sanctioned).
    let module_lower = module_status.to_lowercase();
    if [
        "done", "complete", "merged", "released", "shipped", "archived",
    ]
    .iter()
    .any(|word| module_lower.starts_with(word))
    {
        return;
    }

    let content = plan.item_content(line);

    // An explicit Status field is authoritative; the header suffix only
    // counts when no field is present.
    let status = content
        .iter()
        .find_map(|l| l.strip_prefix("- **Status:**"))
        .map(str::trim_start);
    let terminal = match status {
        Some(value) if !value.is_empty() => is_terminal_status(value),
        _ => header_has_terminal_suffix(header),
    };
    if !terminal {
        return;
    }

    if !content.iter().any(|l| l.starts_with("- **Validation")) {
        report.add(
            &plan.path,
            Severity::Warning,
            "W018",
            format!("{header}: Complete item has no Validation — completion cannot be audited"),
            Some(line),
        );
    }
}

/// `(—|--) *(done|complete|merged|released|shipped)\b` (case-insensitive).
fn header_has_terminal_suffix(header: &str) -> bool {
    let lower = header.to_lowercase();
    let mut search = lower.as_str();
    loop {
        let (at, dash_len) = match (search.find('—'), search.find("--")) {
            (Some(em), Some(da)) if em <= da => (em, '—'.len_utf8()),
            (Some(em), None) => (em, '—'.len_utf8()),
            (_, Some(da)) => (da, 2),
            (None, None) => return false,
        };
        let rest = search[at + dash_len..].trim_start_matches(' ');
        if ["done", "complete", "merged", "released", "shipped"]
            .iter()
            .any(|word| {
                rest.starts_with(word)
                    && !rest[word.len()..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            })
        {
            return true;
        }
        search = &search[at + dash_len..];
    }
}

fn check_w001_id_format(report: &mut LintReport, plan: &PlanFile, header: &str, line: usize) {
    // `sed 's/^### \([A-Za-z0-9-]*\):.*/\1/'` then `^[A-Z]+-[0-9]{3}$`.
    let id = header
        .strip_prefix("### ")
        .and_then(|rest| rest.split(':').next())
        .unwrap_or(header);

    let valid = id.split_once('-').is_some_and(|(prefix, digits)| {
        !prefix.is_empty()
            && prefix.chars().all(|c| c.is_ascii_uppercase())
            && digits.len() == 3
            && digits.chars().all(|c| c.is_ascii_digit())
    });

    if !valid {
        report.add(
            &plan.path,
            Severity::Warning,
            "W001",
            format!("Work item ID '{id}' should match pattern PREFIX-NNN (e.g., AUTH-001)"),
            Some(line),
        );
    }
}

fn check_e005_required_fields(report: &mut LintReport, plan: &PlanFile, header: &str, line: usize) {
    let content = plan.item_content(line);

    // Terminal items are exempt: `- **Status:** <done|complete|merged|released|shipped>\b`.
    let status = content
        .iter()
        .find_map(|l| l.strip_prefix("- **Status:**"))
        .map(str::trim_start)
        .unwrap_or("");
    if is_terminal_status(status) {
        return;
    }

    for (field, label) in [
        ("- **Intent:**", "**Intent:**"),
        ("- **Expected Outcome:**", "**Expected Outcome:**"),
        ("- **Validation:**", "**Validation:**"),
    ] {
        if !content.iter().any(|l| l.starts_with(field)) {
            report.add(
                &plan.path,
                Severity::Error,
                "E005",
                format!("{header}: Missing {label} field"),
                Some(line),
            );
        }
    }
}

fn is_terminal_status(status: &str) -> bool {
    let lower = status.to_lowercase();
    ["done", "complete", "merged", "released", "shipped"]
        .iter()
        .any(|word| {
            lower.starts_with(word)
                && !lower[word.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

fn check_w003_dependencies(
    report: &mut LintReport,
    plan: &PlanFile,
    item_line: usize,
    all_ids: &[String],
    tree_ids: &HashSet<String>,
) {
    // First `- **Dependencies:**` line after the header, before the next heading.
    let deps_line = plan.lines[item_line..]
        .iter()
        .take_while(|l| !l.starts_with("## ") && !l.starts_with("### "))
        .find(|l| l.starts_with("- **Dependencies:**"));
    let Some(deps_line) = deps_line else {
        return;
    };

    // `grep -oE '[A-Z]+-[0-9]{3}'` over the single line. Resolve in-file
    // first, then against the plan-tree index (cross-module dependencies
    // and decision references are legitimate).
    for dep_id in extract_dep_ids(deps_line) {
        if !all_ids.iter().any(|id| id == &dep_id) && !tree_ids.contains(&dep_id) {
            // Line of the first `Dependencies:.*<id>` match in the whole file.
            let line_num = plan
                .lines
                .iter()
                .position(|l| {
                    l.find("Dependencies:")
                        .is_some_and(|at| l[at..].contains(&dep_id))
                })
                .map(|index| index + 1);
            report.add(
                &plan.path,
                Severity::Warning,
                "W003",
                format!("Dependency '{dep_id}' not found in plan"),
                line_num,
            );
        }
    }
}

/// `grep -oE '[A-Z]+-[0-9]{3}'` — uppercase run + dash + exactly 3 digits
/// (longest digit run must be 3; grep would match the first 3 of 4+ digits,
/// mirror that).
fn extract_dep_ids(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut ids = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '-' {
                let digits_start = i + 1;
                let mut j = digits_start;
                while j < chars.len() && chars[j].is_ascii_digit() && j - digits_start < 3 {
                    j += 1;
                }
                if j - digits_start == 3 {
                    ids.push(chars[start..j].iter().collect());
                    i = j;
                }
            }
            continue;
        }
        i += 1;
    }
    ids
}

// --- Issues rules -----------------------------------------------------------------

fn lint_issues(report: &mut LintReport, plan: &PlanFile) {
    if !plan.has_section("## Issues") {
        report.add(
            &plan.path,
            Severity::Error,
            "E010",
            "Missing ## Issues section",
            None,
        );
    }
    if !plan.has_section("## Questions") {
        report.add(
            &plan.path,
            Severity::Error,
            "E011",
            "Missing ## Questions section",
            None,
        );
    }

    if plan.has_section("## Issues") {
        check_tracker_fields(
            report,
            plan,
            "## Issues",
            "ISS-",
            "W010",
            &["Status", "Discovered", "Severity"],
        );
        check_id_format(
            report,
            plan,
            "ISS-",
            "W012",
            "Issue ID should be ISS-NNN format (e.g., ISS-001)",
            "Issue ID prefix must be uppercase ISS- (found wrong casing)",
        );
    }
    if plan.has_section("## Questions") {
        check_tracker_fields(
            report,
            plan,
            "## Questions",
            "Q-",
            "W011",
            &["Status", "Discovered", "Priority"],
        );
        check_id_format(
            report,
            plan,
            "Q-",
            "W013",
            "Question ID should be Q-NNN format (e.g., Q-001)",
            "Question ID prefix must be uppercase Q- (found wrong casing)",
        );
    }
}

/// W010/W011 — note: line numbers are relative to the section content,
/// not the file (bash greps the extracted section text).
fn check_tracker_fields(
    report: &mut LintReport,
    plan: &PlanFile,
    section: &str,
    id_prefix: &str,
    code: &'static str,
    fields: &[&str],
) {
    let content = plan.section_content(section);
    let header_pattern = |line: &str| -> Option<String> {
        let rest = line.strip_prefix("### ")?;
        let rest = rest.strip_prefix(id_prefix)?;
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 3 && rest[digits.len()..].starts_with(':') {
            Some(format!("{id_prefix}{digits}"))
        } else {
            None
        }
    };

    for (index, line) in content.iter().enumerate() {
        let Some(id) = header_pattern(line) else {
            continue;
        };
        let section_line = index + 1; // 1-based within the section content
        let entry: Vec<&str> = content[index + 1..]
            .iter()
            .take_while(|l| !l.starts_with("## ") && !l.starts_with("### "))
            .copied()
            .collect();

        for field in fields {
            let has_field = entry.iter().any(|l| {
                l.strip_prefix('|')
                    .map(|rest| rest.trim_start_matches(' '))
                    .and_then(|rest| rest.strip_prefix(field))
                    .map(|rest| rest.trim_start_matches(' ').starts_with('|'))
                    .unwrap_or(false)
            });
            if !has_field {
                let detail = match *field {
                    "Discovered" => format!("{id}: Missing Discovered field (traceability)"),
                    other => format!("{id}: Missing {other} field in metadata table"),
                };
                report.add(
                    &plan.path,
                    Severity::Warning,
                    code,
                    detail,
                    Some(section_line),
                );
            }
        }
    }
}

/// W012/W013 — headers with the right prefix but wrong format, plus
/// wrong-case prefixes. Line numbers are file lines here.
fn check_id_format(
    report: &mut LintReport,
    plan: &PlanFile,
    prefix: &str,
    code: &'static str,
    format_message: &str,
    case_message: &str,
) {
    for (index, line) in plan.lines.iter().enumerate() {
        let header_prefix = format!("### {prefix}");
        if line.starts_with(&header_prefix) {
            let rest = &line[header_prefix.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            let well_formed = digits.len() == 3 && rest[digits.len()..].starts_with(':');
            if !well_formed {
                report.add(
                    &plan.path,
                    Severity::Warning,
                    code,
                    format_message,
                    Some(index + 1),
                );
            }
        }
    }
    // Wrong-case prefixes (e.g. `### iss-`, `### q-`).
    for (index, line) in plan.lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let wanted = format!("### {}", prefix.to_lowercase());
        if lower.starts_with(&wanted) && !line.starts_with(&format!("### {prefix}")) {
            report.add(
                &plan.path,
                Severity::Warning,
                code,
                case_message,
                Some(index + 1),
            );
        }
    }
}

// --- Design rules ------------------------------------------------------------------

fn lint_design(report: &mut LintReport, plan: &PlanFile) {
    if !plan.has_section("## Problem") {
        report.add(
            &plan.path,
            Severity::Warning,
            "W014",
            "Missing ## Problem section",
            None,
        );
    }
    if !plan.has_section("## Design") {
        report.add(
            &plan.path,
            Severity::Warning,
            "W015",
            "Missing ## Design section",
            None,
        );
    }
    let head = plan.lines.iter().take(20);
    let has_field_header = head.clone().any(|l| table_row_starts_with(l, "Field"));
    let has_status_row = head.clone().any(|l| table_row_starts_with(l, "Status"));
    if !(has_field_header && has_status_row) {
        report.add(
            &plan.path,
            Severity::Warning,
            "W016",
            "Missing metadata table with Status field",
            None,
        );
    }
}

// --- Release rules -------------------------------------------------------------

/// Validate a release narrative under `plans/releases/` (REL-003). Release
/// files are versioned narratives, so the rules are structural: a versioned
/// filename, a header table that records the target and status, and the
/// Release Theme + What Ships sections that carry the narrative.
fn lint_release(report: &mut LintReport, plan: &PlanFile) {
    let basename = plan.path.rsplit('/').next().unwrap_or(&plan.path);
    if !is_release_filename(basename) {
        report.add(
            &plan.path,
            Severity::Error,
            "R001",
            "Release file must be named v<version>.md (e.g. v0.3.0.md)",
            None,
        );
    }

    // Header table: require the rows that drive the release — Target (which
    // version) and Status (where it is in the lifecycle).
    let head = plan.lines.iter().take(20);
    let has_target = head.clone().any(|l| table_row_starts_with(l, "Target"));
    let has_status = head.clone().any(|l| table_row_starts_with(l, "Status"));
    if !(has_target && has_status) {
        report.add(
            &plan.path,
            Severity::Error,
            "R002",
            "Missing release header table with Target and Status fields",
            None,
        );
    }

    if !plan.has_section("## Release Theme") {
        report.add(
            &plan.path,
            Severity::Error,
            "R003",
            "Missing ## Release Theme section",
            None,
        );
    }
    if !plan.has_section("## What Ships") {
        report.add(
            &plan.path,
            Severity::Error,
            "R004",
            "Missing ## What Ships section",
            None,
        );
    }
}

/// A release filename is `v<digit>…​.md` — `v0.3.0.md`, `v1.2.0-beta.md`.
fn is_release_filename(basename: &str) -> bool {
    let Some(stem) = basename.strip_suffix(".md") else {
        return false;
    };
    let mut chars = stem.chars();
    chars.next() == Some('v') && chars.next().is_some_and(|c| c.is_ascii_digit())
}

fn table_row_starts_with(line: &str, cell: &str) -> bool {
    line.strip_prefix('|')
        .map(|rest| rest.trim_start_matches(' '))
        .and_then(|rest| rest.strip_prefix(cell))
        .map(|rest| rest.trim_start_matches(' ').starts_with('|'))
        .unwrap_or(false)
}

// --- Output ------------------------------------------------------------------------

struct Palette {
    red: &'static str,
    yellow: &'static str,
    green: &'static str,
    gray: &'static str,
    bold: &'static str,
    reset: &'static str,
}

impl Palette {
    fn detect() -> Self {
        if std::io::stdout().is_terminal() {
            Self {
                red: "\x1b[0;31m",
                yellow: "\x1b[1;33m",
                green: "\x1b[0;32m",
                gray: "\x1b[0;90m",
                bold: "\x1b[1m",
                reset: "\x1b[0m",
            }
        } else {
            Self {
                red: "",
                yellow: "",
                green: "",
                gray: "",
                bold: "",
                reset: "",
            }
        }
    }
}

/// Text output, matching `print_text_results`.
pub fn render_text(report: &LintReport) -> String {
    let palette = Palette::detect();
    let mut out = String::new();

    for (path, _) in &report.files {
        out.push_str(&format!("{}{}{}\n", palette.bold, path, palette.reset));
        let findings: Vec<&Finding> = report.findings.iter().filter(|f| &f.path == path).collect();
        if findings.is_empty() {
            out.push_str(&format!("  {}✓{} valid\n", palette.green, palette.reset));
        } else {
            for finding in findings {
                let color = match finding.severity {
                    Severity::Error => palette.red,
                    Severity::Warning => palette.yellow,
                };
                let line_info = finding
                    .line
                    .map(|line| format!(" {}(line {line}){}", palette.gray, palette.reset))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {}{}:{} {}{}\n",
                    color, finding.code, palette.reset, finding.message, line_info
                ));
            }
        }
        out.push('\n');
    }

    let total_files = report.files.len();
    let errors = report.errors();
    let warnings = report.warnings();
    let mut summary = format!("{total_files} file");
    if total_files != 1 {
        summary.push('s');
    }
    summary.push_str(" checked");

    if errors == 0 && warnings == 0 {
        out.push_str(&format!(
            "{}{summary}, no issues{}\n",
            palette.green, palette.reset
        ));
    } else {
        let mut parts = Vec::new();
        if errors > 0 {
            let s = if errors != 1 { "s" } else { "" };
            parts.push(format!("{}{errors} error{s}{}", palette.red, palette.reset));
        }
        if warnings > 0 {
            let s = if warnings != 1 { "s" } else { "" };
            parts.push(format!(
                "{}{warnings} warning{s}{}",
                palette.yellow, palette.reset
            ));
        }
        out.push_str(&format!("{summary}, {}\n", parts.join(", ")));
    }

    out
}

/// JSON output, matching `print_json_results` byte-for-byte (single line,
/// strings unescaped exactly as bash interpolates them).
pub fn render_json(report: &LintReport) -> String {
    let mut json = String::from("{\"files\":[");

    for (index, (path, kind)) in report.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "{{\"path\":\"{path}\",\"type\":\"{}\",",
            kind.key()
        ));

        let entry = |finding: &Finding| {
            let mut item = format!(
                "{{\"code\":\"{}\",\"message\":\"{}\"",
                finding.code, finding.message
            );
            if let Some(line) = finding.line {
                item.push_str(&format!(",\"line\":{line}"));
            }
            item.push('}');
            item
        };

        let errors: Vec<String> = report
            .findings
            .iter()
            .filter(|f| &f.path == path && f.severity == Severity::Error)
            .map(entry)
            .collect();
        let warnings: Vec<String> = report
            .findings
            .iter()
            .filter(|f| &f.path == path && f.severity == Severity::Warning)
            .map(entry)
            .collect();

        json.push_str(&format!(
            "\"errors\":[{}],\"warnings\":[{}]}}",
            errors.join(","),
            warnings.join(",")
        ));
    }

    json.push_str(&format!(
        "],\"summary\":{{\"files\":{},\"errors\":{},\"warnings\":{}}}}}",
        report.files.len(),
        report.errors(),
        report.warnings()
    ));

    json
}

/// CLI entry. Returns the process exit code.
pub fn cmd_lint(target: &str, json: bool) -> i32 {
    match lint_target(target) {
        Ok(report) => {
            if json {
                println!("{}", render_json(&report));
            } else {
                print!("{}", render_text(&report));
            }
            if report.errors() > 0 { 1 } else { 0 }
        }
        Err(message) => {
            eprintln!("error: {message}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint_text(path: &str, text: &str) -> LintReport {
        let mut report = LintReport::default();
        let plan = PlanFile::from_text(path, text);
        report
            .files
            .push((path.to_string(), parser::file_type(path)));
        let tree_ids = HashSet::new();
        match parser::file_type(path) {
            FileType::Index => lint_index(&mut report, &plan),
            FileType::Issues => lint_issues(&mut report, &plan),
            FileType::Design => lint_design(&mut report, &plan),
            FileType::Release => lint_release(&mut report, &plan),
            _ => lint_module(&mut report, &plan, &tree_ids),
        }
        report
    }

    fn codes(report: &LintReport) -> Vec<&'static str> {
        report.findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn module_missing_sections_yield_errors() {
        let report = lint_text("plans/modules/x.aps.md", "# X\n\nNothing here.\n");
        assert_eq!(codes(&report), vec!["E001", "E002", "E003"]);
    }

    #[test]
    fn complete_items_skip_e005() {
        let text = "\
| ID | Status |\n| --- | --- |\n| X | Ready |\n\n## Purpose\n\ntext\n\n## Work Items\n\n### X-001: Done thing\n\n- **Status:** Complete 2026-01-01\n\n### X-002: Open thing\n\n- **Status:** Ready\n";
        let report = lint_text("plans/modules/x.aps.md", text);
        let e005: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.code == "E005")
            .collect();
        assert_eq!(e005.len(), 3, "only X-002 misses all three fields");
        assert!(e005.iter().all(|f| f.message.contains("X-002")));
    }

    #[test]
    fn w003_flags_unknown_dependencies() {
        let text = "\
| ID | Status |\n| --- | --- |\n| X | Ready |\n\n## Purpose\n\ntext\n\n## Work Items\n\n### X-001: Thing\n\n- **Intent:** i\n- **Expected Outcome:** o\n- **Validation:** v\n- **Dependencies:** X-999\n";
        let report = lint_text("plans/modules/x.aps.md", text);
        assert!(codes(&report).contains(&"W003"));
        let w003 = report.findings.iter().find(|f| f.code == "W003").unwrap();
        assert!(w003.message.contains("X-999"));
    }

    #[test]
    fn w002_flags_conductor_typo_refs_but_not_valid_ones() {
        let text = "\
| ID | Type | Status |\n| --- | --------- | -------- |\n| REL | Conductor | Complete |\n\n\
## Purpose\n\ntext\n\n\
## Coordinated Modules\n\n\
| Module | Role | Status |\n| --- | --- | --- |\n| [install](./install.aps.md) | wires scaffold | Done |\n\n\
## Cross-Module Work Items\n\n\
- [INSTALL-014](./install.aps.md) — real, resolves in tree\n\
- [INSTALL-999](./install.aps.md) — typo, resolves nowhere\n\n\
## Work Items\n\n### REL-001: Thing\n\n- **Intent:** i\n- **Expected Outcome:** o\n- **Validation:** v\n";
        let plan = PlanFile::from_text("plans/modules/release-planning.aps.md", text);
        assert!(plan.is_conductor());
        let mut tree = HashSet::new();
        tree.insert("INSTALL-014".to_string());
        let mut report = LintReport::default();
        check_w002_conductor_refs(&mut report, &plan, &tree);
        let w002: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.code == "W002")
            .collect();
        assert_eq!(
            w002.len(),
            1,
            "only INSTALL-999 is unresolved: {:?}",
            codes(&report)
        );
        assert!(w002[0].message.contains("INSTALL-999"));
    }

    #[test]
    fn w002_skipped_for_vertical_modules() {
        // A vertical module never carries these sections, and lint_module only
        // runs the conductor check when `is_conductor()`. An unmarked module
        // with a typo'd cross-ref in prose must not raise W002.
        let text = "\
| ID | Status |\n| --- | --- |\n| AUTH | Ready |\n\n\
## Purpose\n\nMentions FAKE-999 in passing.\n\n\
## Work Items\n\n### AUTH-001: Thing\n\n- **Intent:** i\n- **Expected Outcome:** o\n- **Validation:** v\n";
        let report = lint_text("plans/modules/auth.aps.md", text);
        assert!(
            !codes(&report).contains(&"W002"),
            "got {:?}",
            codes(&report)
        );
    }

    #[test]
    fn w006_flags_non_conductor_in_conductor_index_section() {
        use std::fs;
        let root = std::env::temp_dir().join("aps_w006_test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("modules")).unwrap();
        // One conductor module, one vertical module mislisted as conductor.
        fs::write(
            root.join("modules/release-planning.aps.md"),
            "# Rel\n\n| ID | Type | Status |\n| --- | --- | --- |\n| REL | Conductor | Complete |\n\n## Purpose\n\nx\n\n## Work Items\n\n### REL-001: a\n",
        )
        .unwrap();
        fs::write(
            root.join("modules/auth.aps.md"),
            "# Auth\n\n| ID | Status |\n| --- | --- |\n| AUTH | Ready |\n\n## Purpose\n\nx\n\n## Work Items\n\n### AUTH-001: a\n",
        )
        .unwrap();
        let index = root.join("index.aps.md");
        fs::write(
            &index,
            "# Plan\n\n## Modules\n\n### Conductor / Crosscutting (Adopted)\n\n| Module | Status |\n| --- | --- |\n| [release-planning](./modules/release-planning.aps.md) | Complete |\n| [auth](./modules/auth.aps.md) | Ready |\n",
        )
        .unwrap();

        let plan = PlanFile::load(&index.to_string_lossy()).unwrap();
        let mut report = LintReport::default();
        check_w006_conductor_index(&mut report, &plan);

        let w006: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.code == "W006")
            .collect();
        assert_eq!(
            w006.len(),
            1,
            "only auth is mislisted: {:?}",
            codes(&report)
        );
        assert!(w006[0].message.contains("auth.aps.md"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn index_requires_modules_section() {
        let report = lint_text("plans/index.aps.md", "# Plan\n\n## Overview\n\ntext\n");
        assert_eq!(codes(&report), vec!["E004"]);
    }

    #[test]
    fn issues_field_lines_are_section_relative() {
        let text = "\
# Issues\n\n## Issues\n\n### ISS-001: Broken\n\n| Status | Open |\n\n## Questions\n";
        let report = lint_text("plans/issues.md", text);
        let w010: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.code == "W010")
            .collect();
        // Missing Discovered + Severity; header is line 2 of the section content.
        assert_eq!(w010.len(), 2);
        assert!(w010.iter().all(|f| f.line == Some(2)));
    }

    const VALID_RELEASE: &str = "\
# Release Plan: v0.3.0\n\n\
| Field  | Value   |\n| ------ | ------- |\n\
| Target | v0.3.0  |\n| Status | Shipped |\n\n\
## Release Theme\n\n\
**Theme** — narrative.\n\n\
## What Ships\n\n\
| Area | Detail |\n| ---- | ------ |\n| x | y |\n";

    #[test]
    fn valid_release_file_passes() {
        let report = lint_text("plans/releases/v0.3.0.md", VALID_RELEASE);
        assert!(
            codes(&report).is_empty(),
            "unexpected: {:?}",
            codes(&report)
        );
    }

    #[test]
    fn malformed_release_flags_all_codes() {
        // Wrong name, no header table, no sections.
        let report = lint_text("plans/releases/notes.md", "# Notes\n\nnothing\n");
        let mut found = codes(&report);
        found.sort_unstable();
        assert_eq!(found, vec!["R001", "R002", "R003", "R004"]);
    }

    #[test]
    fn release_naming_is_the_only_flag_when_structure_is_sound() {
        // Valid body, but the file is not named v<version>.md.
        let report = lint_text("plans/releases/draft.md", VALID_RELEASE);
        assert_eq!(codes(&report), vec!["R001"]);
    }

    #[test]
    fn release_rules_are_errors_not_warnings() {
        // Valid v-name, empty body → exactly R002 + R003 + R004 (no R001).
        let report = lint_text("plans/releases/v1.0.0.md", "# Empty\n");
        assert_eq!(report.errors(), 3);
        assert_eq!(report.warnings(), 0);
    }

    #[test]
    fn release_filename_validation() {
        for ok in ["v0.3.0.md", "v1.2.0-beta.md", "v10.0.0.md"] {
            assert!(is_release_filename(ok), "{ok} should be valid");
        }
        for bad in [
            "vfoo.md",
            "v.md",
            "version-1.md",
            "V1.0.0.md",
            "notes.md",
            "v1.0.0",
        ] {
            assert!(!is_release_filename(bad), "{bad} should be invalid");
        }
    }

    #[test]
    fn render_text_matches_bash_shape() {
        let report = lint_text("plans/modules/x.aps.md", "# X\n");
        let text = render_text(&report);
        assert!(text.starts_with("plans/modules/x.aps.md\n"));
        assert!(text.contains("  E001: Missing ## Purpose section\n"));
        assert!(text.ends_with("1 file checked, 3 errors\n"));
    }

    #[test]
    fn render_json_matches_bash_shape() {
        let report = lint_text(
            "plans/index.aps.md",
            "## Modules\n\n| [a](x) | d | Ready |\n",
        );
        let json = render_json(&report);
        // W019 fires because the link target `x` does not exist on disk.
        assert_eq!(
            json,
            "{\"files\":[{\"path\":\"plans/index.aps.md\",\"type\":\"index\",\"errors\":[],\"warnings\":[{\"code\":\"W019\",\"message\":\"Module link target not found: x\",\"line\":3}]}],\"summary\":{\"files\":1,\"errors\":0,\"warnings\":1}}"
        );
    }
}

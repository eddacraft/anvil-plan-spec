//! Native `aps lint` (TUI-009, D-028).
//!
//! Implements the bash rule set from lib/rules/*.sh with the same E/W
//! codes, message text, output format, and exit behavior. Quirks of the
//! bash implementation (section-relative line numbers in W010/W011,
//! unescaped JSON strings) are preserved deliberately — the parity
//! contract is byte-identical output on the same input.

use std::io::IsTerminal;
use std::path::Path;

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

    let mut report = LintReport::default();
    for file in &files {
        lint_file(&mut report, file);
    }
    Ok(report)
}

fn lint_file(report: &mut LintReport, path: &str) {
    let kind = parser::file_type(path);
    report.files.push((path.to_string(), kind));

    let Ok(plan) = PlanFile::load(path) else {
        report.add(path, Severity::Error, "E000", "Cannot read file", None);
        return;
    };

    match kind {
        FileType::Index => lint_index(report, &plan),
        FileType::Module | FileType::Simple => lint_module(report, &plan),
        FileType::Issues => lint_issues(report, &plan),
        FileType::Design => lint_design(report, &plan),
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
    for section in ["## Overview", "## Problem & Success Criteria", "## Modules"] {
        check_empty_section(report, plan, section);
    }
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

fn lint_module(report: &mut LintReport, plan: &PlanFile) {
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

    if plan.has_section("## Work Items") {
        lint_work_items(report, plan);
    }
}

fn lint_work_items(report: &mut LintReport, plan: &PlanFile) {
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

    for item in plan.work_items() {
        check_w001_id_format(report, plan, &item.header, item.line);
        check_e005_required_fields(report, plan, &item.header, item.line);
        check_w003_dependencies(report, plan, item.line, &all_ids);
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
) {
    // First `- **Dependencies:**` line after the header, before the next heading.
    let deps_line = plan.lines[item_line..]
        .iter()
        .take_while(|l| !l.starts_with("## ") && !l.starts_with("### "))
        .find(|l| l.starts_with("- **Dependencies:**"));
    let Some(deps_line) = deps_line else {
        return;
    };

    // `grep -oE '[A-Z]+-[0-9]{3}'` over the single line.
    for dep_id in extract_dep_ids(deps_line) {
        if !all_ids.iter().any(|id| id == &dep_id) {
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
                format!("Dependency '{dep_id}' not found in this file"),
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
        match parser::file_type(path) {
            FileType::Index => lint_index(&mut report, &plan),
            FileType::Issues => lint_issues(&mut report, &plan),
            FileType::Design => lint_design(&mut report, &plan),
            _ => lint_module(&mut report, &plan),
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
        assert_eq!(
            json,
            "{\"files\":[{\"path\":\"plans/index.aps.md\",\"type\":\"index\",\"errors\":[],\"warnings\":[]}],\"summary\":{\"files\":1,\"errors\":0,\"warnings\":0}}"
        );
    }
}

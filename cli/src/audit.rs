//! Native `aps audit` (DOGFOOD-002).
//!
//! Ports `cmd_audit` from lib/audit.sh: verify recorded plan state against
//! reality. A001 overstated (Complete whose Validation fails), A002
//! understated (Draft whose Files exist), A003 stale (Ready in a module with
//! no recent review), A004 broken index links. Complete items without a
//! runnable Validation report PARTIAL, not a finding.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

use crate::date;
use crate::next::PlanGraph;
use crate::parser;

struct Finding {
    code: &'static str,
    item: String,
    module: String,
    detail: String,
}

struct Verification {
    id: String,
    result: &'static str,
    detail: String,
}

/// First backtick-quoted span of a Validation value, backticks stripped
/// (`audit_extract_command`). Prose validations yield None.
fn extract_command(validation: &str) -> Option<String> {
    let open = validation.find('`')?;
    let rest = &validation[open + 1..];
    let close = rest.find('`')?;
    Some(rest[..close].replace('\r', ""))
}

fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// `type -P first_word || [[ -f first_word && -x first_word ]]` — true when
/// the command resolves on PATH or is an executable file.
fn is_runnable(word: &str) -> bool {
    if !word.contains('/')
        && let Ok(path) = env::var("PATH")
    {
        for dir in path.split(':').filter(|d| !d.is_empty()) {
            if is_executable_file(&Path::new(dir).join(word)) {
                return true;
            }
        }
    }
    is_executable_file(Path::new(word))
}

/// Run a Validation command, returning its exit code. Uses `timeout` when
/// available, matching `audit_complete_item` (124 == timed out).
fn run_command(cmd: &str, timeout_secs: u32) -> i32 {
    let has_timeout = is_runnable("timeout")
        || env::var("PATH").is_ok_and(|p| {
            p.split(':')
                .any(|d| !d.is_empty() && is_executable_file(&Path::new(d).join("timeout")))
        });
    let status = if has_timeout {
        Command::new("timeout")
            .args(["-k", "5", &timeout_secs.to_string(), "bash", "-c", cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("bash")
            .args(["-c", cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
    status.ok().and_then(|s| s.code()).unwrap_or(1)
}

/// Last reviewed `YYYY-MM-DD` from a module file (`audit_last_reviewed`).
fn last_reviewed(file: &str) -> Option<String> {
    let text = fs::read_to_string(file).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("**Last reviewed:**") {
            let rest = rest.trim_start_matches(' ');
            if rest.len() >= 10 {
                let candidate = &rest[..10];
                if date::parse_civil_date(candidate).is_some() {
                    return Some(candidate.to_string());
                }
            }
        }
    }
    None
}

/// Expand a (possibly glob) path under `repo_root` and report whether any
/// match is a non-empty regular file (`compgen -G` + `-s`). Globs are
/// supported on the final path component (`src/foo/*.rs`); literal paths pass
/// through as an existence test.
fn has_substantive_file(repo_root: &Path, rel: &str) -> bool {
    let resolved = if rel.starts_with('/') {
        Path::new(rel).to_path_buf()
    } else {
        repo_root.join(rel)
    };

    let name = resolved
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    let is_glob = name.as_deref().is_some_and(|n| n.contains(['*', '?', '[']));

    if !is_glob {
        return fs::metadata(&resolved)
            .map(|m| m.is_file() && m.len() > 0)
            .unwrap_or(false);
    }

    let (Some(parent), Some(pattern)) = (resolved.parent(), name) else {
        return false;
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return false;
    };
    for entry in entries.flatten() {
        let fname = entry.file_name();
        if glob_match(&pattern, &fname.to_string_lossy())
            && entry
                .metadata()
                .map(|m| m.is_file() && m.len() > 0)
                .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Minimal shell-style wildcard match (`*`, `?`) for a single path component.
fn glob_match(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    let (mut pi, mut ni) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ni;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            // Drop remaining C0 control characters (illegal in JSON strings).
            c if (c as u32) < 0x20 => {}
            c => out.push(c),
        }
    }
    out
}

/// Scan an index's `## Modules` section for links to missing files
/// (`audit_index_links`, same contract as lint E012).
fn index_link_findings(plan_root: &Path) -> Vec<Finding> {
    let index_path = plan_root.join("index.aps.md");
    let Ok(text) = fs::read_to_string(&index_path) else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    let mut in_modules = false;
    for (i, line) in text.lines().enumerate() {
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
        for target in extract_link_targets(line) {
            // Skip pure anchors and any URI scheme (http, mailto, vscode, …).
            if target.starts_with('#') || has_scheme(&target) {
                continue;
            }
            let target = target.split('#').next().unwrap_or("").to_string();
            if target.is_empty() {
                continue;
            }
            if !plan_root.join(&target).exists() {
                findings.push(Finding {
                    code: "A004",
                    item: "index".to_string(),
                    module: "-".to_string(),
                    detail: format!("broken-link: {target} (line {})", i + 1),
                });
            }
        }
    }
    findings
}

/// All `](target)` link targets in a line, with markdown link titles stripped.
fn extract_link_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']'
            && bytes[i + 1] == b'('
            && let Some(rel) = line[i + 2..].find(')')
        {
            let mut target = line[i + 2..i + 2 + rel].to_string();
            strip_link_title(&mut target);
            targets.push(target);
            i = i + 2 + rel + 1;
            continue;
        }
        i += 1;
    }
    targets
}

/// Remove a trailing `[ \t]+["']…` markdown link title.
fn strip_link_title(target: &mut String) {
    let chars: Vec<char> = target.chars().collect();
    for (idx, &c) in chars.iter().enumerate() {
        if (c == '"' || c == '\'') && idx > 0 && chars[idx - 1].is_whitespace() {
            // Cut from the start of the whitespace run before the quote.
            let mut start = idx;
            while start > 0 && chars[start - 1].is_whitespace() {
                start -= 1;
            }
            *target = chars[..start].iter().collect();
            return;
        }
    }
}

/// `^[A-Za-z][A-Za-z0-9+.-]*:` — a URI scheme prefix.
fn has_scheme(target: &str) -> bool {
    let bytes = target.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return false;
    }
    for &b in &bytes[1..] {
        if b == b':' {
            return true;
        }
        if !(b.is_ascii_alphanumeric() || b == b'+' || b == b'.' || b == b'-') {
            return false;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn audit_complete_item(
    graph: &PlanGraph,
    idx: usize,
    run_validation: bool,
    timeout_secs: u32,
    verifications: &mut Vec<Verification>,
    findings: &mut Vec<Finding>,
) {
    let item = &graph.items[idx];
    let plan = match crate::parser::PlanFile::load(&item.file) {
        Ok(plan) => plan,
        Err(_) => return,
    };
    let content = plan.item_content(item.line);
    let validation = parser::field_value(&content, "Validation");

    if validation.is_empty() {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "PARTIAL",
            detail: "no Validation field".to_string(),
        });
        return;
    }

    let Some(cmd) = extract_command(&validation) else {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "PARTIAL",
            detail: "Validation is not a runnable command".to_string(),
        });
        return;
    };

    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if !is_runnable(first_word) {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "PARTIAL",
            detail: format!("command not found: {first_word}"),
        });
        return;
    }

    if !run_validation {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "PARTIAL",
            detail: "not run (--no-run)".to_string(),
        });
        return;
    }

    let rc = run_command(&cmd, timeout_secs);
    if rc == 0 {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "PASS",
            detail: cmd,
        });
    } else if rc == 124 {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "FAIL",
            detail: format!("timed out after {timeout_secs}s: {cmd}"),
        });
        findings.push(Finding {
            code: "A001",
            item: item.id.clone(),
            module: item.module.clone(),
            detail: format!("overstated: Validation timed out: {cmd}"),
        });
    } else {
        verifications.push(Verification {
            id: item.id.clone(),
            result: "FAIL",
            detail: cmd.clone(),
        });
        findings.push(Finding {
            code: "A001",
            item: item.id.clone(),
            module: item.module.clone(),
            detail: format!("overstated: Validation failed: {cmd}"),
        });
    }
}

fn audit_draft_item(graph: &PlanGraph, idx: usize, repo_root: &Path, findings: &mut Vec<Finding>) {
    let item = &graph.items[idx];
    let Ok(plan) = crate::parser::PlanFile::load(&item.file) else {
        return;
    };
    let content = plan.item_content(item.line);
    let files_field = parser::field_value(&content, "Files");
    if files_field.is_empty() {
        return;
    }

    let mut existing: Vec<String> = Vec::new();
    for raw in files_field.split(',') {
        let path = raw.trim().trim_start_matches("- ").trim();
        if path.is_empty() {
            continue;
        }
        if has_substantive_file(repo_root, path) {
            existing.push(path.to_string());
        }
    }

    if !existing.is_empty() {
        findings.push(Finding {
            code: "A002",
            item: item.id.clone(),
            module: item.module.clone(),
            detail: format!(
                "understated: Draft but files exist: {}",
                existing.join(", ")
            ),
        });
    }
}

fn audit_ready_item(graph: &PlanGraph, idx: usize, stale_days: i64, findings: &mut Vec<Finding>) {
    let item = &graph.items[idx];
    match graph.module_status(&item.module, &item.child) {
        "Ready" | "In Progress" => {}
        _ => return,
    }

    let Some(reviewed) = last_reviewed(&item.file) else {
        findings.push(Finding {
            code: "A003",
            item: item.id.clone(),
            module: item.module.clone(),
            detail: "stale: Ready item in module with no Last reviewed field".to_string(),
        });
        return;
    };

    if let Some(reviewed_days) = date::parse_civil_date(&reviewed) {
        let age = date::today_civil_days() - reviewed_days;
        if age > stale_days {
            findings.push(Finding {
                code: "A003",
                item: item.id.clone(),
                module: item.module.clone(),
                detail: format!(
                    "stale: module last reviewed {reviewed} ({age} days ago, threshold {stale_days})"
                ),
            });
        }
    }
}

fn print_text(audited: usize, verifications: &[Verification], findings: &[Finding]) {
    println!("APS Audit");
    println!();

    if !verifications.is_empty() {
        println!("Complete-item verification:");
        for v in verifications {
            println!("  {:<12} {:<8} {}", v.id, v.result, v.detail);
        }
        println!();
    }

    if findings.is_empty() {
        println!("Findings: none ({audited} items audited)");
        return;
    }

    println!("Findings:");
    for f in findings {
        println!("  {}  {:<12} {}", f.code, f.item, f.detail);
    }
    println!();
    println!("Findings: {} ({audited} items audited)", findings.len());
}

fn print_json(audited: usize, verifications: &[Verification], findings: &[Finding]) {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"summary\": {\n");
    out.push_str(&format!("    \"items_audited\": {audited},\n"));
    out.push_str(&format!("    \"findings\": {}\n", findings.len()));
    out.push_str("  },\n");

    out.push_str("  \"verifications\": [\n");
    for (i, v) in verifications.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"item\": \"{}\", \"result\": \"{}\", \"detail\": \"{}\"}}{}\n",
            json_escape(&v.id),
            json_escape(v.result),
            json_escape(&v.detail),
            if i + 1 < verifications.len() { "," } else { "" }
        ));
    }
    out.push_str("  ],\n");

    out.push_str("  \"findings\": [\n");
    for (i, f) in findings.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"code\": \"{}\", \"item\": \"{}\", \"module\": \"{}\", \"detail\": \"{}\"}}{}\n",
            f.code,
            json_escape(&f.item),
            json_escape(&f.module),
            json_escape(&f.detail),
            if i + 1 < findings.len() { "," } else { "" }
        ));
    }
    out.push_str("  ]\n");
    out.push('}');
    println!("{out}");
}

/// `aps audit [module]` entry. Returns the process exit code (1 on findings).
pub fn cmd_audit(
    plan_root: &str,
    module_filter: &str,
    child_scope: &str,
    json: bool,
    no_run: bool,
    stale_days_flag: Option<u32>,
) -> i32 {
    let root = Path::new(plan_root);
    if !root.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }

    // Flag wins; otherwise APS_STALE_DAYS, then 60 — degrade a bad env value.
    let stale_days: i64 = match stale_days_flag {
        Some(n) => n as i64,
        None => match env::var("APS_STALE_DAYS") {
            Ok(v) if v.chars().all(|c| c.is_ascii_digit()) && !v.is_empty() => {
                v.parse().unwrap_or(60)
            }
            Ok(_) => {
                eprintln!("warning: APS_STALE_DAYS must be a number; using 60");
                60
            }
            Err(_) => 60,
        },
    };
    let timeout_secs: u32 = match env::var("APS_AUDIT_TIMEOUT") {
        Ok(v) if v.chars().all(|c| c.is_ascii_digit()) && !v.is_empty() => v.parse().unwrap_or(60),
        Ok(_) => {
            eprintln!("warning: APS_AUDIT_TIMEOUT must be a number; using 60");
            60
        }
        Err(_) => 60,
    };

    let run_validation = !no_run;
    if run_validation {
        eprintln!("warning: executing Validation commands from plan files (use --no-run to skip)");
    }

    let graph = match PlanGraph::load_items_only(root) {
        Ok(graph) => graph,
        Err(_) => {
            eprintln!("error: No modules directory under: {plan_root}");
            return 1;
        }
    };

    let repo_root = root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    let mut verifications: Vec<Verification> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();
    let mut audited = 0usize;

    for idx in 0..graph.items.len() {
        if !graph.matches_child(&graph.items[idx], child_scope) {
            continue;
        }
        if !graph.matches_module(&graph.items[idx], module_filter) {
            continue;
        }
        match graph.items[idx].status.as_str() {
            "Complete" => {
                audit_complete_item(
                    &graph,
                    idx,
                    run_validation,
                    timeout_secs,
                    &mut verifications,
                    &mut findings,
                );
                audited += 1;
            }
            "Draft" => {
                audit_draft_item(&graph, idx, &repo_root, &mut findings);
                audited += 1;
            }
            "Ready" => {
                audit_ready_item(&graph, idx, stale_days, &mut findings);
                audited += 1;
            }
            _ => {}
        }
    }

    // Index-link integrity: skip when scoped to a module or child; otherwise
    // check every plan root across the federation (MONO-003).
    if module_filter.is_empty() && child_scope.is_empty() {
        for fed_root in crate::next::plan_roots(root) {
            findings.extend(index_link_findings(&fed_root));
        }
    }

    if json {
        print_json(audited, &verifications, &findings);
    } else {
        print_text(audited, &verifications, &findings);
    }

    if findings.is_empty() { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> String {
        "../test/fixtures/audit/plans".to_string()
    }

    #[test]
    fn glob_matches_basic_patterns() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("a?c", "abc"));
        assert!(!glob_match("*.rs", "main.txt"));
        assert!(glob_match("*", "anything"));
    }

    #[test]
    fn scheme_and_anchor_detection() {
        assert!(has_scheme("vscode://file/x"));
        assert!(has_scheme("https://example.com"));
        assert!(!has_scheme("./modules/demo.aps.md"));
        assert!(!has_scheme("plain"));
    }

    #[test]
    fn strips_markdown_link_title() {
        let mut t = "./modules/demo.aps.md \"Titled link\"".to_string();
        strip_link_title(&mut t);
        assert_eq!(t, "./modules/demo.aps.md");
    }

    #[test]
    fn extract_command_finds_first_backtick_span() {
        assert_eq!(extract_command("`true`").as_deref(), Some("true"));
        assert_eq!(
            extract_command("Inspect `src/x/` here").as_deref(),
            Some("src/x/")
        );
        assert_eq!(extract_command("Manual verification"), None);
    }

    #[test]
    fn audit_no_run_reports_all_classes_without_executing() {
        // --no-run: A002/A003/A004 still fire; A001 does not (no execution).
        let code = cmd_audit(&fixture(), "", "", false, true, None);
        assert_eq!(code, 1);
    }

    #[test]
    fn audit_module_scope_suppresses_index_links() {
        // Scoped to demo: A004 (index links) is skipped, but items still audit.
        let code = cmd_audit(&fixture(), "demo", "", false, true, None);
        assert_eq!(code, 1);
    }
}

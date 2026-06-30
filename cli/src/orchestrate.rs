//! Native `aps start` / `aps complete` / `aps graph` (orchestration).
//!
//! Ports `cmd_start`, `cmd_complete`, `cmd_graph` and their helpers from
//! lib/orchestrate.sh. The status/learning rewrite mirrors the awk in
//! `orch_rewrite_work_item` line for line, and the context package matches
//! `orch_context_package` section for section — same output on the same input.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::date;
use crate::next::{PlanGraph, WorkItem, deps_display};
use crate::parser::{self, PlanFile};

/// What `rewrite_work_item` edits inside the target item's block.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RewriteMode {
    /// Replace (or insert) the `- **Status:**` line.
    Status,
    /// Insert a `- **Learning:** "..."` line after Validation.
    Learning,
}

/// `^- \*\*[A-Za-z][^*]*:\*\*` — a `- **Field:**` metadata line.
fn is_meta_line(line: &str) -> bool {
    let Some(body) = line.strip_prefix("- **") else {
        return false;
    };
    let Some(idx) = body.find(":**") else {
        return false;
    };
    let name = &body[..idx];
    !name.is_empty() && name.as_bytes()[0].is_ascii_alphabetic() && !name.contains('*')
}

/// `^[[:space:]]+[^[:space:]]` — an indented continuation line.
fn is_continuation(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.len() != line.len() && !trimmed.is_empty()
}

/// Rewrite the buffered lines of a single work item (`flush_target`).
fn flush_target(buffer: &[String], mode: RewriteMode, value: &str) -> Vec<String> {
    let mut status_idx = None;
    let mut validation_idx = None;
    let mut last_meta: isize = -1;
    for (i, line) in buffer.iter().enumerate() {
        if line.starts_with("- **Status:**") {
            status_idx = Some(i);
        }
        if line.starts_with("- **Validation:**") {
            validation_idx = Some(i);
        }
        // A meta line, or an indented continuation of the last meta, extends
        // the metadata block (`last_meta`).
        if is_meta_line(line) || (is_continuation(line) && last_meta >= 0) {
            last_meta = i as isize;
        }
    }

    match mode {
        RewriteMode::Status => {
            let status_line = format!("- **Status:** {value}");
            if let Some(si) = status_idx {
                let mut out = buffer.to_vec();
                out[si] = status_line;
                return out;
            }
            let last_meta = if last_meta < 0 {
                buffer.len() as isize - 1
            } else {
                last_meta
            } as usize;
            let mut out: Vec<String> = buffer[..=last_meta].to_vec();
            out.push(status_line);
            out.extend(buffer[last_meta + 1..].iter().cloned());
            out
        }
        RewriteMode::Learning => {
            let learning_line = format!("- **Learning:** \"{value}\"");
            let insert_idx = if let Some(vi) = validation_idx {
                let mut ii = vi;
                while ii + 1 < buffer.len() && is_continuation(&buffer[ii + 1]) {
                    ii += 1;
                }
                ii
            } else if last_meta >= 0 {
                last_meta as usize
            } else {
                buffer.len() - 1
            };
            let mut out: Vec<String> = buffer[..=insert_idx].to_vec();
            out.push(learning_line);
            out.extend(buffer[insert_idx + 1..].iter().cloned());
            out
        }
    }
}

/// True when `line` opens the target item's block (`^### <id>:`).
fn is_target_header(line: &str, id: &str) -> bool {
    line.strip_prefix("### ")
        .is_some_and(|rest| rest.starts_with(&format!("{id}:")))
}

/// Rewrite a work item's Status or Learning in place
/// (`orch_rewrite_work_item`). Byte-preserving except for the edited line.
pub fn rewrite_work_item(
    path: &str,
    id: &str,
    mode: RewriteMode,
    value: &str,
) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("Cannot rewrite: file not found: {path} ({err})"))?;
    let lines: Vec<String> = text.split('\n').map(str::to_string).collect();

    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 1);
    let mut buffer: Vec<String> = Vec::new();
    let mut in_target = false;

    for line in lines {
        if line.starts_with("### ") {
            if in_target {
                out.extend(flush_target(&buffer, mode, value));
                buffer.clear();
            }
            if is_target_header(&line, id) {
                in_target = true;
                buffer.clear();
                buffer.push(line);
            } else {
                in_target = false;
                out.push(line);
            }
            continue;
        }
        if in_target && line.starts_with("## ") {
            out.extend(flush_target(&buffer, mode, value));
            buffer.clear();
            in_target = false;
            out.push(line);
            continue;
        }
        if in_target {
            buffer.push(line);
        } else {
            out.push(line);
        }
    }
    if in_target {
        out.extend(flush_target(&buffer, mode, value));
    }

    fs::write(path, out.join("\n"))
        .map_err(|err| format!("Rewrite failed for {id} in {path}: {err}"))
}

/// `<plan_root>/../.aps/context` (`orch_context_root`).
fn context_root(plan_root: &Path) -> PathBuf {
    let parent = plan_root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    parent.join(".aps/context")
}

/// Write `.aps/context/<id>.md` for a work item and return its path
/// (`orch_context_package`).
fn write_context_package(
    graph: &PlanGraph,
    item: &WorkItem,
    plan_root: &Path,
) -> Result<PathBuf, String> {
    let context_dir = context_root(plan_root);
    fs::create_dir_all(&context_dir).map_err(|err| {
        format!(
            "Cannot create context directory: {} ({err})",
            context_dir.display()
        )
    })?;
    let context_file = context_dir.join(format!("{}.md", item.id));

    // Read the item's module file fresh: `start` rewrites Status first, so the
    // Work Item section reflects the new "In Progress" state, matching bash.
    let plan =
        PlanFile::load(&item.file).map_err(|err| format!("Cannot read {}: {err}", item.file))?;
    let content = plan.item_content(item.line);
    let related = parser::field_value(&content, "Files");

    let mut out: Vec<String> = Vec::new();
    out.push(format!("# Context: {} - {}", item.id, item.title));
    out.push(String::new());
    out.push("## Work Item".to_string());
    out.extend(content.iter().map(|s| s.to_string()));
    out.push(String::new());
    out.push("## Module Scope".to_string());
    out.extend(plan.emit_section("Purpose").iter().map(|s| s.to_string()));
    out.push(String::new());
    out.extend(plan.emit_section("In Scope").iter().map(|s| s.to_string()));
    out.push(String::new());
    out.extend(
        plan.emit_section("Out of Scope")
            .iter()
            .map(|s| s.to_string()),
    );
    out.push(String::new());
    out.extend(
        plan.emit_section("Interfaces")
            .iter()
            .map(|s| s.to_string()),
    );
    out.push(String::new());
    out.push("## Decisions".to_string());
    out.extend(plan.emit_section("Decisions").iter().map(|s| s.to_string()));
    out.push(String::new());
    out.push("## Dependency Learnings".to_string());

    let mut found_learning = false;
    for dep in parser::dep_tokens(&item.deps) {
        let Some(dep_item) = graph.find(&dep) else {
            continue;
        };
        let Ok(dep_plan) = PlanFile::load(&dep_item.file) else {
            continue;
        };
        let dep_content = dep_plan.item_content(dep_item.line);
        let learning = parser::field_value(&dep_content, "Learning");
        if !learning.is_empty() {
            out.push(format!("- {dep}: {learning}"));
            found_learning = true;
        }
    }
    if !found_learning {
        out.push("- None".to_string());
    }

    out.push(String::new());
    out.push("## Related Files".to_string());
    if related.is_empty() {
        out.push("- None specified".to_string());
    } else {
        for line in related.lines() {
            out.push(format!("- {line}"));
        }
    }

    let mut body = out.join("\n");
    body.push('\n');
    fs::write(&context_file, body).map_err(|err| {
        format!(
            "Cannot write context package {}: {err}",
            context_file.display()
        )
    })?;
    Ok(context_file)
}

/// `aps start <ID>` — claim a Ready work item (`cmd_start`).
pub fn cmd_start(plan_root: &str, id: &str) -> i32 {
    let root = Path::new(plan_root);
    if !root.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let graph = match PlanGraph::load(root) {
        Ok(graph) => graph,
        Err(message) => {
            eprintln!("error: {message}");
            return 1;
        }
    };

    let Some(item) = graph.find(id) else {
        eprintln!("error: Work item not found: {id}");
        return 1;
    };

    let module_status = graph.module_status(&item.module);
    if !matches!(module_status, "Ready" | "In Progress") {
        eprintln!(
            "error: {id} belongs to module {} (status: {module_status}) - module must be Ready or In Progress to start work items",
            item.module
        );
        return 1;
    }

    let already_started = match item.status.as_str() {
        "Ready" => false,
        "In Progress" => true,
        "Complete" => {
            eprintln!("error: {id} is already Complete - cannot restart");
            return 1;
        }
        other => {
            eprintln!("error: {id} has status '{other}' - cannot start (must be Ready)");
            return 1;
        }
    };

    if !graph.deps_complete(&item.deps) {
        eprintln!(
            "error: {id} has unmet dependencies: {}",
            deps_display(&item.deps)
        );
        return 1;
    }

    if !already_started
        && let Err(err) = rewrite_work_item(&item.file, id, RewriteMode::Status, "In Progress")
    {
        eprintln!("error: {err}");
        return 1;
    }

    let context_file = match write_context_package(&graph, item, root) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    if already_started {
        eprintln!("warning: {id} is already In Progress (no status change)");
    } else {
        println!("Marked {id} as In Progress");
    }
    println!("Suggested branch: work/{}", id.to_lowercase());
    println!("File: {}", item.file);
    println!("Context package: {}", context_file.display());
    0
}

/// `aps complete <ID>` — close out an In Progress work item (`cmd_complete`).
pub fn cmd_complete(plan_root: &str, id: &str, learning: Option<&str>) -> i32 {
    let root = Path::new(plan_root);
    if !root.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let graph = match PlanGraph::load(root) {
        Ok(graph) => graph,
        Err(message) => {
            eprintln!("error: {message}");
            return 1;
        }
    };

    let Some(item) = graph.find(id) else {
        eprintln!("error: Work item not found: {id}");
        return 1;
    };
    let file = item.file.clone();

    match item.status.as_str() {
        "In Progress" => {}
        "Complete" => {
            eprintln!("warning: {id} is already Complete (no change)");
            return 0;
        }
        other => {
            eprintln!("error: {id} has status '{other}' - cannot complete (must be In Progress)");
            return 1;
        }
    }

    // Prompt interactively when no learning was supplied and stdin is a TTY.
    let mut learning = learning.map(str::to_string).unwrap_or_default();
    if learning.is_empty() && io::stdin().is_terminal() {
        print!("Learning (optional): ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            learning = input.trim_end_matches(['\n', '\r']).to_string();
        }
    }

    let today = date::today_utc_ymd();
    if let Err(err) = rewrite_work_item(
        &file,
        id,
        RewriteMode::Status,
        &format!("Complete: {today}"),
    ) {
        eprintln!("error: {err}");
        return 1;
    }
    if !learning.is_empty()
        && let Err(err) = rewrite_work_item(&file, id, RewriteMode::Learning, &learning)
    {
        eprintln!("error: {err}");
        return 1;
    }

    println!("Marked {id} as Complete: {today}");
    if !learning.is_empty() {
        println!("Learning recorded for {id}");
    }
    println!("File: {file}");
    0
}

/// `aps graph [module]` — render items and dependency arrows (`cmd_graph`).
pub fn cmd_graph(plan_root: &str, module_filter: &str) -> i32 {
    let root = Path::new(plan_root);
    if !root.is_dir() {
        eprintln!("error: Path not found: {plan_root}");
        return 1;
    }
    let graph = match PlanGraph::load(root) {
        Ok(graph) => graph,
        Err(message) => {
            eprintln!("error: {message}");
            return 1;
        }
    };

    let mut shown = false;
    for item in &graph.items {
        if !graph.matches_module(item, module_filter) {
            continue;
        }
        shown = true;
        println!("{} [{}] {}", item.id, item.status, item.title);

        let mut deps = String::new();
        for dep in parser::dep_tokens(&item.deps) {
            if let Some(dep_item) = graph.find(&dep) {
                deps.push_str(&format!(" {}[{}]", dep_item.id, dep_item.status));
            } else {
                deps.push_str(&format!(" {dep}[{}]", graph.module_status(&dep)));
            }
        }
        if deps.is_empty() {
            println!("  <- none");
        } else {
            println!("  <-{deps}");
        }
    }

    if !shown {
        if module_filter.is_empty() {
            eprintln!("warning: No work items found");
        } else {
            eprintln!("warning: No work items found for module: {module_filter}");
        }
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn work_copy(tag: &str) -> PathBuf {
        let src = PathBuf::from("../test/fixtures/orchestrate");
        let dst = std::env::temp_dir().join(format!("aps-orch-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dst);
        copy_dir(&src, &dst);
        dst
    }

    fn copy_dir(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap().flatten() {
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if from.is_dir() {
                copy_dir(&from, &to);
            } else {
                fs::copy(&from, &to).unwrap();
            }
        }
    }

    #[test]
    fn meta_and_continuation_detection() {
        assert!(is_meta_line("- **Status:** Ready"));
        assert!(is_meta_line("- **Validation:** `true`"));
        assert!(!is_meta_line("- plain bullet"));
        assert!(!is_meta_line("  indented"));
        assert!(is_continuation("  more text"));
        assert!(!is_continuation("flush left"));
        assert!(!is_continuation("   "));
    }

    #[test]
    fn start_marks_in_progress_and_writes_context() {
        let root = work_copy("start");
        let plans = root.join("plans");
        let code = cmd_start(plans.to_str().unwrap(), "AUTH-003");
        assert_eq!(code, 0);

        let auth = fs::read_to_string(plans.join("modules/auth.aps.md")).unwrap();
        assert!(auth.contains("- **Status:** In Progress"));

        let ctx = fs::read_to_string(root.join(".aps/context/AUTH-003.md")).unwrap();
        assert!(ctx.contains("# Context: AUTH-003 - Add token refresh"));
        assert!(ctx.contains("## Work Item"));
        assert!(ctx.contains("## Module Scope"));
        assert!(ctx.contains("## Dependency Learnings"));
        assert!(ctx.contains("CORE-001: \"Parser output is stable across modules\""));
        assert!(ctx.contains("- src/auth/refresh.sh"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn start_rejects_unmet_deps_and_missing_items() {
        let root = work_copy("start-bad");
        let plans = root.join("plans");
        let plans = plans.to_str().unwrap();
        assert_eq!(cmd_start(plans, "AUTH-004"), 1); // AUTH-003 not complete
        assert_eq!(cmd_start(plans, "NOPE-999"), 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn final_item_context_stops_at_next_section() {
        let root = work_copy("start-final");
        let plans = root.join("plans");
        assert_eq!(cmd_start(plans.to_str().unwrap(), "AUTH-006"), 0);
        let ctx = fs::read_to_string(root.join(".aps/context/AUTH-006.md")).unwrap();
        // The Work Item section must not bleed into the module's ## Decisions.
        let work_item = ctx.split("## Module Scope").next().unwrap();
        assert!(!work_item.contains("D-001"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn complete_stamps_status_and_learning() {
        let root = work_copy("complete");
        let plans = root.join("plans");
        let plans_s = plans.to_str().unwrap();
        assert_eq!(cmd_start(plans_s, "AUTH-003"), 0);
        let code = cmd_complete(plans_s, "AUTH-003", Some("Rotate refresh tokens"));
        assert_eq!(code, 0);

        let auth = fs::read_to_string(plans.join("modules/auth.aps.md")).unwrap();
        assert!(auth.contains("- **Status:** Complete:"));
        assert!(auth.contains("- **Learning:** \"Rotate refresh tokens\""));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn complete_rejects_non_in_progress() {
        let root = work_copy("complete-bad");
        let plans = root.join("plans");
        // AUTH-004 is Ready, not In Progress.
        assert_eq!(cmd_complete(plans.to_str().unwrap(), "AUTH-004", None), 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn graph_renders_items_and_arrows() {
        let root = work_copy("graph");
        let plans = root.join("plans");
        // Capture is awkward; assert exit code and rely on integration tests
        // for text. Unknown module → exit 1.
        assert_eq!(cmd_graph(plans.to_str().unwrap(), "auth"), 0);
        assert_eq!(cmd_graph(plans.to_str().unwrap(), "nope"), 1);
        fs::remove_dir_all(&root).ok();
    }
}

//! Native `aps rollup` (MONO-004).
//!
//! Prints a Markdown roll-up table for a federated (nested-plans) parent: one
//! row per child plan with modules complete/total, the next ready item, and an
//! overall status. Mirrors `cmd_rollup` in lib/orchestrate.sh — same rows, same
//! order, same output. The root index stays hand-authored; this is the data
//! source you copy into the parent's `## Roll-up` section at session end.

use std::path::Path;

use crate::next::{self, PlanGraph};
use crate::parser::{self, PlanFile};

/// Path-derived child name for a plan root (the directory above `plans/`),
/// matching `next::child_name` / bash `orch_child_name`.
fn child_name(root: &Path) -> String {
    let norm = parser::normalize_path(&root.to_string_lossy());
    Path::new(&norm)
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// First actionable item's ID within a child scope, or `—` when none — the same
/// selection `next_ready` makes, reused for the "Next ready item" column.
fn child_next_ready(graph: &PlanGraph, child: &str) -> String {
    graph
        .next_ready("", child)
        .map(|item| item.id.clone())
        .unwrap_or_else(|| "—".to_string())
}

/// CLI entry. Returns the process exit code.
pub fn cmd_rollup(plan_root: &str) -> i32 {
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

    println!("| Child | Modules (complete/total) | Next ready item | Status |");
    println!("| ----- | ------------------------ | --------------- | ------ |");

    let mut shown = false;
    for (idx, fed_root) in next::plan_roots(root).into_iter().enumerate() {
        // The first root is the federation parent itself; roll-up covers children.
        if idx == 0 {
            continue;
        }
        let module_dir = fed_root.join("modules");
        if !module_dir.is_dir() {
            continue;
        }
        shown = true;
        let child = child_name(&fed_root);

        let (mut total, mut complete, mut inprogress) = (0u32, 0u32, 0u32);
        for file in parser::find_aps_files(&module_dir) {
            if !file.ends_with(".aps.md") {
                continue;
            }
            let Ok(plan) = PlanFile::load(&file) else {
                continue;
            };
            let status = parser::normalize_status(plan.status().as_deref().unwrap_or(""), "Draft");
            total += 1;
            match status.as_str() {
                "Complete" => complete += 1,
                "In Progress" => inprogress += 1,
                _ => {}
            }
        }

        let next_ready = child_next_ready(&graph, &child);
        let overall = if total > 0 && complete == total {
            "Complete"
        } else if inprogress > 0 {
            "In Progress"
        } else {
            "Ready"
        };

        println!("| {child} | {complete}/{total} | {next_ready} | {overall} |");
    }

    if !shown {
        eprintln!(
            "warning: No child plans found under {plan_root} (rollup is for federated parents)"
        );
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn monorepo_root() -> PathBuf {
        PathBuf::from("../test/fixtures/monorepo/plans")
    }

    #[test]
    fn rollup_rows_match_child_state() {
        let graph = PlanGraph::load(&monorepo_root()).unwrap();
        // core has one Ready module with an unblocked item.
        assert_eq!(child_next_ready(&graph, "core"), "AUTH-001");
        // api's only item is blocked by its cross-tree dep, so no next item.
        assert_eq!(child_next_ready(&graph, "api"), "—");
    }

    #[test]
    fn rollup_exits_ok_on_federation() {
        assert_eq!(cmd_rollup(monorepo_root().to_str().unwrap()), 0);
    }

    #[test]
    fn rollup_warns_without_children() {
        // A single child plan (no ## Child Plans) has nothing to roll up.
        let child = monorepo_root().join("../packages/core/plans");
        assert_eq!(cmd_rollup(child.to_str().unwrap()), 1);
    }
}

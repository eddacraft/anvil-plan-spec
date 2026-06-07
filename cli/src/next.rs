//! Native `aps next` (TUI-009).
//!
//! Resolves the next Ready work item whose dependencies are Complete,
//! mirroring `cmd_next` in lib/orchestrate.sh: same module gating, same
//! dependency semantics (D-* decisions auto-complete, bare uppercase
//! tokens reference modules), same output text and exit codes.

use std::collections::HashMap;
use std::path::Path;

use crate::parser::{self, PlanFile};

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub deps: String,
    pub module: String,
    pub file: String,
}

#[derive(Debug, Default)]
pub struct PlanGraph {
    pub items: Vec<WorkItem>,
    pub module_statuses: HashMap<String, String>,
}

impl PlanGraph {
    /// Load index module table + all module work items
    /// (`orch_load_index_modules` + `orch_load_work_items` with load_all).
    pub fn load(plan_root: &Path) -> Result<Self, String> {
        let mut graph = Self::default();

        let index_path = plan_root.join("index.aps.md");
        if index_path.is_file()
            && let Ok(index) = PlanFile::load(&index_path.to_string_lossy())
        {
            for (module, status) in parser::index_modules(&index) {
                graph.module_statuses.insert(module, status);
            }
        }

        let module_dir = plan_root.join("modules");
        if !module_dir.is_dir() {
            return Err(format!(
                "No modules directory found: {}/modules",
                plan_root.display()
            ));
        }

        for file in parser::find_aps_files(&module_dir) {
            if !file.ends_with(".aps.md") {
                continue;
            }
            let Ok(plan) = PlanFile::load(&file) else {
                continue;
            };

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
            let module_status =
                parser::normalize_status(plan.status().as_deref().unwrap_or(""), "Draft");
            graph
                .module_statuses
                .insert(module_id.clone(), module_status);

            for item in plan.work_items() {
                let Some(id) = parser::parse_work_item_id(&item.header) else {
                    continue;
                };
                let content = plan.item_content(item.line);
                let mut status = parser::field_value(&content, "Status");
                if status.is_empty() && item.header.contains("Complete") {
                    status = "Complete".to_string();
                }
                let status = parser::normalize_status(&status, "Ready");
                let deps = parser::field_value(&content, "Dependencies");

                graph.items.push(WorkItem {
                    id: id.to_string(),
                    title: parser::work_item_title(&item.header),
                    status,
                    deps,
                    module: module_id.clone(),
                    file: file.clone(),
                });
            }
        }

        Ok(graph)
    }

    fn item_status(&self, id: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.status.as_str())
    }

    /// `orch_deps_complete`.
    pub fn deps_complete(&self, deps: &str) -> bool {
        let trimmed = deps.trim();
        if trimmed.is_empty() || trimmed == "None" || trimmed == "-" {
            return true;
        }
        if !trimmed.chars().any(|c| c.is_alphanumeric()) {
            return true;
        }

        let tokens = parser::dep_tokens(deps);
        if tokens.is_empty() {
            return false;
        }

        tokens.iter().all(|token| {
            if parser::is_item_token(token) {
                // Decision dependencies are resolved in plan text.
                if token.starts_with("D-") {
                    return true;
                }
                self.item_status(token) == Some("Complete")
            } else {
                self.module_statuses.get(token).map(String::as_str) == Some("Complete")
            }
        })
    }

    fn matches_module(&self, item: &WorkItem, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let base = Path::new(&item.file)
            .file_name()
            .map(|name| {
                name.to_string_lossy()
                    .trim_end_matches(".aps.md")
                    .to_string()
            })
            .unwrap_or_default();
        item.module.eq_ignore_ascii_case(filter) || base.eq_ignore_ascii_case(filter)
    }

    /// First Ready item (in file order) whose module is Ready/In Progress
    /// and whose dependencies are Complete.
    pub fn next_ready(&self, module_filter: &str) -> Option<&WorkItem> {
        self.items.iter().find(|item| {
            if !self.matches_module(item, module_filter) {
                return false;
            }
            let module_status = self
                .module_statuses
                .get(&item.module)
                .map(String::as_str)
                .unwrap_or("Unknown");
            if !matches!(module_status, "Ready" | "In Progress") {
                return false;
            }
            if item.status != "Ready" {
                return false;
            }
            self.deps_complete(&item.deps)
        })
    }
}

fn deps_display(deps: &str) -> String {
    let display = deps.replace('\n', ", ");
    if display.is_empty() {
        "None".to_string()
    } else {
        display
    }
}

/// CLI entry. Returns the process exit code.
pub fn cmd_next(plan_root: &str, module_filter: &str) -> i32 {
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

    match graph.next_ready(module_filter) {
        Some(item) => {
            println!("{}: {}", item.id, item.title);
            println!(
                "Module: {} | Dependencies: {} | Status: {}",
                item.module,
                deps_display(&item.deps),
                item.status
            );
            println!("File: {}", item.file);
            0
        }
        None => {
            if module_filter.is_empty() {
                eprintln!("warning: No ready work item found");
            } else {
                eprintln!("warning: No ready work item found for module: {module_filter}");
            }
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        // cli/ is the cargo working directory in tests.
        PathBuf::from("../test/fixtures/orchestrate/plans")
    }

    #[test]
    fn resolves_next_ready_item_from_fixtures() {
        let graph = PlanGraph::load(&fixture_root()).unwrap();
        let item = graph.next_ready("").expect("a ready item exists");

        assert_eq!(item.id, "AUTH-003");
        assert_eq!(item.title, "Add token refresh");
        assert_eq!(item.module, "AUTH");
    }

    #[test]
    fn module_filter_matches_id_and_filename() {
        let graph = PlanGraph::load(&fixture_root()).unwrap();

        assert!(graph.next_ready("auth").is_some());
        assert!(graph.next_ready("AUTH").is_some());
    }

    #[test]
    fn decision_deps_are_auto_complete() {
        let graph = PlanGraph::default();
        assert!(graph.deps_complete("D-026, D-027"));
        assert!(graph.deps_complete("None"));
        assert!(graph.deps_complete(""));
        assert!(graph.deps_complete("-"));
    }

    #[test]
    fn module_deps_require_complete_status() {
        let mut graph = PlanGraph::default();
        graph
            .module_statuses
            .insert("INSTALL".to_string(), "In Progress".to_string());
        assert!(!graph.deps_complete("INSTALL"));

        graph
            .module_statuses
            .insert("INSTALL".to_string(), "Complete".to_string());
        assert!(graph.deps_complete("INSTALL"));
    }

    #[test]
    fn blocked_modules_hide_their_items() {
        let root = std::env::temp_dir().join(format!("aps-next-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("modules")).unwrap();
        fs::write(
            root.join("modules/x.aps.md"),
            "| ID | Status |\n| --- | --- |\n| X | Draft |\n\n## Work Items\n\n### X-001: Thing\n\n- **Status:** Ready\n",
        )
        .unwrap();

        let graph = PlanGraph::load(&root).unwrap();
        assert!(graph.next_ready("").is_none());

        fs::remove_dir_all(&root).unwrap();
    }
}

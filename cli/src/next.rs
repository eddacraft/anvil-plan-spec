//! Native `aps next` (TUI-009).
//!
//! Resolves the next Ready work item whose dependencies are Complete,
//! mirroring `cmd_next` in lib/orchestrate.sh: same module gating, same
//! dependency semantics (D-* decisions auto-complete, bare uppercase
//! tokens reference modules), same output text and exit codes.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::parser::{self, PlanFile};

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub deps: String,
    pub module: String,
    pub file: String,
    /// 1-based line of the `### ID:` header, for re-reading item content.
    pub line: usize,
    /// Path-derived child-plan name this item belongs to (MONO-003). Empty for
    /// a plain single-root plan; used to scope and disambiguate federated trees.
    pub child: String,
    /// Item-level `Packages:` scope tags (PKG-001); empty = inherit from the
    /// module's metadata column.
    pub packages: String,
}

/// Outcome of resolving a work-item reference across a federation (MONO-003).
pub enum RefResolution<'a> {
    Found(&'a WorkItem),
    NotFound,
    /// A bare ID defined in more than one child tree.
    Ambiguous,
}

/// Every plan root in a federation: the given root plus each child root
/// reachable transitively via `## Child Plans`. Deduped on normalised paths.
/// A plain single-root plan yields just itself. (`orch_plan_roots`)
pub fn plan_roots(start: &Path) -> Vec<PathBuf> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    queue.push_back(start.to_path_buf());
    let mut roots = Vec::new();

    while let Some(root) = queue.pop_front() {
        let key = parser::normalize_path(&root.to_string_lossy());
        if !seen.insert(key) {
            continue;
        }
        roots.push(root.clone());

        let index = root.join("index.aps.md");
        if index.is_file() {
            for child_index in parser::resolve_child_plan_links(&index) {
                if let Some(dir) = Path::new(&child_index).parent() {
                    queue.push_back(dir.to_path_buf());
                }
            }
        }
    }

    roots
}

/// Path-derived child name for a plan root: the directory segment above
/// `plans/`. (`orch_child_name`)
fn child_name(root: &Path) -> String {
    let norm = parser::normalize_path(&root.to_string_lossy());
    Path::new(&norm)
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[derive(Debug, Default)]
pub struct PlanGraph {
    pub items: Vec<WorkItem>,
    pub module_statuses: HashMap<String, String>,
    /// Module-level `Packages:` metadata column, keyed like `module_statuses`.
    pub module_packages: HashMap<String, String>,
}

impl PlanGraph {
    /// Load index module table + all module work items
    /// (`orch_load_index_modules` + `orch_load_work_items` with load_all).
    pub fn load(plan_root: &Path) -> Result<Self, String> {
        Self::load_inner(plan_root, true)
    }

    /// Load module work items without the index module table — mirrors
    /// `cmd_audit`, which calls `orch_load_work_items` alone so module
    /// statuses come purely from each module file.
    pub fn load_items_only(plan_root: &Path) -> Result<Self, String> {
        Self::load_inner(plan_root, false)
    }

    fn load_inner(plan_root: &Path, with_index: bool) -> Result<Self, String> {
        let mut graph = Self::default();
        let mut loaded_any = false;

        // Traverse the whole federation: the given root plus every child plan
        // reachable via `## Child Plans` (MONO-003). A single-root plan yields
        // just itself, preserving the original behaviour.
        for root in plan_roots(plan_root) {
            let child = child_name(&root);

            let index_path = root.join("index.aps.md");
            if with_index
                && index_path.is_file()
                && let Ok(index) = PlanFile::load(&index_path.to_string_lossy())
            {
                for (module, status) in parser::index_modules(&index) {
                    graph
                        .module_statuses
                        .insert(module_status_key(&module, &child), status);
                }
            }

            let module_dir = root.join("modules");
            if !module_dir.is_dir() {
                // A federation parent owns no modules of its own — its children
                // supply the work. Skip roots without a modules/ dir.
                continue;
            }
            loaded_any = true;

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
                    .insert(module_status_key(&module_id, &child), module_status);
                graph.module_packages.insert(
                    module_status_key(&module_id, &child),
                    plan.module_packages().unwrap_or_default(),
                );

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
                        line: item.line,
                        child: child.clone(),
                        packages: parser::field_value(&content, "Packages"),
                    });
                }
            }
        }

        if !loaded_any {
            return Err(format!(
                "No modules directory found: {}/modules",
                plan_root.display()
            ));
        }

        Ok(graph)
    }

    fn item_status(&self, id: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.status.as_str())
    }

    /// Locate a work item by bare ID, first match (`orch_item_index`).
    pub fn find(&self, id: &str) -> Option<&WorkItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Resolve a work-item reference across the federation (`orch_resolve_ref`).
    /// Accepts a bare ID (`AUTH-001`) or a cross-tree ref (`core:AUTH-001`); a
    /// non-empty `scope` constrains a bare ID to one child tree. A bare ID that
    /// matches items in more than one tree is [`RefResolution::Ambiguous`].
    pub fn resolve_ref(&self, reference: &str, scope: &str) -> RefResolution<'_> {
        let (rchild, rid) = match reference.split_once(':') {
            Some((c, i)) => (c.to_string(), i.to_string()),
            None => (String::new(), reference.to_string()),
        };
        let rchild = if rchild.is_empty() && !scope.is_empty() {
            scope.to_string()
        } else {
            rchild
        };

        let matches: Vec<&WorkItem> = self
            .items
            .iter()
            .filter(|item| {
                item.id == rid && (rchild.is_empty() || item.child.eq_ignore_ascii_case(&rchild))
            })
            .collect();

        match matches.len() {
            0 => RefResolution::NotFound,
            1 => RefResolution::Found(matches[0]),
            _ => RefResolution::Ambiguous,
        }
    }

    /// Whether an item belongs to the given child scope (`orch_item_matches_child`).
    pub fn matches_child(&self, item: &WorkItem, child: &str) -> bool {
        child.is_empty() || item.child.eq_ignore_ascii_case(child)
    }

    /// Module status within its owning child tree (`ORCH_MODULE_STATUSES`).
    /// A bare-key fallback preserves single-root plans and direct unit fixtures.
    pub fn module_status(&self, module: &str, self_child: &str) -> &str {
        let (child, module) = module.split_once(':').unwrap_or((self_child, module));
        self.module_statuses
            .get(&module_status_key(module, child))
            .or_else(|| self.module_statuses.get(module))
            .map(String::as_str)
            .unwrap_or("Unknown")
    }

    /// `orch_deps_complete`. `self_child` is the depending item's child tree,
    /// so a bare ID resolves in-tree first (D-002 allows the same bare ID in
    /// sibling trees); empty for single-root plans.
    pub fn deps_complete(&self, deps: &str, self_child: &str) -> bool {
        let trimmed = deps.trim();
        if trimmed.is_empty() || trimmed == "None" || trimmed == "-" {
            return true;
        }
        if !trimmed.chars().any(|c| c.is_alphanumeric()) {
            return true;
        }

        let tokens = parser::dep_refs(deps);
        if tokens.is_empty() {
            return false;
        }

        tokens.iter().all(|token| {
            if token.contains(':') {
                // Cross-tree ref (child:ID) — resolve within the named tree.
                matches!(
                    self.resolve_ref(token, ""),
                    RefResolution::Found(item) if item.status == "Complete"
                )
            } else if parser::is_item_token(token) {
                // Decision dependencies are resolved in plan text.
                if token.starts_with("D-") {
                    return true;
                }
                // Bare ID means "an item in my own tree" — resolve within the
                // depending item's child first so declaration order can't
                // misattribute it, falling back to a federation-wide first
                // match only when the ID isn't defined in-tree.
                let status = match self.resolve_ref(token, self_child) {
                    RefResolution::Found(item) => Some(item.status.as_str()),
                    _ => self.item_status(token),
                };
                status == Some("Complete")
            } else {
                self.module_status(token, self_child) == "Complete"
            }
        })
    }

    pub fn matches_module(&self, item: &WorkItem, filter: &str) -> bool {
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
    /// and whose dependencies are Complete. Optionally scoped to one child plan.
    pub fn next_ready(&self, module_filter: &str, child_scope: &str) -> Option<&WorkItem> {
        self.candidates(module_filter, child_scope, "")
            .into_iter()
            .next()
    }

    /// Every ready-with-deps-met item passing the module/child/package
    /// filters, in load order (PKG-001; the shared gate behind `next` and
    /// `next --by-package`).
    pub fn candidates(
        &self,
        module_filter: &str,
        child_scope: &str,
        package_filter: &str,
    ) -> Vec<&WorkItem> {
        self.items
            .iter()
            .filter(|item| {
                if !self.matches_child(item, child_scope) {
                    return false;
                }
                if !self.matches_module(item, module_filter) {
                    return false;
                }
                if !self.matches_package(item, package_filter) {
                    return false;
                }
                let module_status = self.module_status(&item.module, &item.child);
                if !matches!(module_status, "Ready" | "In Progress") {
                    return false;
                }
                if item.status != "Ready" {
                    return false;
                }
                self.deps_complete(&item.deps, &item.child)
            })
            .collect()
    }

    /// Effective `Packages:` value for an item — its own field, else its
    /// module's metadata column (docs/monorepo.md: items inherit from the
    /// module when the field is omitted).
    pub fn item_packages(&self, item: &WorkItem) -> String {
        if !item.packages.is_empty() {
            return item.packages.clone();
        }
        self.module_packages
            .get(&module_status_key(&item.module, &item.child))
            .cloned()
            .unwrap_or_default()
    }

    /// True when the item's effective `Packages:` include `filter`
    /// (normalised comparison). An untagged item matches no package filter.
    fn matches_package(&self, item: &WorkItem, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let want = pkg_normalize(filter);
        let pkgs = self.item_packages(item);
        if pkgs.is_empty() {
            return false;
        }
        pkgs.split(',').any(|entry| pkg_normalize(entry) == want)
    }
}

/// Normalise one `Packages:` entry for matching: trim whitespace/backticks,
/// strip a leading `packages/` or `apps/` root, lowercase. `packages/Core`
/// matches `core`.
pub fn pkg_normalize(entry: &str) -> String {
    let entry = entry.trim().trim_matches('`').trim();
    let entry = entry.strip_prefix("packages/").unwrap_or(entry);
    let entry = entry.strip_prefix("apps/").unwrap_or(entry);
    entry.to_lowercase()
}

fn module_status_key(module: &str, child: &str) -> String {
    if child.is_empty() {
        module.to_uppercase()
    } else {
        format!("{}:{}", child.to_ascii_lowercase(), module.to_uppercase())
    }
}

/// Render a Dependencies field for display (`orch_deps_display`): newlines
/// become `, ` and an empty value reads `None`.
pub fn deps_display(deps: &str) -> String {
    let display = deps.replace('\n', ", ");
    if display.is_empty() {
        "None".to_string()
    } else {
        display
    }
}

/// CLI entry. Returns the process exit code.
pub fn cmd_next(
    plan_root: &str,
    module_filter: &str,
    child_scope: &str,
    package_filter: &str,
    by_package: bool,
) -> i32 {
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

    let candidates = graph.candidates(module_filter, child_scope, package_filter);
    if !candidates.is_empty() {
        if by_package {
            // Group by normalised package name; a multi-tagged item appears
            // under each of its packages. Headings sort lexically; (untagged)
            // comes last. Byte-identical to the bash cmd_next --by-package.
            let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut untagged: Vec<String> = Vec::new();
            for item in &candidates {
                let line = format!("  {}: {} ({})", item.id, item.title, item.module);
                let pkgs = graph.item_packages(item);
                if pkgs.is_empty() {
                    untagged.push(line);
                    continue;
                }
                for entry in pkgs.split(',') {
                    let name = pkg_normalize(entry);
                    if name.is_empty() {
                        continue;
                    }
                    groups.entry(name).or_default().push(line.clone());
                }
            }
            let mut first = true;
            for (name, lines) in &groups {
                if !first {
                    println!();
                }
                first = false;
                println!("{name}:");
                for line in lines {
                    println!("{line}");
                }
            }
            if !untagged.is_empty() {
                if !first {
                    println!();
                }
                println!("(untagged):");
                for line in &untagged {
                    println!("{line}");
                }
            }
        } else {
            let item = candidates[0];
            println!("{}: {}", item.id, item.title);
            println!(
                "Module: {} | Dependencies: {} | Status: {}",
                item.module,
                deps_display(&item.deps),
                item.status
            );
            println!("File: {}", item.file);
        }
        return 0;
    }

    let mut note = String::new();
    if !module_filter.is_empty() {
        note.push_str(&format!(" for module: {module_filter}"));
    }
    if !package_filter.is_empty() {
        note.push_str(&format!(" for package: {package_filter}"));
    }
    if !child_scope.is_empty() {
        note.push_str(&format!(" in child: {child_scope}"));
    }
    eprintln!("warning: No ready work item found{note}");
    1
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
    fn package_filter_and_inheritance_resolve_from_fixtures() {
        let graph = PlanGraph::load(&PathBuf::from("../test/fixtures/pkgnext/plans")).unwrap();
        // Inherited module tag: AUTH-001 has no item field but the AUTH
        // module column says core.
        let core = graph.candidates("", "", "core");
        assert_eq!(core[0].id, "AUTH-001");
        assert!(core.iter().any(|i| i.id == "HND-001"), "item-level tag");
        // Path-qualified + case-insensitive filter.
        assert_eq!(graph.candidates("", "", "packages/CORE").len(), core.len());
        // Item-level tag apps/api normalises to api.
        let api = graph.candidates("", "", "api");
        assert_eq!(api.len(), 1);
        assert_eq!(api[0].id, "HND-001");
        // Untagged items match no filter but stay in the unfiltered queue.
        assert!(graph.candidates("", "", "ghost").is_empty());
        let all = graph.candidates("", "", "");
        assert!(all.iter().any(|i| i.id == "MISC-001"));
        let misc = all.iter().find(|i| i.id == "MISC-001").unwrap();
        assert!(graph.item_packages(misc).is_empty());
    }

    #[test]
    fn resolves_next_ready_item_from_fixtures() {
        let graph = PlanGraph::load(&fixture_root()).unwrap();
        let item = graph.next_ready("", "").expect("a ready item exists");

        assert_eq!(item.id, "AUTH-003");
        assert_eq!(item.title, "Add token refresh");
        assert_eq!(item.module, "AUTH");
    }

    #[test]
    fn module_filter_matches_id_and_filename() {
        let graph = PlanGraph::load(&fixture_root()).unwrap();

        assert!(graph.next_ready("auth", "").is_some());
        assert!(graph.next_ready("AUTH", "").is_some());
    }

    #[test]
    fn decision_deps_are_auto_complete() {
        let graph = PlanGraph::default();
        assert!(graph.deps_complete("D-026, D-027", ""));
        assert!(graph.deps_complete("None", ""));
        assert!(graph.deps_complete("", ""));
        assert!(graph.deps_complete("-", ""));
    }

    #[test]
    fn module_deps_require_complete_status() {
        let mut graph = PlanGraph::default();
        graph
            .module_statuses
            .insert("INSTALL".to_string(), "In Progress".to_string());
        assert!(!graph.deps_complete("INSTALL", ""));

        graph
            .module_statuses
            .insert("INSTALL".to_string(), "Complete".to_string());
        assert!(graph.deps_complete("INSTALL", ""));
    }

    fn monorepo_root() -> PathBuf {
        PathBuf::from("../test/fixtures/monorepo/plans")
    }

    #[test]
    fn federation_spans_child_plans() {
        // The parent root owns no modules; work is pulled from both children.
        let graph = PlanGraph::load(&monorepo_root()).unwrap();
        let ids: Vec<&str> = graph.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"AUTH-001"), "core child item loaded");
        assert!(ids.contains(&"HND-001"), "api child item loaded");

        // Only core:AUTH-001 is unblocked (HND-001 waits on the cross-tree dep).
        let item = graph.next_ready("", "").expect("a ready item exists");
        assert_eq!(item.id, "AUTH-001");
        assert_eq!(item.child, "core");
    }

    #[test]
    fn child_scope_filters_the_queue() {
        let graph = PlanGraph::load(&monorepo_root()).unwrap();
        assert_eq!(
            graph.next_ready("", "core").map(|i| i.id.as_str()),
            Some("AUTH-001")
        );
        // api's only item is blocked by its cross-tree dependency.
        assert!(graph.next_ready("", "api").is_none());
    }

    #[test]
    fn resolve_ref_handles_bare_and_cross_tree() {
        let graph = PlanGraph::load(&monorepo_root()).unwrap();
        assert!(matches!(
            graph.resolve_ref("AUTH-001", ""),
            RefResolution::Found(item) if item.child == "core"
        ));
        assert!(matches!(
            graph.resolve_ref("core:AUTH-001", ""),
            RefResolution::Found(item) if item.child == "core"
        ));
        assert!(matches!(
            graph.resolve_ref("NOPE-999", ""),
            RefResolution::NotFound
        ));
    }

    #[test]
    fn cross_tree_dependency_gates_readiness() {
        // HND-001 depends on core:AUTH-001, which is Ready (not Complete).
        let graph = PlanGraph::load(&monorepo_root()).unwrap();
        assert!(!graph.deps_complete("core:AUTH-001", ""));
    }

    #[test]
    fn bare_dependency_resolves_in_the_depending_items_own_tree() {
        // D-002 allows the same bare ID in sibling trees. A bare dep must
        // resolve to the depending item's own tree, not a declaration-order
        // first match elsewhere. Build a two-tree graph where `core` and `api`
        // both define AUTH-001 with different statuses.
        let mut graph = PlanGraph::default();
        let mk = |id: &str, status: &str, deps: &str, child: &str| WorkItem {
            id: id.to_string(),
            title: String::new(),
            status: status.to_string(),
            deps: deps.to_string(),
            module: "M".to_string(),
            file: format!("{child}/plans/modules/m.aps.md"),
            line: 1,
            child: child.to_string(),
            packages: String::new(),
        };
        // Declaration order puts core (Ready) before api (Complete).
        graph.items.push(mk("AUTH-001", "Ready", "", "core"));
        graph.items.push(mk("AUTH-001", "Complete", "", "api"));

        // A bare dep from the api tree sees api's Complete AUTH-001.
        assert!(graph.deps_complete("AUTH-001", "api"));
        // The same bare dep from the core tree sees core's Ready AUTH-001.
        assert!(!graph.deps_complete("AUTH-001", "core"));
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
        assert!(graph.next_ready("", "").is_none());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn colliding_module_ids_are_scoped_to_their_child_tree() {
        let mut graph = PlanGraph::default();
        graph
            .module_statuses
            .insert("core:AUTH".to_string(), "Ready".to_string());
        graph
            .module_statuses
            .insert("api:AUTH".to_string(), "Draft".to_string());
        graph.items.push(WorkItem {
            id: "AUTH-001".to_string(),
            title: "Core work".to_string(),
            status: "Ready".to_string(),
            deps: String::new(),
            module: "AUTH".to_string(),
            file: "core/plans/modules/auth.aps.md".to_string(),
            line: 1,
            child: "core".to_string(),
            packages: String::new(),
        });
        graph.items.push(WorkItem {
            id: "AUTH-002".to_string(),
            title: "Api work".to_string(),
            status: "Ready".to_string(),
            deps: String::new(),
            module: "AUTH".to_string(),
            file: "api/plans/modules/auth.aps.md".to_string(),
            line: 1,
            child: "api".to_string(),
            packages: String::new(),
        });

        assert_eq!(
            graph.next_ready("", "core").map(|item| item.id.as_str()),
            Some("AUTH-001")
        );
        assert!(graph.next_ready("", "api").is_none());
    }
}

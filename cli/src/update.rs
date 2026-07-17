//! `aps update` — reconcile a project's generated APS footprint.
//!
//! Unlike `setup upgrade` (which silently refreshed only files that already
//! existed), `update` reconciles the *full* expected footprint: it adds
//! missing core files, refreshes existing ones, and — crucially — reports
//! every file as added / updated / unchanged / skipped so the result is never
//! a mystery count. Feature-gated members (designs templates, the skill) are
//! reconciled only when their feature is installed, and reported as skipped
//! with a reason otherwise. User-authored planning content is never touched.

use std::fs;
use std::path::Path;

use crate::config::{self, parse_config};
use crate::scaffold::{agent_files, agent_paths};
use crate::wizard::{AiTool, ModelPreference};

// Core generated files every APS project carries (relative to plans/). These
// are added when missing and refreshed when present.
const CORE: &[(&str, &str)] = &[
    (
        "aps-rules.md",
        include_str!("../scaffold/plans/aps-rules.md"),
    ),
    (
        "modules/.module.template.md",
        include_str!("../scaffold/plans/modules/.module.template.md"),
    ),
    (
        "modules/.simple.template.md",
        include_str!("../scaffold/plans/modules/.simple.template.md"),
    ),
    (
        "modules/.index-monorepo.template.md",
        include_str!("../scaffold/plans/modules/.index-monorepo.template.md"),
    ),
    (
        "execution/.actions.template.md",
        include_str!("../scaffold/plans/execution/.actions.template.md"),
    ),
];

// Designs templates (relative to plans/), reconciled only when designs/ is
// installed — never resurrected in a project that opted out.
const DESIGNS: &[(&str, &str)] = &[(
    "designs/.design.template.md",
    include_str!("../scaffold/designs/.design.template.md"),
)];

// Skill files (relative to the project root), reconciled only when the
// aps-planning/ skill is installed.
const SKILL: &[(&str, &str)] = &[
    (
        "SKILL.md",
        include_str!("../scaffold/aps-planning/SKILL.md"),
    ),
    (
        "reference.md",
        include_str!("../scaffold/aps-planning/reference.md"),
    ),
    (
        "examples.md",
        include_str!("../scaffold/aps-planning/examples.md"),
    ),
    (
        "hooks.md",
        include_str!("../scaffold/aps-planning/hooks.md"),
    ),
];

#[derive(Default)]
struct Tally {
    added: u32,
    updated: u32,
    unchanged: u32,
    removed: u32,
    skipped: u32,
    failed: u32,
}

/// Remove an obsolete generated file without touching neighbouring content.
fn remove_obsolete(path: &Path, display: &str, tally: &mut Tally) {
    if !path.exists() {
        return;
    }
    match fs::remove_file(path) {
        Ok(()) => {
            println!("  - {display} (removed obsolete generated file)");
            tally.removed += 1;
        }
        Err(err) => {
            println!("  ! {display} (failed: {err})");
            tally.failed += 1;
        }
    }
}

/// Write `content` to `path`, reporting the outcome relative to `display`.
fn reconcile(path: &Path, display: &str, content: &str, tally: &mut Tally) {
    let existed = path.exists();
    if existed && fs::read_to_string(path).is_ok_and(|current| current == content) {
        println!("  = {display} (unchanged)");
        tally.unchanged += 1;
        return;
    }
    if let Some(parent) = path.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        println!("  ! {display} (failed: {err})");
        tally.failed += 1;
        return;
    }
    match fs::write(path, content) {
        Ok(()) if existed => {
            println!("  ~ {display} (updated)");
            tally.updated += 1;
        }
        Ok(()) => {
            println!("  + {display} (added)");
            tally.added += 1;
        }
        Err(err) => {
            println!("  ! {display} (failed: {err})");
            tally.failed += 1;
        }
    }
}

/// `aps update [dir]` entry. Returns the process exit code.
pub fn cmd_update(start: &Path) -> i32 {
    let project = config::discover_project(start);
    let root = project
        .as_ref()
        .map(|p| p.root.clone())
        .unwrap_or_else(|| start.to_path_buf());
    let plans_rel = project
        .as_ref()
        .and_then(|p| p.plans_dir.clone())
        .unwrap_or_else(|| "plans".to_string());
    let plans = root.join(plans_rel.trim_end_matches('/'));

    if !plans.is_dir() {
        eprintln!(
            "error: no plans directory at {} — nothing to update",
            plans.display()
        );
        eprintln!("  Run `aps init` to scaffold a new project.");
        return 1;
    }

    let mut tally = Tally::default();

    println!("Core templates ({}):", plans.display());
    for (rel, content) in CORE {
        reconcile(&plans.join(rel), rel, content, &mut tally);
    }

    // Designs templates: reconcile when designs/ exists, else report skipped.
    println!("\nDesigns templates:");
    if plans.join("designs").is_dir() {
        for (rel, content) in DESIGNS {
            reconcile(&plans.join(rel), rel, content, &mut tally);
        }
    } else {
        for (rel, _) in DESIGNS {
            println!("  - {rel} (skipped: designs not installed)");
            tally.skipped += 1;
        }
    }

    // Skill: reconcile when aps-planning/ exists, else report skipped.
    let skill_dir = root.join("aps-planning");
    println!("\nPlanning skill (aps-planning/):");
    if skill_dir.is_dir() {
        for (rel, content) in SKILL {
            reconcile(&skill_dir.join(rel), rel, content, &mut tally);
        }
    } else {
        for (rel, _) in SKILL {
            println!("  - {rel} (skipped: skill not installed)");
            tally.skipped += 1;
        }
    }

    // Agents: regenerate from cores + model preference when already installed.
    // Prefer the project's config.yml model for each tool; fall back to Default
    // (role-weighted map). Codex also drops obsolete registration snippets.
    let legacy_codex_snippets = [
        ".codex/agents/codex-config-snippet.toml",
        ".codex/codex-config-snippet.toml",
    ];
    // Prefer each tool's model preference from `.aps/config.yml` when present.
    let config_models: Vec<(AiTool, ModelPreference)> = fs::read_to_string(root.join(".aps/config.yml"))
        .ok()
        .and_then(|text| parse_config(&text).ok())
        .map(|s| s.tools.into_iter().map(|c| (c.tool, c.model)).collect())
        .unwrap_or_default();
    let model_for = |tool: AiTool| -> ModelPreference {
        config_models
            .iter()
            .find(|(t, _)| *t == tool)
            .map(|(_, m)| *m)
            .unwrap_or(ModelPreference::Default)
    };

    for tool in [AiTool::ClaudeCode, AiTool::Copilot, AiTool::Codex, AiTool::OpenCode] {
        let label = match tool {
            AiTool::ClaudeCode => "Claude Code agents (.claude/agents/)",
            AiTool::Copilot => "Copilot agents (.github/agents/)",
            AiTool::Codex => "Codex agents (.codex/agents/)",
            AiTool::OpenCode => "OpenCode agents (.opencode/agent/)",
            _ => "agents",
        };
        println!("\n{label}:");
        let installed = agent_paths(tool)
            .iter()
            .any(|rel| root.join(rel).is_file())
            || (tool == AiTool::Codex
                && legacy_codex_snippets
                    .iter()
                    .any(|rel| root.join(rel).is_file()));
        if installed {
            for (rel, content) in agent_files(tool, model_for(tool)) {
                reconcile(&root.join(&rel), &rel, &content, &mut tally);
            }
            if tool == AiTool::Codex {
                for rel in legacy_codex_snippets {
                    remove_obsolete(&root.join(rel), rel, &mut tally);
                }
            }
        } else {
            for rel in agent_paths(tool) {
                println!("  - {rel} (skipped: not installed)");
                tally.skipped += 1;
            }
        }
    }

    println!(
        "\nUpdated: {} added, {} updated, {} unchanged, {} removed, {} skipped{}",
        tally.added,
        tally.updated,
        tally.unchanged,
        tally.removed,
        tally.skipped,
        if tally.failed > 0 {
            format!(", {} failed", tally.failed)
        } else {
            String::new()
        }
    );
    if tally.failed > 0 {
        return 1;
    }
    println!("Your plan content (index, modules, decisions, designs) was preserved.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aps-update-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn errors_without_a_plans_dir() {
        let root = scratch("noplans");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(cmd_update(&root), 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn adds_missing_core_and_refreshes_existing() {
        let root = scratch("reconcile");
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("execution")).unwrap();
        // A stale aps-rules that must be refreshed; templates are missing and
        // must be added — the bug `setup upgrade` had was silently skipping
        // these missing ones.
        fs::write(plans.join("aps-rules.md"), "stale\n").unwrap();

        assert_eq!(cmd_update(&root), 0);

        // Missing core templates were added (not silently skipped).
        assert!(plans.join("modules/.module.template.md").is_file());
        assert!(plans.join("execution/.actions.template.md").is_file());
        // The stale file was refreshed to the shipped content.
        let rules = fs::read_to_string(plans.join("aps-rules.md")).unwrap();
        assert_ne!(rules, "stale\n");
        // Designs/skill are gated off when not installed.
        assert!(!plans.join("designs/.design.template.md").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn reconciles_designs_and_skill_when_present() {
        let root = scratch("gated");
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("designs")).unwrap();
        fs::create_dir_all(root.join("aps-planning")).unwrap();
        assert_eq!(cmd_update(&root), 0);
        assert!(plans.join("designs/.design.template.md").is_file());
        assert!(root.join("aps-planning/SKILL.md").is_file());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn reconciles_codex_roles_and_removes_legacy_snippets_when_present() {
        let root = scratch("codex");
        let plans = root.join("plans");
        let agents = root.join(".codex/agents");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("execution")).unwrap();
        fs::create_dir_all(&agents).unwrap();
        fs::write(
            agents.join("aps-planner.toml"),
            "developer_instructions = \"\"\"stale\"\"\"\n",
        )
        .unwrap();
        fs::write(agents.join("codex-config-snippet.toml"), "legacy\n").unwrap();
        fs::write(root.join(".codex/codex-config-snippet.toml"), "legacy\n").unwrap();

        assert_eq!(cmd_update(&root), 0);

        let planner = fs::read_to_string(agents.join("aps-planner.toml")).unwrap();
        assert!(planner.contains("name = \"aps-planner\""));
        assert!(!planner.contains("stale"));
        assert!(agents.join("aps-librarian.toml").is_file());
        assert!(agents.join("aps-conductor.toml").is_file());
        assert!(!agents.join("codex-config-snippet.toml").exists());
        assert!(!root.join(".codex/codex-config-snippet.toml").exists());

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn leaves_unrelated_codex_agents_untouched() {
        let root = scratch("unrelated-codex");
        let plans = root.join("plans");
        let agents = root.join(".codex/agents");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("execution")).unwrap();
        fs::create_dir_all(&agents).unwrap();
        fs::write(
            agents.join("reviewer.toml"),
            "name = \"reviewer\"\ndeveloper_instructions = \"review\"\n",
        )
        .unwrap();

        assert_eq!(cmd_update(&root), 0);

        assert!(agents.join("reviewer.toml").is_file());
        assert!(!agents.join("aps-planner.toml").exists());
        assert!(!agents.join("aps-librarian.toml").exists());
        assert!(!agents.join("aps-conductor.toml").exists());

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn second_run_reports_unchanged() {
        let root = scratch("idempotent");
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(plans.join("execution")).unwrap();
        assert_eq!(cmd_update(&root), 0); // first run adds
        assert_eq!(cmd_update(&root), 0); // second run is a no-op write-wise
        fs::remove_dir_all(&root).ok();
    }
}

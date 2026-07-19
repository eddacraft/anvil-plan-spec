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
use crate::managed::{self, ReconcileResult};
use crate::scaffold::{
    AGENTS_SKILL_DIR, CLAUDE_SKILL_DIR, HOOK_SCRIPTS, SKILL_FILES, agent_files, agent_paths,
    mark_executable,
};
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

// Legacy v1 skill files (relative to a root aps-planning/ tree). Projects
// that still carry the v1 layout get refreshed in place; `aps migrate`
// is what moves them to the v2 roots.
const LEGACY_SKILL: &[(&str, &str)] = &[
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

    // Skill: managed reconcile for each v2 skill root that is installed
    // (.claude/skills/ for Claude Code / Copilot / OpenCode, .agents/skills/
    // for Codex / Grok). Sidecar `.aps-managed.json` distinguishes APS-owned
    // content from user edits — dirty trees are refused, not overwritten.
    // Legacy root aps-planning/ stays on string reconcile until migrate.
    // Agents remain on string reconcile (managed inventory is Phase 3).
    println!("\nPlanning skill:");
    let mut skill_roots = 0;
    let expected_skill = managed::expected_skill_manifest();
    for dir in [CLAUDE_SKILL_DIR, AGENTS_SKILL_DIR] {
        let skill_dir = root.join(dir);
        if !skill_dir.is_dir() {
            continue;
        }
        skill_roots += 1;
        match managed::reconcile_managed_skill(&skill_dir, &SKILL_FILES, &expected_skill) {
            Ok(ReconcileResult::Unchanged) => {
                println!("  = {dir} (unchanged)");
                tally.unchanged += 1;
            }
            Ok(ReconcileResult::Updated) => {
                println!("  ~ {dir} (updated)");
                tally.updated += 1;
            }
            Ok(ReconcileResult::Added) => {
                println!("  + {dir} (added)");
                tally.added += 1;
            }
            Ok(ReconcileResult::DirtySkipped) => {
                println!("  ! {dir} (dirty: user modified; not updated)");
                tally.skipped += 1;
            }
            Ok(ReconcileResult::UnmanagedSkipped) => {
                println!("  ! {dir} (unmanaged: differs from embeds; not updated)");
                tally.skipped += 1;
            }
            Ok(ReconcileResult::Adopted) => {
                println!("  ~ {dir} (adopted managed marker)");
                tally.updated += 1;
            }
            Ok(ReconcileResult::BrokenSkipped) => {
                println!("  ! {dir} (broken managed marker; not updated)");
                tally.skipped += 1;
            }
            Err(err) => {
                println!("  ! {dir} (failed: {err})");
                tally.failed += 1;
            }
        }
    }
    let legacy_skill = root.join("aps-planning");
    if legacy_skill.is_dir() {
        skill_roots += 1;
        for (name, content) in LEGACY_SKILL {
            let rel = format!("aps-planning/{name}");
            reconcile(&root.join(&rel), &rel, content, &mut tally);
        }
        println!("  ! aps-planning/ is the v1 location — run `aps migrate` to move it");
    }
    if skill_roots == 0 {
        for (name, _) in SKILL_FILES {
            println!("  - {name} (skipped: skill not installed)");
            tally.skipped += 1;
        }
    }

    // Hook scripts: reconcile .aps/scripts/ when hooks are installed.
    println!("\nHook scripts (.aps/scripts/):");
    if root.join(".aps/scripts").is_dir() {
        for (rel, content) in HOOK_SCRIPTS {
            let path = root.join(rel);
            reconcile(&path, rel, content, &mut tally);
            let _ = mark_executable(&path);
        }
    } else {
        for (rel, _) in HOOK_SCRIPTS {
            println!("  - {rel} (skipped: hooks not installed)");
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
    let config_models: Vec<(AiTool, ModelPreference)> =
        fs::read_to_string(root.join(".aps/config.yml"))
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

    for tool in [
        AiTool::ClaudeCode,
        AiTool::Copilot,
        AiTool::Codex,
        AiTool::OpenCode,
    ] {
        let label = match tool {
            AiTool::ClaudeCode => "Claude Code agents (.claude/agents/)",
            AiTool::Copilot => "Copilot agents (.github/agents/)",
            AiTool::Codex => "Codex agents (.codex/agents/)",
            AiTool::OpenCode => "OpenCode agents (.opencode/agent/)",
            _ => "agents",
        };
        println!("\n{label}:");
        let installed = agent_paths(tool).iter().any(|rel| root.join(rel).is_file())
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
        fs::create_dir_all(root.join(".claude/skills/aps-planning")).unwrap();
        assert_eq!(cmd_update(&root), 0);
        assert!(plans.join("designs/.design.template.md").is_file());
        assert!(root.join(".claude/skills/aps-planning/SKILL.md").is_file());
        // Empty skill dir is installed with content + managed marker.
        assert!(
            root.join(".claude/skills/aps-planning")
                .join(crate::managed::MANIFEST_NAME)
                .is_file()
        );
        // The v2 skill is three files — hooks.md is v1-only.
        assert!(!root.join(".claude/skills/aps-planning/hooks.md").exists());
        // No other skill root is invented.
        assert!(!root.join(".agents").exists());
        assert!(!root.join("aps-planning").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn refreshes_legacy_root_skill_in_place() {
        let root = scratch("legacy-skill");
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(root.join("aps-planning")).unwrap();
        fs::write(root.join("aps-planning/SKILL.md"), "stale\n").unwrap();

        assert_eq!(cmd_update(&root), 0);

        // Refreshed in place (v1 set, including hooks.md); migrate moves it.
        let skill = fs::read_to_string(root.join("aps-planning/SKILL.md")).unwrap();
        assert_ne!(skill, "stale\n");
        assert!(root.join("aps-planning/hooks.md").is_file());
        assert!(!root.join(".claude").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn reconciles_hook_scripts_when_installed() {
        let root = scratch("hooks");
        let plans = root.join("plans");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(root.join(".aps/scripts")).unwrap();
        fs::write(root.join(".aps/scripts/install-hooks.sh"), "stale\n").unwrap();

        assert_eq!(cmd_update(&root), 0);

        let script = fs::read_to_string(root.join(".aps/scripts/install-hooks.sh")).unwrap();
        assert_ne!(script, "stale\n");
        assert!(root.join(".aps/scripts/init-session.sh").is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(root.join(".aps/scripts/install-hooks.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_ne!(mode & 0o111, 0, "hook script should be executable");
        }
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

    #[test]
    fn managed_skill_refuses_dirty_user_edits() {
        let root = scratch("skill-dirty");
        let plans = root.join("plans");
        let skill = root.join(".claude/skills/aps-planning");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(&skill).unwrap();
        // First update installs embeds + marker into the empty skill dir.
        assert_eq!(cmd_update(&root), 0);
        assert!(skill.join(crate::managed::MANIFEST_NAME).is_file());
        fs::write(skill.join("SKILL.md"), "# local fork\n").unwrap();

        assert_eq!(cmd_update(&root), 0);
        assert_eq!(
            fs::read_to_string(skill.join("SKILL.md")).unwrap(),
            "# local fork\n"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn managed_skill_adopts_matching_unmanaged_tree() {
        let root = scratch("skill-adopt");
        let plans = root.join("plans");
        let skill = root.join(".claude/skills/aps-planning");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(&skill).unwrap();
        for (name, content) in SKILL_FILES {
            fs::write(skill.join(name), content).unwrap();
        }

        assert_eq!(cmd_update(&root), 0);
        assert!(skill.join(crate::managed::MANIFEST_NAME).is_file());
        // Content unchanged; only the marker was added.
        assert_eq!(
            fs::read_to_string(skill.join("SKILL.md")).unwrap(),
            SKILL_FILES[0].1
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn managed_skill_skips_unmanaged_differing_tree() {
        let root = scratch("skill-unmanaged");
        let plans = root.join("plans");
        let skill = root.join(".claude/skills/aps-planning");
        fs::create_dir_all(plans.join("modules")).unwrap();
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "custom\n").unwrap();
        fs::write(skill.join("reference.md"), "custom\n").unwrap();
        fs::write(skill.join("examples.md"), "custom\n").unwrap();

        assert_eq!(cmd_update(&root), 0);
        assert!(!skill.join(crate::managed::MANIFEST_NAME).exists());
        assert_eq!(
            fs::read_to_string(skill.join("SKILL.md")).unwrap(),
            "custom\n"
        );
        fs::remove_dir_all(&root).ok();
    }
}

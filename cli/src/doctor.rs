//! `aps doctor` — diagnose a project's migration state from a vendored bash
//! CLI to the global release binary (INSTALL-017).
//!
//! It reports four things: whether the global binary's version matches the
//! project's `cli_version` pin, leftover vendored CLI trees that `aps migrate`
//! can clean, the completeness of a global `~/.aps/lib/` bash runtime, and
//! stale direnv `PATH_add bin` entries left over from the vendored layout.

use std::path::{Path, PathBuf};

use crate::config::{self, parse_config};
use crate::managed::{self, SkillState};
use crate::scaffold::{AGENTS_SKILL_DIR, CLAUDE_SKILL_DIR, CLI_VERSION};
use crate::wizard::AiTool;

/// Files a complete vendored/global bash `lib/` runtime must contain. Used to
/// flag a half-installed `~/.aps/lib/` (e.g. one missing `audit.sh`).
pub const REQUIRED_LIB_FILES: &[&str] = &[
    "output.sh",
    "lint.sh",
    "orchestrate.sh",
    "audit.sh",
    "scaffold.sh",
    "rules/common.sh",
    "rules/module.sh",
    "rules/index.sh",
    "rules/workitem.sh",
    "rules/issues.sh",
    "rules/design.sh",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Healthy — nothing to do.
    Ok,
    /// Worth attention but not broken (e.g. version drift, leftover bloat).
    Warn,
    /// Broken state that needs fixing (e.g. an incomplete runtime).
    Problem,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub level: Level,
    pub label: String,
    pub detail: String,
}

impl Finding {
    fn new(level: Level, label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            level,
            label: label.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    /// True when any finding is a `Problem` (used to drive a non-zero exit).
    pub fn has_problems(&self) -> bool {
        self.findings.iter().any(|f| f.level == Level::Problem)
    }
}

/// Inspect a project and machine state, returning a structured report.
///
/// `start` is the directory to begin the project search from; `home` is the
/// home directory holding a global `~/.aps/`; `exe` is the running binary path
/// (for the global-binary line). All filesystem state is read, never written —
/// `doctor` is safe to run anywhere.
pub fn diagnose(start: &Path, home: &Path, exe: Option<&Path>) -> Report {
    let mut findings = Vec::new();

    // 1. Global binary presence + version.
    match exe {
        Some(path) => findings.push(Finding::new(
            Level::Ok,
            "global binary",
            format!("aps {CLI_VERSION} at {}", path.display()),
        )),
        None => findings.push(Finding::new(
            Level::Warn,
            "global binary",
            format!("running aps {CLI_VERSION} (path undetermined)"),
        )),
    }

    // 2. cli_version pin vs the running binary.
    let project = config::discover_project(start);
    let root = project
        .as_ref()
        .map(|p| p.root.clone())
        .unwrap_or_else(|| start.to_path_buf());
    match project.as_ref().and_then(|p| p.cli_version.as_deref()) {
        Some(pin) if pin == CLI_VERSION => findings.push(Finding::new(
            Level::Ok,
            "cli_version",
            format!("project pins {pin}, matches this binary"),
        )),
        Some(pin) => findings.push(Finding::new(
            Level::Warn,
            "cli_version",
            format!("project pins {pin} but this binary is {CLI_VERSION} — install the pinned release or update the pin"),
        )),
        None if project.is_some() => findings.push(Finding::new(
            Level::Warn,
            "cli_version",
            "no cli_version in .aps/config.yml — add one to pin the toolchain".to_string(),
        )),
        None => findings.push(Finding::new(
            Level::Warn,
            "cli_version",
            "no .aps/config.yml found — run `aps init` to write the project contract".to_string(),
        )),
    }

    // 3. Leftover vendored CLI trees that `aps migrate` can back up and remove.
    // Gate on an APS-specific marker so an unrelated project `bin/` or `lib/`
    // (common in real repos) is never mistaken for a vendored APS CLI.
    let leftovers: Vec<&str> = [
        ("bin/aps", "bin/aps"),
        ("lib/lint.sh", "lib"),
        (".aps/bin/aps", ".aps/bin"),
        (".aps/lib/lint.sh", ".aps/lib"),
    ]
    .into_iter()
    .filter(|(marker, _)| root.join(marker).exists())
    .map(|(_, label)| label)
    .collect();
    if leftovers.is_empty() {
        findings.push(Finding::new(
            Level::Ok,
            "vendored CLI",
            "no vendored bin/lib trees — running on the global binary".to_string(),
        ));
    } else {
        findings.push(Finding::new(
            Level::Warn,
            "vendored CLI",
            format!(
                "leftover vendored CLI under {}: {} — run `aps migrate` to back up and remove",
                root.display(),
                leftovers.join(", ")
            ),
        ));
    }

    // 4. Completeness of a global ~/.aps/lib/ bash runtime. Absence is healthy
    // in the binary-first world (the native binary needs no bash lib); we only
    // flag a *partial* runtime, which is a genuinely broken bash fallback.
    let global_lib = home.join(".aps/lib");
    if global_lib.is_dir() {
        let missing: Vec<&str> = REQUIRED_LIB_FILES
            .iter()
            .copied()
            .filter(|rel| !global_lib.join(rel).is_file())
            .collect();
        if missing.is_empty() {
            findings.push(Finding::new(
                Level::Ok,
                "global runtime",
                format!("{} is complete", global_lib.display()),
            ));
        } else {
            findings.push(Finding::new(
                Level::Problem,
                "global runtime",
                format!(
                    "{} is incomplete — missing: {}. Reinstall the CLI (curl .../install | bash -s -- --cli --bash)",
                    global_lib.display(),
                    missing.join(", ")
                ),
            ));
        }
    } else {
        findings.push(Finding::new(
            Level::Ok,
            "global runtime",
            "no vendored bash runtime at ~/.aps/lib — running on the native binary".to_string(),
        ));
    }

    // 5. Stale direnv PATH_add bin entry from the vendored layout.
    let envrc = root.join(".envrc");
    if let Ok(text) = std::fs::read_to_string(&envrc)
        && text
            .lines()
            .any(|l| l.split('#').next().unwrap_or("").contains("PATH_add bin"))
    {
        findings.push(Finding::new(
            Level::Warn,
            "direnv",
            format!(
                "{} still adds ./bin to PATH — drop it once you run on the global binary",
                envrc.display()
            ),
        ));
    }

    // 6. Planning skill freshness (Phase 1 managed inventory — skills only).
    findings.extend(skill_freshness_findings(&root));

    Report { findings }
}

/// Tools that install the planning skill (generic-only projects do not).
fn config_expects_skill(root: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join(".aps/config.yml")) else {
        return false;
    };
    let Ok(selections) = parse_config(&text) else {
        return false;
    };
    selections.tools.iter().any(|c| {
        matches!(
            c.tool,
            AiTool::ClaudeCode
                | AiTool::Copilot
                | AiTool::Codex
                | AiTool::OpenCode
                | AiTool::Grok
        )
    })
}

fn skill_freshness_findings(root: &Path) -> Vec<Finding> {
    let expected = managed::expected_skill_manifest();
    let roots = [
        (CLAUDE_SKILL_DIR, root.join(CLAUDE_SKILL_DIR)),
        (AGENTS_SKILL_DIR, root.join(AGENTS_SKILL_DIR)),
    ];
    let present: Vec<_> = roots
        .iter()
        .filter(|(_, path)| path.is_dir())
        .collect();

    if present.is_empty() {
        return if config_expects_skill(root) {
            vec![Finding::new(
                Level::Warn,
                "planning skill",
                "not installed — run `aps setup claude-code` (or codex/grok)".to_string(),
            )]
        } else {
            vec![Finding::new(
                Level::Ok,
                "planning skill",
                "not installed (optional)".to_string(),
            )]
        };
    }

    present
        .into_iter()
        .map(|(label, path)| {
            let state = managed::evaluate_skill_dir(path, &expected);
            match state {
                SkillState::Fresh => Finding::new(
                    Level::Ok,
                    "planning skill",
                    format!("{label}: fresh"),
                ),
                SkillState::Stale => Finding::new(
                    Level::Warn,
                    "planning skill",
                    format!("{label}: stale — run `aps update`"),
                ),
                SkillState::Dirty => Finding::new(
                    Level::Warn,
                    "planning skill",
                    format!("{label}: dirty (user modified) — not overwritten by `aps update`"),
                ),
                SkillState::Unmanaged => Finding::new(
                    Level::Warn,
                    "planning skill",
                    format!(
                        "{label}: unmanaged (no {}) — run `aps update` to adopt if content matches",
                        managed::MANIFEST_NAME
                    ),
                ),
                SkillState::Broken => Finding::new(
                    Level::Problem,
                    "planning skill",
                    format!("{label}: broken managed marker — fix or remove {}", managed::MANIFEST_NAME),
                ),
                SkillState::Absent => Finding::new(
                    Level::Warn,
                    "planning skill",
                    format!("{label}: empty — run `aps update` to install"),
                ),
            }
        })
        .collect()
}

/// Render a report to stdout and return the process exit code (1 if any
/// `Problem`, else 0).
pub fn run(start: &Path) -> i32 {
    // Canonicalize so leftover/direnv findings print an absolute path, not ".".
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let start = start.as_path();
    let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let exe = std::env::current_exe().ok();
    let report = diagnose(start, &home, exe.as_deref());

    println!("aps doctor — migration diagnostics\n");
    for finding in &report.findings {
        let tag = match finding.level {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Problem => "FAIL",
        };
        println!("  [{tag}] {}: {}", finding.label, finding.detail);
    }

    if report.has_problems() {
        println!(
            "\nSome checks failed. See docs/installation.md → \"Migrating to the global binary\"."
        );
        1
    } else {
        println!("\nNo blocking problems. See docs/installation.md for the migration steps.");
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aps-doctor-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn level_of(report: &Report, label: &str) -> Level {
        report
            .findings
            .iter()
            .find(|f| f.label == label)
            .unwrap_or_else(|| panic!("no finding labelled {label}"))
            .level
    }

    #[test]
    fn clean_global_project_is_healthy() {
        let root = scratch("clean");
        let home = scratch("clean-home");
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {CLI_VERSION}\nplans_dir: plans/\n"),
        )
        .unwrap();
        // An unrelated project lib/ (no APS marker) must not be flagged.
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/widget.js"), "// app code\n").unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));

        assert!(!report.has_problems());
        assert_eq!(level_of(&report, "cli_version"), Level::Ok);
        assert_eq!(level_of(&report, "vendored CLI"), Level::Ok);

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn bloated_v1_project_flags_leftovers_and_drift() {
        let root = scratch("bloated");
        let home = scratch("bloated-home");
        // v1 scatter: root bin/lib (with the bash CLI marker), a stale direnv
        // entry, and a drifted cli_version pin.
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/aps"), "#!/usr/bin/env bash\n").unwrap();
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/lint.sh"), "# bash lint\n").unwrap();
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(root.join(".aps/config.yml"), "cli_version: 0.0.1\n").unwrap();
        fs::write(root.join(".envrc"), "PATH_add bin\n").unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));

        assert_eq!(level_of(&report, "cli_version"), Level::Warn);
        assert_eq!(level_of(&report, "vendored CLI"), Level::Warn);
        assert_eq!(level_of(&report, "direnv"), Level::Warn);

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn incomplete_global_lib_is_a_problem() {
        let root = scratch("incomplete");
        let home = scratch("incomplete-home");
        let lib = home.join(".aps/lib/rules");
        fs::create_dir_all(&lib).unwrap();
        // Install every required file EXCEPT audit.sh.
        for rel in REQUIRED_LIB_FILES.iter().filter(|r| **r != "audit.sh") {
            let path = home.join(".aps/lib").join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "# stub\n").unwrap();
        }

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));

        assert!(report.has_problems());
        let runtime = report
            .findings
            .iter()
            .find(|f| f.label == "global runtime")
            .unwrap();
        assert_eq!(runtime.level, Level::Problem);
        assert!(runtime.detail.contains("audit.sh"));

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn missing_skill_is_ok_when_optional() {
        let root = scratch("skill-optional");
        let home = scratch("skill-optional-home");
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {CLI_VERSION}\nplans_dir: plans/\n"),
        )
        .unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));
        assert_eq!(level_of(&report, "planning skill"), Level::Ok);
        assert!(
            report
                .findings
                .iter()
                .find(|f| f.label == "planning skill")
                .unwrap()
                .detail
                .contains("optional")
        );

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn missing_skill_warns_when_tools_configured() {
        let root = scratch("skill-expected");
        let home = scratch("skill-expected-home");
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!(
                "cli_version: {CLI_VERSION}\nplans_dir: plans/\ntools:\n  - name: claude-code\n    agents: true\n    hooks: none\n    model: default\n"
            ),
        )
        .unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));
        assert_eq!(level_of(&report, "planning skill"), Level::Warn);

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn fresh_skill_is_ok() {
        let root = scratch("skill-fresh");
        let home = scratch("skill-fresh-home");
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {CLI_VERSION}\nplans_dir: plans/\n"),
        )
        .unwrap();
        let skill = root.join(CLAUDE_SKILL_DIR);
        let expected = managed::expected_skill_manifest();
        for (name, content) in crate::scaffold::SKILL_FILES {
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join(name), content).unwrap();
        }
        managed::write_skill_marker(&skill, &expected).unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));
        assert_eq!(level_of(&report, "planning skill"), Level::Ok);
        assert!(
            report
                .findings
                .iter()
                .find(|f| f.label == "planning skill")
                .unwrap()
                .detail
                .contains("fresh")
        );

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn dirty_skill_warns() {
        let root = scratch("skill-dirty");
        let home = scratch("skill-dirty-home");
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {CLI_VERSION}\nplans_dir: plans/\n"),
        )
        .unwrap();
        let skill = root.join(CLAUDE_SKILL_DIR);
        let expected = managed::expected_skill_manifest();
        for (name, content) in crate::scaffold::SKILL_FILES {
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join(name), content).unwrap();
        }
        managed::write_skill_marker(&skill, &expected).unwrap();
        fs::write(skill.join("SKILL.md"), "edited\n").unwrap();

        let report = diagnose(&root, &home, Some(Path::new("/usr/bin/aps")));
        assert_eq!(level_of(&report, "planning skill"), Level::Warn);
        assert!(
            report
                .findings
                .iter()
                .find(|f| f.label == "planning skill")
                .unwrap()
                .detail
                .contains("dirty")
        );

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&home).ok();
    }
}

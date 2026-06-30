//! `aps migrate` — move a project onto the global `aps` binary.
//!
//! The binary-first migration documented in installation.md §"Migrating to the
//! Global Binary": diagnose toolchain drift, back up and remove the vendored
//! bash CLI bloat (the core of scaffold/upgrade — root `bin/` + `lib/`,
//! `.aps/bin` + `.aps/lib`, the v1 `aps-planning/` skill, `.claude/commands/`),
//! rewrite stale hook paths, drop a stale direnv `PATH_add bin`, and pin
//! `cli_version`. Dry-run by default; `--apply` performs the changes after
//! backing every removed path up to `.aps/backup/<timestamp>/`. User content —
//! `plans/**`, AGENTS.md, CLAUDE.md, GEMINI.md, settings — is never removed.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::config;
use crate::date;
use crate::doctor::{self, Level};
use crate::scaffold::CLI_VERSION;

/// Known APS bash-lib filenames relative to a `lib/` dir (scaffold/upgrade).
const APS_LIB_FILES: &[&str] = &[
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

/// True when every file under `dir` is a recognised APS bash-lib file — so the
/// whole `lib/` is APS-generated and safe to remove (`dir_only_aps_lib`).
fn dir_only_aps_lib(dir: &Path) -> bool {
    fn walk(dir: &Path, base: &Path, ok: &mut bool) {
        let Ok(entries) = fs::read_dir(dir) else {
            *ok = false;
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, ok);
            } else {
                let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy();
                if !APS_LIB_FILES.contains(&rel.as_ref()) {
                    *ok = false;
                }
            }
            if !*ok {
                return;
            }
        }
    }
    let mut ok = true;
    walk(dir, dir, &mut ok);
    ok
}

/// Recursively copy a file or directory (`cp -R`).
fn copy_path(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)?.flatten() {
            copy_path(&entry.path(), &dst.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst).map(|_| ())
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// Vendored-CLI cleanup plan for `target` (the REMOVE/AMBIGUOUS split from
/// scaffold/upgrade).
struct CleanupPlan {
    remove: Vec<String>,
    ambiguous: Vec<String>,
}

fn build_cleanup_plan(target: &Path) -> CleanupPlan {
    let mut remove = Vec::new();
    let mut ambiguous = Vec::new();

    for cmd in [
        ".claude/commands/plan.md",
        ".claude/commands/plan-status.md",
    ] {
        if target.join(cmd).exists() {
            remove.push(cmd.to_string());
        }
    }
    if target.join("bin/aps").is_file() {
        remove.push("bin/aps".to_string());
    }
    let lib = target.join("lib");
    if lib.is_dir() && lib.join("lint.sh").is_file() {
        if dir_only_aps_lib(&lib) {
            remove.push("lib".to_string());
        } else {
            ambiguous.push("lib/ (mixed APS + non-APS files)".to_string());
        }
    }
    let skill = target.join("aps-planning");
    if skill.is_dir() {
        if skill.join("SKILL.md").is_file() || skill.join("hooks.md").is_file() {
            remove.push("aps-planning".to_string());
        } else {
            ambiguous.push("aps-planning/ (unrecognised contents)".to_string());
        }
    }
    if target.join(".aps/lib").is_dir() {
        remove.push(".aps/lib".to_string());
    }
    if target.join(".aps/bin").is_dir() {
        remove.push(".aps/bin".to_string());
    }

    CleanupPlan { remove, ambiguous }
}

/// The uncommented `.envrc` line index carrying `PATH_add bin`, if any.
fn stale_direnv_line(target: &Path) -> Option<(PathBuf, usize)> {
    let envrc = target.join(".envrc");
    let text = fs::read_to_string(&envrc).ok()?;
    let idx = text.lines().position(|l| {
        l.split('#')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<_>>()
            == ["PATH_add", "bin"]
    })?;
    Some((envrc, idx))
}

/// True when the project has a config without a `cli_version` pin.
fn needs_pin(target: &Path) -> bool {
    config::discover_project(target).is_some_and(|p| p.cli_version.is_none())
}

fn print_diagnosis(start: &Path) {
    let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let exe = std::env::current_exe().ok();
    let report = doctor::diagnose(start, &home, exe.as_deref());
    println!("Diagnosis:");
    for finding in &report.findings {
        let tag = match finding.level {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Problem => "FAIL",
        };
        println!("  [{tag}] {}: {}", finding.label, finding.detail);
    }
    println!();
}

fn confirm(prompt: &str) -> bool {
    if !io::stdin().is_terminal() {
        return false;
    }
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).is_ok()
        && answer.trim_start().to_lowercase().starts_with('y')
}

/// `aps migrate [dir]` entry. Returns the process exit code.
pub fn cmd_migrate(start: &Path, apply: bool, assume_yes: bool) -> i32 {
    // Operate at the project root when discoverable, else the given dir.
    let target = config::discover_project(start)
        .map(|p| p.root)
        .unwrap_or_else(|| start.to_path_buf());

    if !target.join("plans").is_dir() {
        eprintln!(
            "error: no plans/ directory at {} — nothing to migrate",
            target.display()
        );
        eprintln!("  Run `aps init` to scaffold a new project.");
        return 1;
    }

    print_diagnosis(start);

    let plan = build_cleanup_plan(&target);
    let direnv = stale_direnv_line(&target);
    let pin = needs_pin(&target);

    if plan.remove.is_empty() && plan.ambiguous.is_empty() && direnv.is_none() && !pin {
        println!("Already on the global binary — nothing to migrate.");
        return 0;
    }

    println!("Migration plan for {}:", target.display());
    if !plan.remove.is_empty() {
        println!("\n  Back up and remove (vendored CLI bloat):");
        for p in &plan.remove {
            println!("    - {p}");
        }
    }
    if !plan.ambiguous.is_empty() {
        println!("\n  Ambiguous — left untouched, review manually:");
        for p in &plan.ambiguous {
            println!("    - {p}");
        }
    }
    if pin {
        println!("\n  Pin toolchain: add `cli_version: {CLI_VERSION}` to .aps/config.yml");
    }
    if direnv.is_some() {
        println!("\n  Drop stale direnv `PATH_add bin` from .envrc");
    }
    println!("\n  Protected and never removed: plans/, AGENTS.md, CLAUDE.md, GEMINI.md, settings");

    if !apply {
        println!("\nDry run — no files changed. Re-run with --apply to migrate.");
        return 0;
    }

    let has_destructive = !plan.remove.is_empty() || direnv.is_some() || pin;
    if !has_destructive {
        println!("\nNothing to apply (only ambiguous items). No changes made.");
        return 0;
    }

    if !assume_yes && !confirm("\nApply the migration above? [y/N] ") {
        if !io::stdin().is_terminal() {
            eprintln!("error: refusing to modify files non-interactively without --yes");
            return 1;
        }
        println!("Aborted — nothing was changed.");
        return 0;
    }

    let stamp = date::now_stamp();
    let backup = target.join(".aps/backup").join(&stamp);

    // 1. Back up + remove vendored bloat.
    for p in &plan.remove {
        let src = target.join(p);
        if !src.exists() {
            continue;
        }
        let dst = backup.join(p);
        if let Err(err) = copy_path(&src, &dst).and_then(|()| remove_path(&src)) {
            eprintln!("error: failed to migrate {p}: {err}");
            return 1;
        }
        println!("Backed up + removed {p}");
    }
    // Tidy now-empty parent dirs (ignore failures — only removes if empty).
    let _ = fs::remove_dir(target.join("bin"));
    let _ = fs::remove_dir(target.join(".claude/commands"));

    // 2. Rewrite stale hook paths in settings.
    let settings = target.join(".claude/settings.local.json");
    if let Ok(text) = fs::read_to_string(&settings)
        && text.contains("aps-planning/scripts/")
    {
        let _ = copy_path(&settings, &backup.join(".claude/settings.local.json"));
        let rewritten = text.replace("aps-planning/scripts/", ".aps/scripts/");
        if fs::write(&settings, rewritten).is_ok() {
            println!(
                "Rewrote hook paths (aps-planning/scripts/ -> .aps/scripts/) in settings.local.json"
            );
        }
    }

    // 3. Drop the stale direnv PATH_add bin line.
    if let Some((envrc, idx)) = direnv
        && let Ok(text) = fs::read_to_string(&envrc)
    {
        let _ = copy_path(&envrc, &backup.join(".envrc"));
        let kept: Vec<&str> = text
            .lines()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, l)| l)
            .collect();
        let mut body = kept.join("\n");
        if text.ends_with('\n') {
            body.push('\n');
        }
        if fs::write(&envrc, body).is_ok() {
            println!("Dropped stale `PATH_add bin` from .envrc (run `direnv allow` to refresh)");
        }
    }

    // 4. Pin cli_version when the config omits it.
    if pin && let Some(project) = config::discover_project(&target) {
        let config_path = project.root.join(".aps/config.yml");
        if let Ok(text) = fs::read_to_string(&config_path) {
            let _ = copy_path(&config_path, &backup.join(".aps/config.yml"));
            let mut body = text;
            if !body.ends_with('\n') {
                body.push('\n');
            }
            body.push_str(&format!("cli_version: {CLI_VERSION}\n"));
            if fs::write(&config_path, body).is_ok() {
                println!("Pinned cli_version: {CLI_VERSION} in .aps/config.yml");
            }
        }
    }

    println!("\nMigration complete. Backup saved to .aps/backup/{stamp}/");
    println!("Your plans/ and instruction files were not modified.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aps-migrate-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn aps_project(tag: &str) -> PathBuf {
        let root = scratch(tag);
        fs::create_dir_all(root.join("plans")).unwrap();
        fs::create_dir_all(root.join(".aps")).unwrap();
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {CLI_VERSION}\nplans_dir: plans/\n"),
        )
        .unwrap();
        root
    }

    #[test]
    fn errors_without_a_plans_dir() {
        let root = scratch("noplans");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(cmd_migrate(&root, false, true), 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn clean_project_reports_nothing_to_migrate() {
        let root = aps_project("clean");
        assert_eq!(cmd_migrate(&root, true, true), 0);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn dry_run_lists_but_does_not_remove_vendored_cli() {
        let root = aps_project("dry");
        fs::create_dir_all(root.join(".aps/bin")).unwrap();
        fs::write(root.join(".aps/bin/aps"), "#!/usr/bin/env bash\n").unwrap();

        assert_eq!(cmd_migrate(&root, false, true), 0); // dry run
        assert!(
            root.join(".aps/bin/aps").exists(),
            "dry run must not remove"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn apply_backs_up_and_removes_vendored_cli() {
        let root = aps_project("apply");
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/aps"), "#!/usr/bin/env bash\n").unwrap();
        fs::create_dir_all(root.join("lib/rules")).unwrap();
        for f in ["output.sh", "lint.sh", "orchestrate.sh"] {
            fs::write(root.join("lib").join(f), "# stub\n").unwrap();
        }

        assert_eq!(cmd_migrate(&root, true, true), 0);
        assert!(!root.join("bin/aps").exists(), "vendored bin/aps removed");
        // Ambiguous lib (missing some APS files but has extras? here only APS
        // files, so it's removed) — verify backup exists.
        let backups: Vec<_> = fs::read_dir(root.join(".aps/backup"))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(backups.len(), 1, "one timestamped backup dir");
        assert!(
            backups[0].path().join("bin/aps").exists(),
            "bin/aps backed up"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn ambiguous_lib_is_left_untouched() {
        let root = aps_project("ambig");
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/lint.sh"), "# aps\n").unwrap();
        fs::write(root.join("lib/my-own.sh"), "# not aps\n").unwrap();

        assert_eq!(cmd_migrate(&root, true, true), 0);
        assert!(
            root.join("lib/my-own.sh").exists(),
            "ambiguous lib preserved"
        );
        assert!(root.join("lib/lint.sh").exists());
        fs::remove_dir_all(&root).ok();
    }
}

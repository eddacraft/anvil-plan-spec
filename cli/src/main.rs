use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

#[allow(unused_imports)]
use eddacraft_tui as _;

mod audit;
mod config;
mod date;
mod doctor;
mod lint;
mod migrate;
mod next;
mod orchestrate;
mod parser;
mod rollup;
mod scaffold;
mod setup;
mod update;
mod wizard;

#[derive(Parser)]
#[command(
    name = "aps",
    version,
    about = "Anvil Plan Spec — native CLI and TUI wizard",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Fail (exit non-zero) when the project's cli_version pin differs from
    /// this binary. Applies to project-scoped commands (lint, next).
    #[arg(long, global = true)]
    strict: bool,
}

// Init carries the whole non-interactive wizard surface (~290 bytes); the
// enum is parsed once at startup, so boxing it buys nothing but indirection.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Command {
    /// Set up APS in this project (TUI wizard, flags, or config replay)
    Init {
        /// Skip the TUI and scaffold from flags (auto-enabled without a TTY)
        #[arg(long)]
        non_interactive: bool,
        /// Replay a configuration written by a previous init
        #[arg(long, value_name = "CONFIG_YML")]
        from: Option<PathBuf>,
        /// Profile: solo, team, or agent-operator
        #[arg(long)]
        profile: Option<String>,
        /// Project shape: single or monorepo
        #[arg(long)]
        shape: Option<String>,
        /// AI tools (comma-separated): claude-code, copilot, codex, opencode, gemini, generic
        #[arg(long, value_delimiter = ',')]
        tools: Vec<String>,
        /// Plan templates (comma-separated): quickstart, module, index, monorepo-index, index-nested
        #[arg(long, value_delimiter = ',')]
        templates: Vec<String>,
        /// Path to a custom template file to install
        #[arg(long)]
        custom_template: Option<String>,
        /// Plans directory (default: plans/)
        #[arg(long)]
        plans_dir: Option<String>,
        /// Docs location (default: docs/)
        #[arg(long)]
        docs_dir: Option<String>,
        /// Tooling root (default: .aps/)
        #[arg(long)]
        tooling_root: Option<String>,
        /// Components (comma-separated): lint-rules, aps-rules, project-context,
        /// designs-dir, decisions-dir, releases-dir
        #[arg(long, value_delimiter = ',')]
        components: Vec<String>,
        /// Hook verbosity applied to all selected tools: full, minimal, none
        #[arg(long)]
        hooks: Option<String>,
        /// Model preference applied to all selected tools: default, opus, sonnet
        #[arg(long)]
        model: Option<String>,
        /// Skip agent installation for all selected tools
        #[arg(long)]
        no_agents: bool,
    },
    /// Add optional APS pieces: picker without arguments, shortcuts otherwise
    Setup {
        /// What to set up: cli, init, agent, hooks, upgrade, all, or a tool
        /// name (claude-code, copilot, codex, opencode, gemini, generic)
        target: Option<String>,
        /// Skip confirmation for bulky/destructive flows (all, upgrade)
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Reconcile a project's generated APS footprint (templates, skill)
    Update {
        /// Project directory (default: current directory)
        dir: Option<String>,
    },
    /// Move a project onto the global binary: diagnose, remove vendored bloat
    Migrate {
        /// Project directory (default: current directory)
        dir: Option<String>,
        /// Back up and apply the changes (default: dry run)
        #[arg(long)]
        apply: bool,
        /// Preview only — never modify files (the default)
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt under --apply
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Validate APS documents under plans/
    Lint {
        /// File or directory to lint (default: plans/)
        target: Option<String>,
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Resolve the next ready work item
    Next {
        /// Optional module filter (e.g. orch, tui)
        module: Option<String>,
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Scope to one child plan in a federated tree (e.g. core)
        #[arg(long, value_name = "NAME")]
        child: Option<String>,
        /// Only items whose Packages: tags include NAME (item field, else
        /// module metadata; tagged monorepo tier)
        #[arg(long, value_name = "NAME")]
        package: Option<String>,
        /// List every ready item, grouped by package; untagged items appear
        /// under (untagged)
        #[arg(long)]
        by_package: bool,
    },
    /// Mark a Ready work item as In Progress and write its context package
    Start {
        /// Work item ID (e.g. AUTH-003) or cross-tree ref (e.g. core:AUTH-003)
        id: String,
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Scope resolution to one child plan (disambiguates a colliding ID)
        #[arg(long, value_name = "NAME")]
        child: Option<String>,
    },
    /// Mark an In Progress work item as Complete
    Complete {
        /// Work item ID (e.g. AUTH-003) or cross-tree ref (e.g. core:AUTH-003)
        id: String,
        /// Append a learning line after Validation (ORCH D-002)
        #[arg(long, value_name = "TEXT")]
        learning: Option<String>,
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Scope resolution to one child plan (disambiguates a colliding ID)
        #[arg(long, value_name = "NAME")]
        child: Option<String>,
    },
    /// Show work items and dependency arrows
    Graph {
        /// Optional module ID or file name (e.g. AUTH or auth)
        module: Option<String>,
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Scope to one child plan in a federated tree (e.g. core)
        #[arg(long, value_name = "NAME")]
        child: Option<String>,
    },
    /// Print a Markdown roll-up table for a federated (nested-plans) parent
    Rollup {
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Print modules grouped by Packages: tag instead (tagged monorepo
        /// tier; untagged modules appear under (untagged))
        #[arg(long)]
        by_package: bool,
    },
    /// Audit plan state against reality (runs Complete items' Validation)
    Audit {
        /// Optional module ID or file name to scope the audit
        module: Option<String>,
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
        /// Do not execute Validation commands (verification reports PARTIAL)
        #[arg(long)]
        no_run: bool,
        /// Staleness threshold in days (default: 60)
        #[arg(long, value_name = "DAYS")]
        stale_days: Option<u32>,
        /// Plan root directory (default: plans)
        #[arg(long, value_name = "DIR")]
        plans: Option<String>,
        /// Scope to one child plan in a federated tree (e.g. core)
        #[arg(long, value_name = "NAME")]
        child: Option<String>,
    },
    /// Diagnose migration state (global binary, cli_version, leftover CLI)
    Doctor,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => {
            println!("aps {} — pass --help for usage", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::Init {
            non_interactive,
            from,
            profile,
            shape,
            tools,
            templates,
            custom_template,
            plans_dir,
            docs_dir,
            tooling_root,
            components,
            hooks,
            model,
            no_agents,
        }) => {
            let flags = config::InitFlags {
                profile,
                shape,
                tools,
                templates,
                custom_template,
                plans_dir,
                docs_dir,
                tooling_root,
                components,
                hooks,
                model,
                no_agents,
            };
            let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

            if config::wants_tui(non_interactive, from.as_deref(), &flags, tty) {
                if let Err(err) = wizard::run() {
                    eprintln!("aps init failed: {err}");
                    std::process::exit(1);
                }
                return;
            }

            if let Err(err) = run_non_interactive_init(from.as_deref(), &flags) {
                eprintln!("aps init failed: {err}");
                std::process::exit(1);
            }
        }
        Some(Command::Setup { target, yes }) => {
            let result = match target {
                None => setup::run_picker().map_err(|err| err.to_string()),
                Some(key) => setup::run_shortcut(Path::new("."), &key, yes),
            };
            if let Err(err) = result {
                eprintln!("aps setup failed: {err}");
                std::process::exit(1);
            }
        }
        Some(Command::Lint { target, json }) => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            // Explicit target wins; otherwise discover the project's plans_dir.
            let resolved = match target {
                Some(t) => t,
                None => {
                    if let Err(err) = config::check_cli_version(&cwd, cli.strict) {
                        eprintln!("aps lint: {err}");
                        std::process::exit(2);
                    }
                    config::default_plans(&cwd)
                }
            };
            let code = lint::cmd_lint(&resolved, json);
            std::process::exit(code);
        }
        Some(Command::Next {
            module,
            plans,
            child,
            package,
            by_package,
        }) => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let resolved = match plans {
                Some(p) => p,
                None => {
                    if let Err(err) = config::check_cli_version(&cwd, cli.strict) {
                        eprintln!("aps next: {err}");
                        std::process::exit(2);
                    }
                    config::default_plans(&cwd)
                }
            };
            let code = next::cmd_next(
                &resolved,
                module.as_deref().unwrap_or(""),
                child.as_deref().unwrap_or(""),
                package.as_deref().unwrap_or(""),
                by_package,
            );
            std::process::exit(code);
        }
        Some(Command::Update { dir }) => {
            let start = PathBuf::from(dir.unwrap_or_else(|| ".".to_string()));
            std::process::exit(update::cmd_update(&start));
        }
        Some(Command::Migrate {
            dir,
            apply,
            dry_run,
            yes,
        }) => {
            let start = PathBuf::from(dir.unwrap_or_else(|| ".".to_string()));
            // --dry-run is the default and always wins over --apply if both given.
            let apply = apply && !dry_run;
            std::process::exit(migrate::cmd_migrate(&start, apply, yes));
        }
        Some(Command::Start { id, plans, child }) => {
            let resolved = resolve_plans(plans, cli.strict, "aps start");
            std::process::exit(orchestrate::cmd_start(
                &resolved,
                &id,
                child.as_deref().unwrap_or(""),
            ));
        }
        Some(Command::Complete {
            id,
            learning,
            plans,
            child,
        }) => {
            let resolved = resolve_plans(plans, cli.strict, "aps complete");
            std::process::exit(orchestrate::cmd_complete(
                &resolved,
                &id,
                learning.as_deref(),
                child.as_deref().unwrap_or(""),
            ));
        }
        Some(Command::Graph {
            module,
            plans,
            child,
        }) => {
            let resolved = resolve_plans(plans, cli.strict, "aps graph");
            std::process::exit(orchestrate::cmd_graph(
                &resolved,
                module.as_deref().unwrap_or(""),
                child.as_deref().unwrap_or(""),
            ));
        }
        Some(Command::Rollup { plans, by_package }) => {
            let resolved = resolve_plans(plans, cli.strict, "aps rollup");
            std::process::exit(rollup::cmd_rollup(&resolved, by_package));
        }
        Some(Command::Audit {
            module,
            json,
            no_run,
            stale_days,
            plans,
            child,
        }) => {
            let resolved = resolve_plans(plans, cli.strict, "aps audit");
            std::process::exit(audit::cmd_audit(
                &resolved,
                module.as_deref().unwrap_or(""),
                child.as_deref().unwrap_or(""),
                json,
                no_run,
                stale_days,
            ));
        }
        Some(Command::Doctor) => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            std::process::exit(doctor::run(&cwd));
        }
    }
}

/// Resolve a plan root for project-scoped orchestration commands: an explicit
/// `--plans` wins; otherwise discover it from `.aps/config.yml`, enforcing the
/// `cli_version` pin (exit 2 on a strict mismatch) exactly like `lint`/`next`.
fn resolve_plans(plans: Option<String>, strict: bool, label: &str) -> String {
    match plans {
        Some(p) => p,
        None => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Err(err) = config::check_cli_version(&cwd, strict) {
                eprintln!("{label}: {err}");
                std::process::exit(2);
            }
            config::default_plans(&cwd)
        }
    }
}

fn run_non_interactive_init(from: Option<&Path>, flags: &config::InitFlags) -> Result<(), String> {
    let base = match from {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
            Some(config::parse_config(&text)?)
        }
        None => None,
    };
    let selections = config::build_selections(base, flags)?;
    config::run_scaffold_console(Path::new("."), &selections)
}

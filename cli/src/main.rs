use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

#[allow(unused_imports)]
use eddacraft_tui as _;

mod config;
mod lint;
mod next;
mod parser;
mod scaffold;
mod setup;
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
        /// Plan templates (comma-separated): quickstart, module, index, monorepo-index
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
        /// designs-dir, decisions-dir
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
    },
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
        Some(Command::Next { module, plans }) => {
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
            let code = next::cmd_next(&resolved, module.as_deref().unwrap_or(""));
            std::process::exit(code);
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

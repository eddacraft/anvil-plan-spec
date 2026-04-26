use clap::{Parser, Subcommand};

#[allow(unused_imports)]
use eddacraft_tui as _;

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
}

#[derive(Subcommand)]
enum Command {
    /// Run the interactive setup wizard (TUI)
    Init,
    /// Validate APS documents under plans/
    Lint,
    /// Resolve the next ready work item
    Next {
        /// Optional module filter (e.g. orch, tui)
        module: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => {
            println!("aps {} — pass --help for usage", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::Init) => {
            eprintln!("`aps init` is not yet implemented (TUI-002 onward)");
            std::process::exit(2);
        }
        Some(Command::Lint) => {
            eprintln!("`aps lint` native port pending (TUI-005 / D-028)");
            std::process::exit(2);
        }
        Some(Command::Next { module }) => {
            let scope = module.as_deref().unwrap_or("(all modules)");
            eprintln!("`aps next {scope}` is implemented in the bash CLI; native port pending");
            std::process::exit(2);
        }
    }
}

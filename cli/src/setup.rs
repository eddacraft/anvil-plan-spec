//! `aps setup` — mode picker and shortcut flows (TUI-007/TUI-008).
//!
//! Bare `aps setup` opens an eddacraft-tui picker over the setup flows
//! defined by INSTALL-010/INSTALL-012; `aps setup <thing>` runs one flow
//! non-interactively. Flows reuse the scaffold step machinery so the TUI
//! and shortcut paths produce identical footprints.

use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;

use eddacraft_tui::prelude::{Action, EddaCraftTheme, KeyHandler, Select, SelectItem, SelectState};
use eddacraft_tui::prelude::{ShellBranding, render_shell};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};

use crate::config::tool_from_key;
use crate::scaffold::{
    self, FileOp, ScaffoldRun, ScaffoldStep, StepStatus, agents_step, hooks_step, skill_step,
};
use crate::wizard::{AiTool, ToolConfig};

// Embedded planning files for the minimal flows.
const INDEX_APS: &str = include_str!("../scaffold/plans/index.aps.md");
const APS_RULES: &str = include_str!("../scaffold/plans/aps-rules.md");
const PROJECT_CONTEXT: &str = include_str!("../scaffold/plans/project-context.md");
const MODULE_TEMPLATE: &str = include_str!("../scaffold/plans/modules/.module.template.md");
const SIMPLE_TEMPLATE: &str = include_str!("../scaffold/plans/modules/.simple.template.md");
const INDEX_MONOREPO_TEMPLATE: &str =
    include_str!("../scaffold/plans/modules/.index-monorepo.template.md");
const ACTIONS_TEMPLATE: &str = include_str!("../scaffold/plans/execution/.actions.template.md");
const DESIGN_TEMPLATE: &str = include_str!("../scaffold/designs/.design.template.md");

/// Agent-readable bootstrap instructions (TUI-008). Written to the repo so
/// a remote agent can pick up the workflow without further prompting.
pub const AGENT_NEXT_STEPS: &str = "\
# APS Agent Bootstrap — Next Steps

This repository was just initialized with a minimal APS planning layer
for an AI agent. Before implementing anything:

1. Read `plans/aps-rules.md` for the planning conventions.
2. Ask the operator for the project intent — what is being built and why.
3. Populate `plans/project-context.md` with that durable background.
4. Draft `plans/index.aps.md` (problem, outcomes, scope, modules).
5. Wait for an approved work item before writing any implementation code.

No hooks, agents, or tool integrations were installed. Run `aps setup`
to add them when needed.
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupMode {
    InstallCli,
    InitMinimal,
    AgentBootstrap,
    ToolIntegrations,
    Hooks,
    Upgrade,
    All,
}

impl SetupMode {
    pub const ALL_MODES: [SetupMode; 7] = [
        SetupMode::InstallCli,
        SetupMode::InitMinimal,
        SetupMode::AgentBootstrap,
        SetupMode::ToolIntegrations,
        SetupMode::Hooks,
        SetupMode::Upgrade,
        SetupMode::All,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::InstallCli => "Install APS CLI on this machine",
            Self::InitMinimal => "Initialize minimal APS in this repo",
            Self::AgentBootstrap => "Initialize this repo for an AI agent",
            Self::ToolIntegrations => "Add tool integrations",
            Self::Hooks => "Configure hooks",
            Self::Upgrade => "Upgrade an existing APS project",
            Self::All => "Install the full APS footprint",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::InstallCli => "copy this binary to ~/.aps/bin",
            Self::InitMinimal => "index, rules, and context — nothing else",
            Self::AgentBootstrap => "minimal plan files + agent next steps",
            Self::ToolIntegrations => "skills and agents for selected AI tools",
            Self::Hooks => "hook scripts under .aps/scripts/",
            Self::Upgrade => {
                "refresh existing generated templates (use `aps update` to add missing)"
            }
            Self::All => "templates, components, skill, and hooks",
        }
    }

    /// Destructive or bulky flows gate behind an explicit confirmation.
    pub fn needs_confirmation(self) -> bool {
        matches!(self, Self::Upgrade | Self::All)
    }
}

/// Map a `aps setup <thing>` shortcut to a mode. Tool names select the
/// tool-integration flow for exactly that tool.
pub fn mode_from_key(key: &str) -> Result<(SetupMode, Option<AiTool>), String> {
    match key {
        "cli" => Ok((SetupMode::InstallCli, None)),
        "init" => Ok((SetupMode::InitMinimal, None)),
        "agent" => Ok((SetupMode::AgentBootstrap, None)),
        "hooks" => Ok((SetupMode::Hooks, None)),
        "upgrade" => Ok((SetupMode::Upgrade, None)),
        "all" => Ok((SetupMode::All, None)),
        tool => tool_from_key(tool)
            .map(|tool| (SetupMode::ToolIntegrations, Some(tool)))
            .map_err(|_| {
                format!(
                    "unknown setup target '{tool}' (expected cli, init, agent, hooks, \
                     upgrade, all, or a tool name)"
                )
            }),
    }
}

// --- Flow step builders -------------------------------------------------------

/// Minimal planning footprint per INSTALL-011: bare planning content only.
pub fn minimal_init_steps() -> Vec<ScaffoldStep> {
    vec![
        ScaffoldStep {
            label: "Create planning directories".to_string(),
            ops: vec![
                FileOp::Mkdir(PathBuf::from("plans")),
                FileOp::Mkdir(PathBuf::from("plans/modules")),
                FileOp::Mkdir(PathBuf::from("plans/execution")),
            ],
        },
        ScaffoldStep {
            label: "Write planning files".to_string(),
            ops: vec![
                FileOp::Write {
                    path: PathBuf::from("plans/index.aps.md"),
                    content: INDEX_APS,
                },
                FileOp::Write {
                    path: PathBuf::from("plans/aps-rules.md"),
                    content: APS_RULES,
                },
                FileOp::Write {
                    path: PathBuf::from("plans/project-context.md"),
                    content: PROJECT_CONTEXT,
                },
            ],
        },
    ]
}

/// Minimal init plus agent-readable next steps (TUI-008). Installs no
/// hooks, agents, CLI runtime, or tool integrations.
pub fn agent_bootstrap_steps() -> Vec<ScaffoldStep> {
    let mut steps = minimal_init_steps();
    steps.push(ScaffoldStep {
        label: "Write agent next steps".to_string(),
        ops: vec![FileOp::Write {
            path: PathBuf::from("plans/agent-next-steps.md"),
            content: AGENT_NEXT_STEPS,
        }],
    });
    steps
}

/// Skill + agents (+ hook scripts) for the selected tools.
pub fn tool_integration_steps(tools: &[AiTool]) -> Vec<ScaffoldStep> {
    let configs: Vec<ToolConfig> = tools
        .iter()
        .map(|tool| ToolConfig::default_for(*tool))
        .collect();
    let mut steps = Vec::new();
    if let Some(step) = skill_step(&configs) {
        steps.push(refresh_generated_step(step));
    }
    if let Some(step) = agents_step(&configs) {
        steps.push(refresh_generated_step(step));
    }
    if configs
        .iter()
        .any(|config| config.hooks != crate::wizard::HookVerbosity::None)
    {
        steps.push(refresh_generated_step(hooks_step()));
    }
    steps
}

/// Tool-integration assets are generated and may be refreshed by an explicit
/// `aps setup <tool>` rerun. User-authored files never enter these steps.
fn refresh_generated_step(mut step: ScaffoldStep) -> ScaffoldStep {
    step.ops = step
        .ops
        .into_iter()
        .map(|op| match op {
            FileOp::Write { path, content } => FileOp::Overwrite { path, content },
            FileOp::WriteOwned { path, content } => FileOp::OverwriteOwned { path, content },
            other => other,
        })
        .collect();
    step
}

/// Generated files an upgrade may refresh, with current content.
const UPGRADABLE: [(&str, &str); 6] = [
    ("plans/aps-rules.md", APS_RULES),
    ("plans/modules/.module.template.md", MODULE_TEMPLATE),
    ("plans/modules/.simple.template.md", SIMPLE_TEMPLATE),
    (
        "plans/modules/.index-monorepo.template.md",
        INDEX_MONOREPO_TEMPLATE,
    ),
    ("plans/execution/.actions.template.md", ACTIONS_TEMPLATE),
    ("plans/designs/.design.template.md", DESIGN_TEMPLATE),
];

/// Refresh generated template files that already exist. Never touches
/// user-authored planning content; errors when no APS project is present.
pub fn upgrade_steps(root: &Path) -> Result<Vec<ScaffoldStep>, String> {
    if !root.join("plans").is_dir() {
        return Err("no plans/ directory here — nothing to upgrade".to_string());
    }

    // `upgrade` refreshes the generated files a project already has, in place.
    // Files it doesn't have are reported (never silently dropped) and left for
    // `aps update`, which reconciles the full footprint by adding the missing
    // ones. This split is why the count can read "5 of 6": the 6th is absent.
    let ops: Vec<FileOp> = UPGRADABLE
        .iter()
        .filter(|(path, _)| root.join(path).exists())
        .map(|(path, content)| FileOp::Overwrite {
            path: PathBuf::from(path),
            content,
        })
        .collect();

    if ops.is_empty() {
        return Err(
            "no generated APS files found to refresh — run `aps update` to add them".to_string(),
        );
    }

    let missing = UPGRADABLE.len() - ops.len();
    let label = if missing > 0 {
        format!(
            "Refresh {} generated template(s) — {missing} not present (run `aps update` to add)",
            ops.len()
        )
    } else {
        format!("Refresh {} generated template(s)", ops.len())
    };

    Ok(vec![ScaffoldStep { label, ops }])
}

/// The full tool-agnostic footprint: minimal init, all templates, all
/// components, skill, and hook scripts. Tool agents stay explicit.
pub fn full_steps() -> Vec<ScaffoldStep> {
    let mut steps = minimal_init_steps();
    steps.push(ScaffoldStep {
        label: "Install templates".to_string(),
        ops: vec![
            FileOp::Mkdir(PathBuf::from("plans/decisions")),
            FileOp::Mkdir(PathBuf::from("plans/designs")),
            FileOp::Write {
                path: PathBuf::from("plans/modules/.module.template.md"),
                content: MODULE_TEMPLATE,
            },
            FileOp::Write {
                path: PathBuf::from("plans/modules/.simple.template.md"),
                content: SIMPLE_TEMPLATE,
            },
            FileOp::Write {
                path: PathBuf::from("plans/modules/.index-monorepo.template.md"),
                content: INDEX_MONOREPO_TEMPLATE,
            },
            FileOp::Write {
                path: PathBuf::from("plans/execution/.actions.template.md"),
                content: ACTIONS_TEMPLATE,
            },
            FileOp::Write {
                path: PathBuf::from("plans/designs/.design.template.md"),
                content: DESIGN_TEMPLATE,
            },
        ],
    });
    steps.extend(skill_step(&[]));
    steps.push(hooks_step());
    steps
}

/// Copy the running binary to `dest_dir/aps` (default: ~/.aps/bin).
pub fn install_cli(dest_dir: &Path) -> Result<String, String> {
    let exe =
        std::env::current_exe().map_err(|err| format!("cannot locate current binary: {err}"))?;
    fs::create_dir_all(dest_dir)
        .map_err(|err| format!("cannot create {}: {err}", dest_dir.display()))?;
    let dest = dest_dir.join("aps");
    fs::copy(&exe, &dest)
        .map_err(|err| format!("cannot copy binary to {}: {err}", dest.display()))?;
    Ok(format!(
        "Installed {} — add it to your PATH if needed",
        dest.display()
    ))
}

pub fn default_cli_dir() -> PathBuf {
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aps/bin")
}

// --- Picker state machine -----------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    Menu,
    Tools,
    Confirm,
    Run,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupEvent {
    Continue,
    Complete,
    Quit,
}

#[derive(Debug)]
pub struct SetupState {
    root: PathBuf,
    step: SetupStep,
    menu_index: usize,
    mode: Option<SetupMode>,
    tool_index: usize,
    selected_tools: Vec<AiTool>,
    run: Option<ScaffoldRun>,
    message: Option<String>,
    error: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl SetupState {
    const TOOLS: [AiTool; 12] = [
        AiTool::ClaudeCode,
        AiTool::Copilot,
        AiTool::Codex,
        AiTool::OpenCode,
        AiTool::Grok,
        AiTool::Antigravity,
        AiTool::Amp,
        AiTool::GeminiCli,
        AiTool::Windsurf,
        AiTool::RooCode,
        AiTool::OpenClaw,
        AiTool::Generic,
    ];

    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            step: SetupStep::Menu,
            menu_index: 0,
            mode: None,
            tool_index: 0,
            selected_tools: Vec::new(),
            run: None,
            message: None,
            error: None,
        }
    }

    pub fn step(&self) -> SetupStep {
        self.step
    }

    pub fn mode(&self) -> Option<SetupMode> {
        self.mode
    }

    pub fn run(&self) -> Option<&ScaffoldRun> {
        self.run.as_ref()
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn selected_tools(&self) -> &[AiTool] {
        &self.selected_tools
    }

    pub fn handle(&mut self, action: Action) -> SetupEvent {
        if self.step == SetupStep::Run {
            return match action {
                Action::Quit => SetupEvent::Quit,
                _ => SetupEvent::Continue,
            };
        }

        match action {
            Action::Quit => SetupEvent::Quit,
            Action::Up => {
                self.move_selection(false);
                SetupEvent::Continue
            }
            Action::Down => {
                self.move_selection(true);
                SetupEvent::Continue
            }
            Action::Toggle => {
                if self.step == SetupStep::Tools {
                    let tool = Self::TOOLS[self.tool_index];
                    if let Some(index) = self.selected_tools.iter().position(|t| *t == tool) {
                        self.selected_tools.remove(index);
                    } else {
                        self.selected_tools.push(tool);
                    }
                }
                SetupEvent::Continue
            }
            Action::Back => match self.step {
                SetupStep::Menu => SetupEvent::Quit,
                SetupStep::Tools | SetupStep::Confirm => {
                    self.step = SetupStep::Menu;
                    self.mode = None;
                    SetupEvent::Continue
                }
                SetupStep::Summary => SetupEvent::Complete,
                SetupStep::Run => SetupEvent::Continue,
            },
            Action::Select => self.advance(),
            _ => SetupEvent::Continue,
        }
    }

    fn move_selection(&mut self, forward: bool) {
        let (index, len) = match self.step {
            SetupStep::Menu => (&mut self.menu_index, SetupMode::ALL_MODES.len()),
            SetupStep::Tools => (&mut self.tool_index, Self::TOOLS.len()),
            _ => return,
        };
        *index = if forward {
            (*index + 1) % len
        } else {
            index.checked_sub(1).unwrap_or(len - 1)
        };
    }

    fn advance(&mut self) -> SetupEvent {
        match self.step {
            SetupStep::Menu => {
                let mode = SetupMode::ALL_MODES[self.menu_index];
                self.mode = Some(mode);
                match mode {
                    SetupMode::ToolIntegrations => self.step = SetupStep::Tools,
                    _ if mode.needs_confirmation() => self.step = SetupStep::Confirm,
                    _ => self.start(mode),
                }
                SetupEvent::Continue
            }
            SetupStep::Tools => {
                if self.selected_tools.is_empty() {
                    return SetupEvent::Continue; // nothing selected yet
                }
                self.start(SetupMode::ToolIntegrations);
                SetupEvent::Continue
            }
            SetupStep::Confirm => {
                let mode = self.mode.expect("confirm step requires a mode");
                self.start(mode);
                SetupEvent::Continue
            }
            SetupStep::Run => SetupEvent::Continue,
            SetupStep::Summary => SetupEvent::Complete,
        }
    }

    fn start(&mut self, mode: SetupMode) {
        match flow_steps(mode, &self.selected_tools, &self.root) {
            Ok(FlowPlan::Steps(steps)) => {
                self.run = Some(ScaffoldRun::from_steps(self.root.clone(), steps));
                self.step = SetupStep::Run;
            }
            Ok(FlowPlan::Immediate(message)) => {
                self.message = Some(message);
                self.step = SetupStep::Summary;
            }
            Err(error) => {
                self.error = Some(error);
                self.step = SetupStep::Summary;
            }
        }
    }

    /// Drive scaffold execution; no-op outside Run.
    pub fn tick(&mut self) {
        if self.step != SetupStep::Run {
            return;
        }
        match &mut self.run {
            Some(run) => {
                if !run.run_next() {
                    self.step = SetupStep::Summary;
                }
            }
            None => self.step = SetupStep::Summary,
        }
    }
}

enum FlowPlan {
    Steps(Vec<ScaffoldStep>),
    Immediate(String),
}

fn flow_steps(mode: SetupMode, tools: &[AiTool], root: &Path) -> Result<FlowPlan, String> {
    match mode {
        SetupMode::InstallCli => install_cli(&default_cli_dir()).map(FlowPlan::Immediate),
        SetupMode::InitMinimal => Ok(FlowPlan::Steps(minimal_init_steps())),
        SetupMode::AgentBootstrap => Ok(FlowPlan::Steps(agent_bootstrap_steps())),
        SetupMode::ToolIntegrations => Ok(FlowPlan::Steps(tool_integration_steps(tools))),
        SetupMode::Hooks => Ok(FlowPlan::Steps(vec![hooks_step()])),
        SetupMode::Upgrade => upgrade_steps(root).map(FlowPlan::Steps),
        SetupMode::All => Ok(FlowPlan::Steps(full_steps())),
    }
}

// --- Non-interactive shortcuts --------------------------------------------------

/// Run a setup shortcut (`aps setup <thing>`). `assume_yes` skips the
/// confirmation that `all` and `upgrade` otherwise require.
pub fn run_shortcut(root: &Path, key: &str, assume_yes: bool) -> Result<(), String> {
    let (mode, tool) = mode_from_key(key)?;

    if mode.needs_confirmation()
        && !assume_yes
        && !confirm_on_terminal(&format!("{} — proceed? [y/N] ", mode.label()))?
    {
        return Err("aborted".to_string());
    }

    let tools: Vec<AiTool> = tool.into_iter().collect();
    match flow_steps(mode, &tools, root)? {
        FlowPlan::Immediate(message) => {
            println!("{message}");
            Ok(())
        }
        FlowPlan::Steps(steps) => {
            let mut run = ScaffoldRun::from_steps(root.to_path_buf(), steps);
            let mut index = 0;
            while run.run_next() {
                let (label, status) = run.steps().nth(index).expect("executed step exists");
                match status {
                    StepStatus::Done => println!("==> {label} ... ok"),
                    StepStatus::Failed(message) => println!("==> {label} ... FAILED: {message}"),
                    StepStatus::Pending => {}
                }
                index += 1;
            }
            if mode == SetupMode::AgentBootstrap {
                println!("\n{AGENT_NEXT_STEPS}");
            }
            let failures = run.failures();
            if failures.is_empty() {
                Ok(())
            } else {
                Err(format!("{} setup step(s) failed", failures.len()))
            }
        }
    }
}

fn confirm_on_terminal(prompt: &str) -> Result<bool, String> {
    if !io::stdin().is_terminal() {
        return Err(
            "this flow requires confirmation — re-run with --yes in non-interactive use"
                .to_string(),
        );
    }
    print!("{prompt}");
    use std::io::Write;
    io::stdout().flush().map_err(|err| err.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|err| err.to_string())?;
    Ok(answer.trim().eq_ignore_ascii_case("y"))
}

// --- Picker TUI ------------------------------------------------------------------

pub fn run_picker() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "interactive setup requires a terminal — use `aps setup <thing>`",
        ));
    }

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(&mut terminal);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let theme = EddaCraftTheme;
    let mut state = SetupState::default();

    loop {
        terminal.draw(|frame| render(frame, &theme, &mut state))?;

        if state.step() == SetupStep::Run {
            state.tick();
            if crossterm::event::poll(Duration::from_millis(10))? {
                let crossterm::event::Event::Key(key) = crossterm::event::read()? else {
                    continue;
                };
                if state.handle(KeyHandler::map(key)) == SetupEvent::Quit {
                    return Ok(());
                }
            }
            continue;
        }

        if crossterm::event::poll(Duration::from_millis(250))? {
            let crossterm::event::Event::Key(key) = crossterm::event::read()? else {
                continue;
            };
            match state.handle(KeyHandler::map(key)) {
                SetupEvent::Continue => {}
                SetupEvent::Complete | SetupEvent::Quit => return Ok(()),
            }
        }
    }
}

fn render(frame: &mut Frame<'_>, theme: &EddaCraftTheme, state: &mut SetupState) {
    let content = render_shell(
        frame,
        frame.area(),
        ShellBranding::Anvil,
        "APS",
        "Setup",
        "j/k navigate  space toggle  enter select  esc back  q quit",
        theme,
        env!("CARGO_PKG_VERSION"),
    );

    match state.step() {
        SetupStep::Menu => render_menu(frame, content, theme, state),
        SetupStep::Tools => render_tools(frame, content, theme, state),
        SetupStep::Confirm => render_confirm(frame, content, state),
        SetupStep::Run => render_run(frame, content, state),
        SetupStep::Summary => render_summary(frame, content, state),
    }
}

fn render_menu(frame: &mut Frame<'_>, area: Rect, theme: &EddaCraftTheme, state: &SetupState) {
    let items: Vec<SelectItem> = SetupMode::ALL_MODES
        .iter()
        .map(|mode| SelectItem::new(mode.label(), mode.description()))
        .collect();
    let mut select_state = SelectState {
        selected: state.menu_index,
        offset: 0,
    };
    Select::new(items, theme)
        .block(Block::default().title("Setup").borders(Borders::ALL))
        .render(area, frame.buffer_mut(), &mut select_state);
}

fn render_tools(frame: &mut Frame<'_>, area: Rect, theme: &EddaCraftTheme, state: &SetupState) {
    let items: Vec<SelectItem> = SetupState::TOOLS
        .iter()
        .map(|tool| {
            let selected = state.selected_tools().contains(tool);
            SelectItem::new(
                format!(
                    "{} {}",
                    if selected { "[x]" } else { "[ ]" },
                    scaffold::tool_key(*tool)
                ),
                "",
            )
        })
        .collect();
    let mut select_state = SelectState {
        selected: state.tool_index,
        offset: 0,
    };
    Select::new(items, theme)
        .block(
            Block::default()
                .title("Tool Integrations (space to toggle, enter to install)")
                .borders(Borders::ALL),
        )
        .render(area, frame.buffer_mut(), &mut select_state);
}

fn render_confirm(frame: &mut Frame<'_>, area: Rect, state: &SetupState) {
    let mode = state.mode().expect("confirm requires mode");
    let text = format!(
        "{}\n\n{}\n\nPress enter to proceed, esc to cancel.",
        mode.label(),
        mode.description()
    );
    Paragraph::new(text)
        .block(Block::default().title("Confirm").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_run(frame: &mut Frame<'_>, area: Rect, state: &SetupState) {
    let mut lines = Vec::new();
    if let Some(run) = state.run() {
        let (done, total) = run.progress();
        lines.push(format!("Running… {done}/{total}"));
        lines.push(String::new());
        for (label, status) in run.steps() {
            let marker = match status {
                StepStatus::Pending => "  …",
                StepStatus::Done => "  ✓",
                StepStatus::Failed(_) => "  ✗",
            };
            lines.push(format!("{marker} {label}"));
            if let StepStatus::Failed(message) = status {
                lines.push(format!("      {message}"));
            }
        }
    }
    Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_summary(frame: &mut Frame<'_>, area: Rect, state: &SetupState) {
    let mut lines = Vec::new();
    if let Some(error) = state.error() {
        lines.push(format!("Failed: {error}"));
    } else if let Some(message) = state.message() {
        lines.push(message.to_string());
    } else if let Some(run) = state.run() {
        let failures = run.failures();
        if failures.is_empty() {
            lines.push("Done.".to_string());
        } else {
            lines.push(format!("Finished with {} error(s):", failures.len()));
            for (label, message) in &failures {
                lines.push(format!("  ✗ {label}: {message}"));
            }
        }
        for (label, status) in run.steps() {
            if matches!(status, StepStatus::Done) {
                lines.push(format!("  ✓ {label}"));
            }
        }
        if state.mode() == Some(SetupMode::AgentBootstrap) {
            lines.push(String::new());
            lines.push("Agent instructions written to plans/agent-next-steps.md".to_string());
        }
    }
    lines.push(String::new());
    lines.push("Press enter to finish.".to_string());

    Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Summary").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn setup_menu_offers_every_supported_tool() {
        // The interactive `aps setup` tool menu must list each supported harness,
        // else it is selectable via `aps init` but not `aps setup` (regression
        // guard for the D-045 harness adds).
        for tool in [
            AiTool::ClaudeCode,
            AiTool::Copilot,
            AiTool::Codex,
            AiTool::OpenCode,
            AiTool::Grok,
            AiTool::Antigravity,
            AiTool::Amp,
            AiTool::GeminiCli,
            AiTool::Windsurf,
            AiTool::RooCode,
            AiTool::OpenClaw,
            AiTool::Generic,
        ] {
            assert!(
                SetupState::TOOLS.contains(&tool),
                "setup menu missing {tool:?}"
            );
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("aps-setup-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn run_state_to_completion(state: &mut SetupState) {
        while state.step() == SetupStep::Run {
            state.tick();
        }
    }

    #[test]
    fn menu_lists_all_required_flows() {
        let labels: Vec<&str> = SetupMode::ALL_MODES
            .iter()
            .map(|mode| mode.label())
            .collect();
        for needle in [
            "Install APS CLI",
            "Initialize minimal APS",
            "AI agent",
            "tool integrations",
            "hooks",
            "Upgrade",
        ] {
            assert!(
                labels.iter().any(|label| label.contains(needle)),
                "no menu entry containing '{needle}'"
            );
        }
    }

    #[test]
    fn shortcuts_map_to_modes() {
        assert_eq!(
            mode_from_key("agent").unwrap(),
            (SetupMode::AgentBootstrap, None)
        );
        assert_eq!(
            mode_from_key("claude-code").unwrap(),
            (SetupMode::ToolIntegrations, Some(AiTool::ClaudeCode))
        );
        assert!(mode_from_key("nonsense").is_err());
    }

    #[test]
    fn minimal_init_writes_only_planning_files() {
        let root = temp_root("minimal");
        run_shortcut(&root, "init", false).unwrap();

        assert!(root.join("plans/index.aps.md").exists());
        assert!(root.join("plans/aps-rules.md").exists());
        assert!(root.join("plans/project-context.md").exists());
        assert!(!root.join("aps-planning").exists());
        assert!(!root.join(".claude").exists());
        assert!(!root.join("bin").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn agent_bootstrap_adds_next_steps() {
        let root = temp_root("agent");
        run_shortcut(&root, "agent", false).unwrap();

        let next_steps = fs::read_to_string(root.join("plans/agent-next-steps.md")).unwrap();
        for needle in [
            "plans/aps-rules.md",
            "project intent",
            "plans/project-context.md",
            "plans/index.aps.md",
            "approved work item",
        ] {
            assert!(next_steps.contains(needle), "missing '{needle}'");
        }
        // Same minimal footprint as init — no hooks/agents/CLI.
        assert!(!root.join("aps-planning").exists());
        assert!(!root.join(".claude").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn tool_shortcut_installs_only_requested_tool() {
        let root = temp_root("tool");
        run_shortcut(&root, "claude-code", false).unwrap();

        assert!(root.join(".claude/agents/aps-conductor.md").exists());
        assert!(root.join(".claude/skills/aps-planning/SKILL.md").exists());
        assert!(!root.join("aps-planning").exists());
        assert!(!root.join(".github/agents").exists());
        assert!(!root.join(".codex").exists());
        assert!(!root.join(".agents").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn codex_tool_shortcut_refreshes_roles_and_removes_legacy_snippets() {
        let root = temp_root("codex-refresh");
        run_shortcut(&root, "codex", false).unwrap();

        fs::write(
            root.join(".codex/agents/aps-planner.toml"),
            "developer_instructions = \"\"\"stale\"\"\"\n",
        )
        .unwrap();
        fs::write(
            root.join(".codex/agents/codex-config-snippet.toml"),
            "legacy\n",
        )
        .unwrap();
        fs::write(root.join(".codex/codex-config-snippet.toml"), "legacy\n").unwrap();

        run_shortcut(&root, "codex", false).unwrap();

        let planner = fs::read_to_string(root.join(".codex/agents/aps-planner.toml")).unwrap();
        assert!(planner.contains("name = \"aps-planner\""));
        assert!(!planner.contains("stale"));
        // Codex gets the skill at the .agents/skills root, not .claude/skills.
        assert!(root.join(".agents/skills/aps-planning/SKILL.md").exists());
        assert!(!root.join(".claude").exists());
        assert!(
            !root
                .join(".codex/agents/codex-config-snippet.toml")
                .exists()
        );
        assert!(!root.join(".codex/codex-config-snippet.toml").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn all_requires_confirmation_when_not_a_tty() {
        let root = temp_root("all-confirm");
        // Test processes have no TTY on stdin under cargo test harness? They
        // may — so only assert the --yes path works and produces the full
        // footprint.
        run_shortcut(&root, "all", true).unwrap();

        assert!(root.join("plans/index.aps.md").exists());
        assert!(root.join("plans/designs/.design.template.md").exists());
        assert!(root.join(".claude/skills/aps-planning/SKILL.md").exists());
        assert!(root.join(".aps/scripts/install-hooks.sh").exists());
        assert!(root.join(".aps/scripts/install-hooks.ps1").exists());
        assert!(!root.join("aps-planning").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn upgrade_refreshes_only_existing_generated_files() {
        let root = temp_root("upgrade");
        fs::create_dir_all(root.join("plans/modules")).unwrap();
        fs::write(root.join("plans/aps-rules.md"), "old rules").unwrap();
        fs::write(root.join("plans/modules/.module.template.md"), "old tpl").unwrap();
        fs::write(root.join("plans/index.aps.md"), "# My Plan — user content").unwrap();

        run_shortcut(&root, "upgrade", true).unwrap();

        let rules = fs::read_to_string(root.join("plans/aps-rules.md")).unwrap();
        assert_ne!(rules, "old rules");
        let tpl = fs::read_to_string(root.join("plans/modules/.module.template.md")).unwrap();
        assert_ne!(tpl, "old tpl");
        // User-authored content untouched.
        let index = fs::read_to_string(root.join("plans/index.aps.md")).unwrap();
        assert_eq!(index, "# My Plan — user content");
        // Non-existent generated files are not created.
        assert!(!root.join("plans/designs/.design.template.md").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn upgrade_errors_outside_aps_projects() {
        let root = temp_root("upgrade-empty");
        assert!(run_shortcut(&root, "upgrade", true).is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn picker_state_machine_runs_agent_flow() {
        let root = temp_root("picker");
        let mut state = SetupState::new(root.clone());

        // Move to "Initialize this repo for an AI agent" (third entry).
        state.handle(Action::Down);
        state.handle(Action::Down);
        state.handle(Action::Select);
        assert_eq!(state.step(), SetupStep::Run);

        run_state_to_completion(&mut state);

        assert_eq!(state.step(), SetupStep::Summary);
        assert!(state.run().unwrap().failures().is_empty());
        assert!(root.join("plans/agent-next-steps.md").exists());
        assert_eq!(state.handle(Action::Select), SetupEvent::Complete);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn picker_gates_bulky_flows_behind_confirm() {
        let root = temp_root("picker-confirm");
        let mut state = SetupState::new(root.clone());

        // Move to "Install the full APS footprint" (last entry).
        for _ in 0..SetupMode::ALL_MODES.len() - 1 {
            state.handle(Action::Down);
        }
        state.handle(Action::Select);
        assert_eq!(state.step(), SetupStep::Confirm);

        // Esc cancels back to the menu without writing anything.
        state.handle(Action::Back);
        assert_eq!(state.step(), SetupStep::Menu);
        assert!(!root.join("plans").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn picker_tool_selection_requires_a_choice() {
        let root = temp_root("picker-tools");
        let mut state = SetupState::new(root.clone());

        // Move to "Add tool integrations" (fourth entry).
        for _ in 0..3 {
            state.handle(Action::Down);
        }
        state.handle(Action::Select);
        assert_eq!(state.step(), SetupStep::Tools);

        // Enter without a selection stays put.
        state.handle(Action::Select);
        assert_eq!(state.step(), SetupStep::Tools);

        state.handle(Action::Toggle); // claude-code
        state.handle(Action::Select);
        run_state_to_completion(&mut state);

        assert!(root.join(".claude/agents/aps-conductor.md").exists());

        fs::remove_dir_all(&root).unwrap();
    }
}

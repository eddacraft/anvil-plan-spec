use eddacraft_tui::prelude::Action;
use eddacraft_tui::prelude::{EddaCraftTheme, KeyHandler, Select, SelectItem, SelectState};
use eddacraft_tui::prelude::{ShellBranding, render_shell};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};
use std::io::{self, IsTerminal};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Solo,
    Team,
    AgentOperator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectShape {
    SingleProject,
    Monorepo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTool {
    Pnpm,
    Turbo,
    Nx,
    Lerna,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTool {
    ClaudeCode,
    Copilot,
    Codex,
    OpenCode,
    Gemini,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookVerbosity {
    Full,
    Minimal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPreference {
    Default,
    Opus,
    Sonnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolConfig {
    pub tool: AiTool,
    pub install_agents: bool,
    pub hooks: HookVerbosity,
    pub model: ModelPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Profile,
    ProjectShape,
    AiTooling,
    ToolConfig,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardEvent {
    Continue,
    Complete,
    Quit,
}

#[derive(Debug)]
pub struct WizardState {
    step: WizardStep,
    profile: Profile,
    profile_index: usize,
    project_shape: ProjectShape,
    project_shape_index: usize,
    ai_tool_index: usize,
    selected_tools: Vec<AiTool>,
    tool_configs: Vec<ToolConfig>,
    tool_config_index: usize,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            step: WizardStep::Profile,
            profile: Profile::Solo,
            profile_index: 0,
            project_shape: ProjectShape::SingleProject,
            project_shape_index: 0,
            ai_tool_index: 0,
            selected_tools: Vec::new(),
            tool_configs: Vec::new(),
            tool_config_index: 0,
        }
    }
}

impl WizardState {
    const PROFILES: [Profile; 3] = [Profile::Solo, Profile::Team, Profile::AgentOperator];
    const PROJECT_SHAPES: [ProjectShape; 2] = [ProjectShape::SingleProject, ProjectShape::Monorepo];
    const WORKSPACE_TOOLS: [WorkspaceTool; 4] = [
        WorkspaceTool::Pnpm,
        WorkspaceTool::Turbo,
        WorkspaceTool::Nx,
        WorkspaceTool::Lerna,
    ];
    const AI_TOOLS: [AiTool; 6] = [
        AiTool::ClaudeCode,
        AiTool::Copilot,
        AiTool::Codex,
        AiTool::OpenCode,
        AiTool::Gemini,
        AiTool::Generic,
    ];

    pub fn step(&self) -> WizardStep {
        self.step
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn set_project_shape(&mut self, project_shape: ProjectShape) {
        self.project_shape = project_shape;
        self.project_shape_index = Self::PROJECT_SHAPES
            .iter()
            .position(|shape| *shape == project_shape)
            .unwrap_or(0);
    }

    pub fn workspace_tools(&self) -> &[WorkspaceTool] {
        match self.project_shape {
            ProjectShape::SingleProject => &[],
            ProjectShape::Monorepo => &Self::WORKSPACE_TOOLS,
        }
    }

    pub fn toggle_tool(&mut self, tool: AiTool) {
        if let Some(index) = self
            .selected_tools
            .iter()
            .position(|selected| *selected == tool)
        {
            self.selected_tools.remove(index);
            self.tool_configs.retain(|config| config.tool != tool);
            self.tool_config_index = self
                .tool_config_index
                .min(self.tool_configs.len().saturating_sub(1));
        } else {
            self.selected_tools.push(tool);
            self.selected_tools
                .sort_by_key(|tool| Self::ai_tool_order(*tool));
            self.tool_configs.push(ToolConfig::default_for(tool));
            self.tool_configs
                .sort_by_key(|config| Self::ai_tool_order(config.tool));
        }
    }

    pub fn selected_tools(&self) -> &[AiTool] {
        &self.selected_tools
    }

    pub fn tool_configs(&self) -> &[ToolConfig] {
        &self.tool_configs
    }

    pub fn handle(&mut self, action: Action) -> WizardEvent {
        match action {
            Action::Quit => WizardEvent::Quit,
            Action::Up => {
                self.move_selection(false);
                WizardEvent::Continue
            }
            Action::Down => {
                self.move_selection(true);
                WizardEvent::Continue
            }
            Action::Toggle => {
                self.toggle_current();
                WizardEvent::Continue
            }
            Action::Left => {
                self.cycle_current_hooks(false);
                WizardEvent::Continue
            }
            Action::Right => {
                self.cycle_current_hooks(true);
                WizardEvent::Continue
            }
            Action::Character('a') => {
                self.toggle_current_agents();
                WizardEvent::Continue
            }
            Action::Character('m') => {
                self.cycle_current_model();
                WizardEvent::Continue
            }
            Action::Back => {
                self.back();
                WizardEvent::Continue
            }
            Action::Select => self.advance(),
            _ => WizardEvent::Continue,
        }
    }

    fn advance(&mut self) -> WizardEvent {
        self.step = match self.step {
            WizardStep::Profile => WizardStep::ProjectShape,
            WizardStep::ProjectShape => WizardStep::AiTooling,
            WizardStep::AiTooling if self.selected_tools.is_empty() => WizardStep::Done,
            WizardStep::AiTooling => WizardStep::ToolConfig,
            WizardStep::ToolConfig => WizardStep::Done,
            WizardStep::Done => return WizardEvent::Complete,
        };

        WizardEvent::Continue
    }

    fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Profile => WizardStep::Profile,
            WizardStep::ProjectShape => WizardStep::Profile,
            WizardStep::AiTooling => WizardStep::ProjectShape,
            WizardStep::ToolConfig => WizardStep::AiTooling,
            WizardStep::Done => WizardStep::ToolConfig,
        };
    }

    fn move_selection(&mut self, forward: bool) {
        match self.step {
            WizardStep::Profile => {
                self.profile_index = move_index(self.profile_index, Self::PROFILES.len(), forward);
                self.profile = Self::PROFILES[self.profile_index];
            }
            WizardStep::ProjectShape => {
                self.project_shape_index = move_index(
                    self.project_shape_index,
                    Self::PROJECT_SHAPES.len(),
                    forward,
                );
                self.set_project_shape(Self::PROJECT_SHAPES[self.project_shape_index]);
            }
            WizardStep::AiTooling => {
                self.ai_tool_index = move_index(self.ai_tool_index, Self::AI_TOOLS.len(), forward);
            }
            WizardStep::ToolConfig => {
                self.tool_config_index =
                    move_index(self.tool_config_index, self.tool_configs.len(), forward);
            }
            WizardStep::Done => {}
        }
    }

    fn toggle_current(&mut self) {
        if self.step == WizardStep::AiTooling {
            self.toggle_tool(Self::AI_TOOLS[self.ai_tool_index]);
        }
    }

    fn current_tool_config_mut(&mut self) -> Option<&mut ToolConfig> {
        if self.step != WizardStep::ToolConfig {
            return None;
        }

        self.tool_configs.get_mut(self.tool_config_index)
    }

    fn toggle_current_agents(&mut self) {
        if let Some(config) = self.current_tool_config_mut() {
            config.install_agents = !config.install_agents;
        }
    }

    fn cycle_current_hooks(&mut self, forward: bool) {
        if let Some(config) = self.current_tool_config_mut() {
            config.hooks = config.hooks.next(forward);
        }
    }

    fn cycle_current_model(&mut self) {
        if let Some(config) = self.current_tool_config_mut() {
            config.model = config.model.next();
        }
    }

    fn ai_tool_order(tool: AiTool) -> usize {
        Self::AI_TOOLS
            .iter()
            .position(|candidate| *candidate == tool)
            .unwrap_or(Self::AI_TOOLS.len())
    }
}

impl ToolConfig {
    fn default_for(tool: AiTool) -> Self {
        Self {
            tool,
            install_agents: tool.supports_agents(),
            hooks: if tool == AiTool::Generic {
                HookVerbosity::None
            } else {
                HookVerbosity::Minimal
            },
            model: tool.default_model(),
        }
    }
}

pub fn run() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "interactive TUI requires a terminal",
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
    let mut state = WizardState::default();

    loop {
        terminal.draw(|frame| render(frame, &theme, &mut state))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            let crossterm::event::Event::Key(key) = crossterm::event::read()? else {
                continue;
            };
            match state.handle(KeyHandler::map(key)) {
                WizardEvent::Continue => {}
                WizardEvent::Complete | WizardEvent::Quit => return Ok(()),
            }
        }
    }
}

fn render(frame: &mut Frame<'_>, theme: &EddaCraftTheme, state: &mut WizardState) {
    let content = render_shell(
        frame,
        frame.area(),
        ShellBranding::Anvil,
        "APS",
        "Init",
        "j/k navigate  space toggle  a agents  m model  h/l hooks  enter next  q quit",
        theme,
        env!("CARGO_PKG_VERSION"),
    );

    match state.step() {
        WizardStep::Profile => render_profile(frame, content, theme, state),
        WizardStep::ProjectShape => render_project_shape(frame, content, theme, state),
        WizardStep::AiTooling => render_ai_tooling(frame, content, theme, state),
        WizardStep::ToolConfig => render_tool_config(frame, content, theme, state),
        WizardStep::Done => render_done(frame, content, state),
    }
}

fn render_profile(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    render_select(
        frame,
        area,
        theme,
        "Profile",
        vec![
            SelectItem::new("Solo dev", "Small project defaults"),
            SelectItem::new("Team", "Shared conventions and release planning"),
            SelectItem::new(
                "AI agent operator",
                "Agent-first planning and review defaults",
            ),
        ],
        state.profile_index,
    );
}

fn render_project_shape(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let chunks = Layout::vertical([Constraint::Length(6), Constraint::Min(3)]).split(area);
    render_select(
        frame,
        chunks[0],
        theme,
        "Project Shape",
        vec![
            SelectItem::new("Single project", "One plans/ tree at the repository root"),
            SelectItem::new(
                "Monorepo",
                "Detect workspace tooling and package-level plans",
            ),
        ],
        state.project_shape_index,
    );

    let detail = if state.project_shape == ProjectShape::Monorepo {
        let tools = state
            .workspace_tools()
            .iter()
            .map(|tool| tool.label())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Workspace detection: {tools}\nPer-package plans: enabled in later wizard steps")
    } else {
        "Workspace detection hidden for single-project setup".to_string()
    };
    Paragraph::new(detail)
        .block(
            Block::default()
                .title("Conditional Options")
                .borders(Borders::ALL),
        )
        .render(chunks[1], frame.buffer_mut());
}

fn render_ai_tooling(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let items = WizardState::AI_TOOLS
        .iter()
        .map(|tool| {
            let selected = state.selected_tools().contains(tool);
            SelectItem::new(
                format!("{} {}", if selected { "[x]" } else { "[ ]" }, tool.label()),
                tool.description(),
            )
        })
        .collect();

    render_select(frame, area, theme, "AI Tooling", items, state.ai_tool_index);
}

fn render_tool_config(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let items = state
        .tool_configs()
        .iter()
        .map(|config| {
            SelectItem::new(
                config.tool.label(),
                format!(
                    "agents: {}  hooks: {}  model: {}",
                    if config.install_agents { "yes" } else { "no" },
                    config.hooks.label(),
                    config.model.label()
                ),
            )
        })
        .collect();

    render_select(
        frame,
        area,
        theme,
        "Per-Tool Config",
        items,
        state.tool_config_index,
    );
}

fn render_done(frame: &mut Frame<'_>, area: Rect, state: &WizardState) {
    let tools = if state.selected_tools().is_empty() {
        "none".to_string()
    } else {
        state
            .selected_tools()
            .iter()
            .map(|tool| tool.label())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let summary = format!(
        "Profile: {}\nProject shape: {}\nAI tools: {}\n\nScaffold execution starts in TUI-004.",
        state.profile().label(),
        state.project_shape.label(),
        tools
    );

    Paragraph::new(summary)
        .block(Block::default().title("Summary").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_select(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    title: &'static str,
    items: Vec<SelectItem>,
    selected: usize,
) {
    let mut select_state = SelectState {
        selected,
        offset: 0,
    };
    let select =
        Select::new(items, theme).block(Block::default().title(title).borders(Borders::ALL));
    select.render(area, frame.buffer_mut(), &mut select_state);
}

impl Profile {
    fn label(self) -> &'static str {
        match self {
            Self::Solo => "solo dev",
            Self::Team => "team",
            Self::AgentOperator => "AI agent operator",
        }
    }
}

impl ProjectShape {
    fn label(self) -> &'static str {
        match self {
            Self::SingleProject => "single project",
            Self::Monorepo => "monorepo",
        }
    }
}

impl WorkspaceTool {
    fn label(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Turbo => "turbo",
            Self::Nx => "nx",
            Self::Lerna => "lerna",
        }
    }
}

impl AiTool {
    fn label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Copilot => "Copilot",
            Self::Codex => "Codex",
            Self::OpenCode => "OpenCode",
            Self::Gemini => "Gemini",
            Self::Generic => "Generic",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::ClaudeCode => "skills, agents, and optional hooks",
            Self::Copilot => "repo instructions and custom agents",
            Self::Codex => "skills plus agent role config",
            Self::OpenCode => "skills and subagents",
            Self::Gemini => "workspace-linked skills",
            Self::Generic => "plain APS files only",
        }
    }

    fn supports_agents(self) -> bool {
        !matches!(self, Self::Generic)
    }

    fn default_model(self) -> ModelPreference {
        match self {
            Self::ClaudeCode | Self::OpenCode => ModelPreference::Opus,
            Self::Codex => ModelPreference::Sonnet,
            Self::Copilot | Self::Gemini => ModelPreference::Default,
            Self::Generic => ModelPreference::Default,
        }
    }
}

impl HookVerbosity {
    fn next(self, forward: bool) -> Self {
        let index = match self {
            Self::Full => 0,
            Self::Minimal => 1,
            Self::None => 2,
        };
        match move_index(index, 3, forward) {
            0 => Self::Full,
            1 => Self::Minimal,
            _ => Self::None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Minimal => "minimal",
            Self::None => "none",
        }
    }
}

impl ModelPreference {
    fn next(self) -> Self {
        match self {
            Self::Default => Self::Opus,
            Self::Opus => Self::Sonnet,
            Self::Sonnet => Self::Default,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Opus => "opus",
            Self::Sonnet => "sonnet",
        }
    }
}

fn move_index(current: usize, len: usize, forward: bool) -> usize {
    if len == 0 {
        return 0;
    }

    if forward {
        (current + 1) % len
    } else {
        current.checked_sub(1).unwrap_or(len - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_starts_on_profile() {
        let state = WizardState::default();

        assert_eq!(state.step(), WizardStep::Profile);
        assert_eq!(state.profile(), Profile::Solo);
    }

    #[test]
    fn select_advances_and_escape_goes_back() {
        let mut state = WizardState::default();

        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::ProjectShape);

        assert_eq!(state.handle(Action::Back), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Profile);
    }

    #[test]
    fn quit_action_exits_cleanly() {
        let mut state = WizardState::default();

        assert_eq!(state.handle(Action::Quit), WizardEvent::Quit);
    }

    #[test]
    fn monorepo_options_only_show_for_monorepos() {
        let mut state = WizardState::default();

        assert_eq!(state.workspace_tools(), &[]);
        state.set_project_shape(ProjectShape::Monorepo);

        assert_eq!(
            state.workspace_tools(),
            &[
                WorkspaceTool::Pnpm,
                WorkspaceTool::Turbo,
                WorkspaceTool::Nx,
                WorkspaceTool::Lerna,
            ]
        );
    }

    #[test]
    fn tool_config_only_includes_selected_tools() {
        let mut state = WizardState::default();

        state.toggle_tool(AiTool::ClaudeCode);
        state.toggle_tool(AiTool::OpenCode);

        assert_eq!(
            state.selected_tools(),
            &[AiTool::ClaudeCode, AiTool::OpenCode]
        );
        let config_tools = state
            .tool_configs()
            .iter()
            .map(|config| config.tool)
            .collect::<Vec<_>>();
        assert_eq!(config_tools, vec![AiTool::ClaudeCode, AiTool::OpenCode]);
    }

    #[test]
    fn space_toggles_current_ai_tool() {
        let mut state = WizardState::default();
        state.handle(Action::Select);
        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::AiTooling);
        assert_eq!(state.selected_tools(), &[]);

        state.handle(Action::Toggle);

        assert_eq!(state.selected_tools(), &[AiTool::ClaudeCode]);
    }

    #[test]
    fn selected_tools_get_persisted_config_defaults() {
        let mut state = WizardState::default();

        state.toggle_tool(AiTool::ClaudeCode);
        state.toggle_tool(AiTool::Generic);

        assert_eq!(
            state.tool_configs(),
            &[
                ToolConfig {
                    tool: AiTool::ClaudeCode,
                    install_agents: true,
                    hooks: HookVerbosity::Minimal,
                    model: ModelPreference::Opus,
                },
                ToolConfig {
                    tool: AiTool::Generic,
                    install_agents: false,
                    hooks: HookVerbosity::None,
                    model: ModelPreference::Default,
                },
            ]
        );
    }

    #[test]
    fn tool_config_step_updates_current_tool_settings() {
        let mut state = WizardState::default();

        state.toggle_tool(AiTool::ClaudeCode);
        state.handle(Action::Select);
        state.handle(Action::Select);
        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::ToolConfig);

        state.handle(Action::Character('a'));
        state.handle(Action::Right);
        state.handle(Action::Character('m'));

        assert_eq!(
            state.tool_configs()[0],
            ToolConfig {
                tool: AiTool::ClaudeCode,
                install_agents: false,
                hooks: HookVerbosity::None,
                model: ModelPreference::Sonnet,
            }
        );
    }

    #[test]
    fn done_screen_renders_before_completion() {
        let mut state = WizardState::default();

        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Done);

        assert_eq!(state.handle(Action::Select), WizardEvent::Complete);
    }
}

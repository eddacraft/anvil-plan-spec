use eddacraft_tui::prelude::Action;
use eddacraft_tui::prelude::TextInputState;
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
pub enum Template {
    Quickstart,
    Module,
    Index,
    MonorepoIndex,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Profile,
    ProjectShape,
    Templates,
    AiTooling,
    ToolConfig,
    Paths,
    Components,
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
    selected_templates: Vec<Template>,
    template_index: usize,
    templates_touched: bool,
    custom_template_path: TextInputState,
    editing_custom_template: bool,
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
            selected_templates: Vec::new(),
            template_index: 0,
            templates_touched: false,
            custom_template_path: TextInputState::default(),
            editing_custom_template: false,
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
    const TEMPLATES: [Template; 5] = [
        Template::Quickstart,
        Template::Module,
        Template::Index,
        Template::MonorepoIndex,
        Template::Custom,
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

    pub fn selected_templates(&self) -> &[Template] {
        &self.selected_templates
    }

    pub fn custom_template_path(&self) -> &str {
        &self.custom_template_path.value
    }

    pub fn is_editing(&self) -> bool {
        self.step == WizardStep::Templates && self.editing_custom_template
    }

    pub fn toggle_template(&mut self, template: Template) {
        self.templates_touched = true;
        if let Some(index) = self
            .selected_templates
            .iter()
            .position(|selected| *selected == template)
        {
            self.selected_templates.remove(index);
            if template == Template::Custom {
                self.editing_custom_template = false;
            }
        } else {
            self.selected_templates.push(template);
            self.selected_templates
                .sort_by_key(|template| Self::template_order(*template));
            if template == Template::Custom {
                self.editing_custom_template = true;
            }
        }
    }

    fn default_templates(profile: Profile, shape: ProjectShape) -> Vec<Template> {
        match (profile, shape) {
            (_, ProjectShape::Monorepo) => vec![Template::Module, Template::MonorepoIndex],
            (Profile::Solo, ProjectShape::SingleProject) => vec![Template::Quickstart],
            (_, ProjectShape::SingleProject) => vec![Template::Module, Template::Index],
        }
    }

    fn handle_editing(&mut self, action: Action) -> WizardEvent {
        match action {
            Action::Quit => return WizardEvent::Quit,
            Action::Character(c) => self.custom_template_path.insert(c),
            Action::Backspace => self.custom_template_path.backspace(),
            Action::Delete => self.custom_template_path.delete(),
            Action::Left => self.custom_template_path.move_left(),
            Action::Right => self.custom_template_path.move_right(),
            Action::Home => self.custom_template_path.home(),
            Action::End => self.custom_template_path.end(),
            Action::Select => self.editing_custom_template = false,
            Action::Back => {
                self.editing_custom_template = false;
                if self.custom_template_path.value.is_empty() {
                    self.selected_templates
                        .retain(|template| *template != Template::Custom);
                }
            }
            _ => {}
        }

        WizardEvent::Continue
    }

    pub fn handle(&mut self, action: Action) -> WizardEvent {
        if self.is_editing() {
            return self.handle_editing(action);
        }

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
            WizardStep::ProjectShape => WizardStep::Templates,
            WizardStep::Templates => WizardStep::AiTooling,
            WizardStep::AiTooling if self.selected_tools.is_empty() => WizardStep::Paths,
            WizardStep::AiTooling => WizardStep::ToolConfig,
            WizardStep::ToolConfig => WizardStep::Paths,
            WizardStep::Paths => WizardStep::Components,
            WizardStep::Components => WizardStep::Done,
            WizardStep::Done => return WizardEvent::Complete,
        };

        if self.step == WizardStep::Templates && !self.templates_touched {
            self.selected_templates = Self::default_templates(self.profile, self.project_shape);
        }

        WizardEvent::Continue
    }

    fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Profile => WizardStep::Profile,
            WizardStep::ProjectShape => WizardStep::Profile,
            WizardStep::Templates => WizardStep::ProjectShape,
            WizardStep::AiTooling => WizardStep::Templates,
            WizardStep::ToolConfig => WizardStep::AiTooling,
            WizardStep::Paths if self.selected_tools.is_empty() => WizardStep::AiTooling,
            WizardStep::Paths => WizardStep::ToolConfig,
            WizardStep::Components => WizardStep::Paths,
            WizardStep::Done => WizardStep::Components,
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
            WizardStep::Templates => {
                self.template_index =
                    move_index(self.template_index, Self::TEMPLATES.len(), forward);
            }
            // Selection state for these steps lands with their sections
            // (TUI-003 Tasks 3-4).
            WizardStep::Paths | WizardStep::Components => {}
            WizardStep::Done => {}
        }
    }

    fn toggle_current(&mut self) {
        match self.step {
            WizardStep::AiTooling => self.toggle_tool(Self::AI_TOOLS[self.ai_tool_index]),
            WizardStep::Templates => self.toggle_template(Self::TEMPLATES[self.template_index]),
            _ => {}
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

    fn template_order(template: Template) -> usize {
        Self::TEMPLATES
            .iter()
            .position(|candidate| *candidate == template)
            .unwrap_or(Self::TEMPLATES.len())
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
        // Placeholder panes until rendering lands in TUI-003 Task 5.
        WizardStep::Templates => render_placeholder(frame, content, "Templates"),
        WizardStep::Paths => render_placeholder(frame, content, "Paths"),
        WizardStep::Components => render_placeholder(frame, content, "Components"),
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

fn render_placeholder(frame: &mut Frame<'_>, area: Rect, title: &'static str) {
    Paragraph::new("Section under construction — Enter to continue, Esc to go back")
        .block(Block::default().title(title).borders(Borders::ALL))
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

        for _ in 0..6 {
            assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        }
        assert_eq!(state.step(), WizardStep::Done);

        assert_eq!(state.handle(Action::Select), WizardEvent::Complete);
    }

    #[test]
    fn templates_step_sits_between_project_shape_and_ai_tooling() {
        let mut state = WizardState::default();

        state.handle(Action::Select);
        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::Templates);

        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::AiTooling);
    }

    #[test]
    fn paths_and_components_follow_tool_config() {
        let mut state = WizardState::default();
        state.toggle_tool(AiTool::ClaudeCode);

        for _ in 0..4 {
            state.handle(Action::Select);
        }
        assert_eq!(state.step(), WizardStep::ToolConfig);

        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Paths);

        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Components);

        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Done);
    }

    #[test]
    fn empty_tool_selection_skips_tool_config_but_not_paths() {
        let mut state = WizardState::default();

        for _ in 0..4 {
            state.handle(Action::Select);
        }

        assert_eq!(state.step(), WizardStep::Paths);
    }

    #[test]
    fn back_navigation_reverses_the_new_step_order() {
        let mut state = WizardState::default();
        state.toggle_tool(AiTool::ClaudeCode);

        for _ in 0..7 {
            state.handle(Action::Select);
        }
        assert_eq!(state.step(), WizardStep::Done);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Components);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Paths);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::ToolConfig);
    }

    fn at_templates(state: &mut WizardState) {
        state.handle(Action::Select);
        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn template_defaults_follow_profile_and_shape() {
        // single + solo -> quickstart
        let mut solo = WizardState::default();
        at_templates(&mut solo);
        assert_eq!(solo.selected_templates(), &[Template::Quickstart]);

        // single + team -> index + module
        let mut team = WizardState::default();
        team.handle(Action::Down);
        at_templates(&mut team);
        assert_eq!(
            team.selected_templates(),
            &[Template::Module, Template::Index]
        );

        // monorepo -> module + monorepo-index
        let mut mono = WizardState::default();
        mono.handle(Action::Select);
        mono.handle(Action::Down);
        mono.handle(Action::Select);
        assert_eq!(mono.step(), WizardStep::Templates);
        assert_eq!(
            mono.selected_templates(),
            &[Template::Module, Template::MonorepoIndex]
        );
    }

    #[test]
    fn template_defaults_recompute_until_first_toggle() {
        let mut state = WizardState::default();
        at_templates(&mut state);
        assert_eq!(state.selected_templates(), &[Template::Quickstart]);

        state.handle(Action::Back);
        state.handle(Action::Down); // switch to monorepo
        state.handle(Action::Select);

        assert_eq!(
            state.selected_templates(),
            &[Template::Module, Template::MonorepoIndex]
        );
    }

    #[test]
    fn first_toggle_freezes_template_defaults() {
        let mut state = WizardState::default();
        at_templates(&mut state);

        state.handle(Action::Toggle); // deselect highlighted quickstart
        assert_eq!(state.selected_templates(), &[]);

        state.handle(Action::Back);
        state.handle(Action::Down); // switch to monorepo
        state.handle(Action::Select);

        assert_eq!(state.selected_templates(), &[]);
    }

    #[test]
    fn custom_template_row_activates_text_input() {
        let mut state = WizardState::default();
        at_templates(&mut state);

        assert!(!state.is_editing());
        state.handle(Action::Up); // wrap to last row: custom
        state.handle(Action::Toggle);
        assert!(state.is_editing());

        state.handle(Action::Character('t'));
        state.handle(Action::Character('p'));
        state.handle(Action::Backspace);
        assert_eq!(state.custom_template_path(), "t");

        state.handle(Action::Select); // confirm input
        assert!(!state.is_editing());
        assert!(state.selected_templates().contains(&Template::Custom));
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn escape_with_empty_custom_path_deselects_custom() {
        let mut state = WizardState::default();
        at_templates(&mut state);

        state.handle(Action::Up);
        state.handle(Action::Toggle);
        assert!(state.is_editing());

        state.handle(Action::Back);

        assert!(!state.is_editing());
        assert!(!state.selected_templates().contains(&Template::Custom));
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn back_from_paths_skips_tool_config_when_no_tools_selected() {
        let mut state = WizardState::default();

        for _ in 0..4 {
            state.handle(Action::Select);
        }
        assert_eq!(state.step(), WizardStep::Paths);

        state.handle(Action::Back);

        assert_eq!(state.step(), WizardStep::AiTooling);
    }
}

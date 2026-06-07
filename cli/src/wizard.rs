use eddacraft_tui::prelude::Action;
use eddacraft_tui::prelude::{EddaCraftTheme, KeyHandler, Select, SelectItem, SelectState};
use eddacraft_tui::prelude::{TextInput, TextInputState};
use eddacraft_tui::prelude::{ShellBranding, render_shell};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
pub enum Component {
    LintRules,
    ApsRules,
    ProjectContext,
    DesignsDir,
    DecisionsDir,
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
    /// Once the user toggles any template row, defaults stop recomputing —
    /// even if they later go back and change profile or shape. Manual
    /// selections always win over recomputed defaults (council C-008).
    templates_touched: bool,
    custom_template_path: TextInputState,
    editing_custom_template: bool,
    path_inputs: [TextInputState; 3],
    path_focus: usize,
    path_error: Option<&'static str>,
    selected_components: Vec<Component>,
    component_index: usize,
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
            path_inputs: [
                path_input("plans/"),
                path_input("docs/"),
                path_input(".aps/"),
            ],
            path_focus: 0,
            path_error: None,
            selected_components: WizardState::COMPONENTS.to_vec(),
            component_index: 0,
        }
    }
}

fn path_input(value: &str) -> TextInputState {
    let mut input = TextInputState::default();
    input.value = value.to_string();
    input.set_cursor(value.len());
    input
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
    const COMPONENTS: [Component; 5] = [
        Component::LintRules,
        Component::ApsRules,
        Component::ProjectContext,
        Component::DesignsDir,
        Component::DecisionsDir,
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
        match self.step {
            WizardStep::Templates => self.editing_custom_template,
            WizardStep::Paths => true,
            _ => false,
        }
    }

    pub fn plans_dir(&self) -> &str {
        &self.path_inputs[0].value
    }

    pub fn docs_dir(&self) -> &str {
        &self.path_inputs[1].value
    }

    pub fn tooling_root(&self) -> &str {
        &self.path_inputs[2].value
    }

    pub fn path_focus(&self) -> usize {
        self.path_focus
    }

    pub fn selected_components(&self) -> &[Component] {
        &self.selected_components
    }

    pub fn toggle_component(&mut self, component: Component) {
        if let Some(index) = self
            .selected_components
            .iter()
            .position(|selected| *selected == component)
        {
            self.selected_components.remove(index);
        } else {
            self.selected_components.push(component);
            self.selected_components
                .sort_by_key(|component| Self::component_order(*component));
        }
    }

    fn has_component(&self, component: Component) -> bool {
        self.selected_components.contains(&component)
    }

    /// Render the directory structure implied by the current selections.
    /// Pure function of wizard state so the preview can update live while
    /// path fields are edited.
    pub fn directory_preview(&self) -> String {
        let mut plans_children: Vec<String> = Vec::new();
        for template in &self.selected_templates {
            plans_children.push(match template {
                Template::Quickstart => "quickstart.aps.md".to_string(),
                Template::Index => "index.aps.md".to_string(),
                Template::MonorepoIndex => "index.aps.md (monorepo)".to_string(),
                Template::Module => "modules/".to_string(),
                Template::Custom => format!("{} (custom)", self.custom_template_path()),
            });
        }
        if self.has_component(Component::ApsRules) {
            plans_children.push("aps-rules.md".to_string());
        }
        if self.has_component(Component::ProjectContext) {
            plans_children.push("project-context.md".to_string());
        }
        plans_children.push("execution/".to_string());
        if self.has_component(Component::DesignsDir) {
            plans_children.push("designs/".to_string());
        }
        if self.has_component(Component::DecisionsDir) {
            plans_children.push("decisions/".to_string());
        }

        let mut tooling_children: Vec<String> = Vec::new();
        if self.has_component(Component::LintRules) {
            tooling_children.push("lint/".to_string());
        }

        let mut lines: Vec<String> = Vec::new();
        push_tree(&mut lines, self.plans_dir(), &plans_children);
        push_tree(&mut lines, self.docs_dir(), &[]);
        push_tree(&mut lines, self.tooling_root(), &tooling_children);
        lines.join("\n")
    }

    pub fn toggle_template(&mut self, template: Template) {
        // Note: custom_template_path keeps its value across Custom
        // toggle-off/on cycles so an accidental deselect doesn't lose a
        // typed path (council C-011).
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

    fn active_input_mut(&mut self) -> &mut TextInputState {
        match self.step {
            WizardStep::Paths => &mut self.path_inputs[self.path_focus],
            WizardStep::Templates => &mut self.custom_template_path,
            // is_editing() restricts handle_editing to the two steps above.
            _ => unreachable!("handle_editing reached outside an editing step"),
        }
    }

    /// Deselect Custom when its path is blank — a pathless custom template
    /// has nothing to install. Applied on both confirm and cancel.
    fn drop_custom_if_blank(&mut self) {
        if self.custom_template_path.value.trim().is_empty() {
            self.selected_templates
                .retain(|template| *template != Template::Custom);
        }
    }

    fn handle_editing(&mut self, action: Action) -> WizardEvent {
        match action {
            Action::Quit => return WizardEvent::Quit,
            Action::Character(c) if is_text_char(c) => {
                self.path_error = None;
                self.active_input_mut().insert(c);
            }
            // Control, bidi-override, and zero-width characters are dropped:
            // they corrupt stored paths and enable filename spoofing.
            Action::Character(_) => {}
            Action::Backspace => {
                self.path_error = None;
                self.active_input_mut().backspace();
            }
            Action::Delete => {
                self.path_error = None;
                self.active_input_mut().delete();
            }
            Action::Left => self.active_input_mut().move_left(),
            Action::Right => self.active_input_mut().move_right(),
            Action::Home => self.active_input_mut().home(),
            Action::End => self.active_input_mut().end(),
            Action::Up if self.step == WizardStep::Paths => {
                self.path_focus = move_index(self.path_focus, self.path_inputs.len(), false);
            }
            Action::Down if self.step == WizardStep::Paths => {
                self.path_focus = move_index(self.path_focus, self.path_inputs.len(), true);
            }
            Action::Select => match self.step {
                WizardStep::Paths => {
                    if self
                        .path_inputs
                        .iter()
                        .any(|input| input.value.trim().is_empty())
                    {
                        self.path_error = Some("Path fields cannot be empty");
                    } else {
                        return self.advance();
                    }
                }
                _ => {
                    self.editing_custom_template = false;
                    self.drop_custom_if_blank();
                }
            },
            Action::Back => match self.step {
                WizardStep::Paths => self.back(),
                _ => {
                    self.editing_custom_template = false;
                    self.drop_custom_if_blank();
                }
            },
            // Up/Down are intentionally inert while editing the custom
            // template path — there is no second field to focus.
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

        self.apply_template_defaults();

        WizardEvent::Continue
    }

    /// Recompute template defaults whenever the Templates step is entered —
    /// forward or backward — until the user makes a manual selection.
    fn apply_template_defaults(&mut self) {
        if self.step == WizardStep::Templates && !self.templates_touched {
            self.selected_templates = Self::default_templates(self.profile, self.project_shape);
        }
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

        self.apply_template_defaults();
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
            WizardStep::Components => {
                self.component_index =
                    move_index(self.component_index, Self::COMPONENTS.len(), forward);
            }
            // Paths focus movement is handled by handle_editing.
            WizardStep::Paths => {}
            WizardStep::Done => {}
        }
    }

    fn toggle_current(&mut self) {
        match self.step {
            WizardStep::AiTooling => self.toggle_tool(Self::AI_TOOLS[self.ai_tool_index]),
            WizardStep::Templates => self.toggle_template(Self::TEMPLATES[self.template_index]),
            WizardStep::Components => {
                self.toggle_component(Self::COMPONENTS[self.component_index])
            }
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

    fn component_order(component: Component) -> usize {
        Self::COMPONENTS
            .iter()
            .position(|candidate| *candidate == component)
            .unwrap_or(Self::COMPONENTS.len())
    }
}

fn push_tree(lines: &mut Vec<String>, root: &str, children: &[String]) {
    lines.push(root.to_string());
    for (index, child) in children.iter().enumerate() {
        let connector = if index + 1 == children.len() {
            "└── "
        } else {
            "├── "
        };
        lines.push(format!("{connector}{child}"));
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

/// Restores the terminal on drop — including on panic or early error —
/// so a wizard crash never leaves the user's shell in raw mode inside the
/// alternate screen. All cleanup is best-effort: a failing step must not
/// prevent the remaining ones (council C-004).
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = crossterm::terminal::disable_raw_mode();
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
    let _guard = TerminalGuard;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    run_loop(&mut terminal)
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
            match state.handle(map_key(key, state.is_editing())) {
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
        help_line(state),
        theme,
        env!("CARGO_PKG_VERSION"),
    );

    match state.step() {
        WizardStep::Profile => render_profile(frame, content, theme, state),
        WizardStep::ProjectShape => render_project_shape(frame, content, theme, state),
        WizardStep::AiTooling => render_ai_tooling(frame, content, theme, state),
        WizardStep::ToolConfig => render_tool_config(frame, content, theme, state),
        WizardStep::Templates => render_templates(frame, content, theme, state),
        WizardStep::Paths => render_paths(frame, content, theme, state),
        WizardStep::Components => render_components(frame, content, theme, state),
        WizardStep::Done => render_done(frame, content, state),
    }
}

fn help_line(state: &WizardState) -> &'static str {
    match state.step() {
        WizardStep::Paths => "type to edit  up/down field  enter next  esc back  ctrl+c quit",
        WizardStep::Templates if state.is_editing() => {
            "type path  enter confirm  esc cancel  ctrl+c quit"
        }
        WizardStep::Templates | WizardStep::Components => {
            "j/k navigate  space toggle  enter next  esc back  q quit"
        }
        _ => "j/k navigate  space toggle  a agents  m model  h/l hooks  enter next  q quit",
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
    let templates = if state.selected_templates().is_empty() {
        "none".to_string()
    } else {
        state
            .selected_templates()
            .iter()
            .map(|template| template.label())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let components = if state.selected_components().is_empty() {
        "none".to_string()
    } else {
        state
            .selected_components()
            .iter()
            .map(|component| component.label())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let summary = format!(
        "Profile: {}\nProject shape: {}\nAI tools: {}\nTemplates: {}\nPaths: plans {} | docs {} | tooling {}\nComponents: {}\n\nScaffold execution starts in TUI-004.",
        state.profile().label(),
        state.project_shape.label(),
        tools,
        templates,
        state.plans_dir(),
        state.docs_dir(),
        state.tooling_root(),
        components
    );

    Paragraph::new(summary)
        .block(Block::default().title("Summary").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_templates(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let chunks = Layout::vertical([Constraint::Min(7), Constraint::Length(3)]).split(area);

    let items = WizardState::TEMPLATES
        .iter()
        .map(|template| {
            let selected = state.selected_templates().contains(template);
            SelectItem::new(
                format!(
                    "{} {}",
                    if selected { "[x]" } else { "[ ]" },
                    template.label()
                ),
                template.description(),
            )
        })
        .collect();
    render_select(
        frame,
        chunks[0],
        theme,
        "Templates",
        items,
        state.template_index,
    );

    let title = if state.is_editing() {
        "Custom template path (editing)"
    } else {
        "Custom template path"
    };
    let input = TextInput::new(theme)
        .placeholder("path/to/template.md")
        .block(Block::default().title(title).borders(Borders::ALL));
    input.render(
        chunks[1],
        frame.buffer_mut(),
        &mut state.custom_template_path,
    );
}

fn render_paths(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let columns =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);
    let fields = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(columns[0]);

    const LABELS: [&str; 3] = ["Plans directory", "Docs location", "Tooling root"];
    for (index, label) in LABELS.iter().enumerate() {
        let title = if state.path_focus() == index {
            format!("▸ {label}")
        } else {
            format!("  {label}")
        };
        let input = TextInput::new(theme)
            .block(Block::default().title(title).borders(Borders::ALL));
        input.render(
            fields[index],
            frame.buffer_mut(),
            &mut state.path_inputs[index],
        );
    }

    if let Some(error) = state.path_error {
        Paragraph::new(format!("⚠ {error}"))
            .render(fields[3], frame.buffer_mut());
    }

    Paragraph::new(state.directory_preview())
        .block(Block::default().title("Preview").borders(Borders::ALL))
        .render(columns[1], frame.buffer_mut());
}

fn render_components(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let columns =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);

    let items = WizardState::COMPONENTS
        .iter()
        .map(|component| {
            let selected = state.selected_components().contains(component);
            SelectItem::new(
                format!(
                    "{} {}",
                    if selected { "[x]" } else { "[ ]" },
                    component.label()
                ),
                component.description(),
            )
        })
        .collect();
    render_select(
        frame,
        columns[0],
        theme,
        "Components",
        items,
        state.component_index,
    );

    Paragraph::new(state.directory_preview())
        .block(Block::default().title("Preview").borders(Borders::ALL))
        .render(columns[1], frame.buffer_mut());
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

impl Template {
    fn label(self) -> &'static str {
        match self {
            Self::Quickstart => "quickstart",
            Self::Module => "module",
            Self::Index => "index",
            Self::MonorepoIndex => "monorepo-index",
            Self::Custom => "custom path",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Quickstart => "minimal single-file plan to try APS fast",
            Self::Module => "bounded module with work items",
            Self::Index => "roadmap index for a new initiative",
            Self::MonorepoIndex => "index variant for multi-package repos",
            Self::Custom => "bring your own template file or directory",
        }
    }
}

impl Component {
    fn label(self) -> &'static str {
        match self {
            Self::LintRules => "lint rules",
            Self::ApsRules => "aps-rules.md",
            Self::ProjectContext => "project-context.md",
            Self::DesignsDir => "designs/ directory",
            Self::DecisionsDir => "decisions/ directory",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::LintRules => "document linting under the tooling root",
            Self::ApsRules => "portable agent guide (APS-managed)",
            Self::ProjectContext => "project intent and constraints (user-owned)",
            Self::DesignsDir => "technical design documents",
            Self::DecisionsDir => "decision records",
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

/// Map a key event to an Action, bypassing the vim-style aliases
/// (`j`/`k`/`h`/`l`/`q`/space) while a text field is being edited so those
/// characters are typable in paths.
pub fn map_key(event: KeyEvent, editing: bool) -> Action {
    if !editing {
        return KeyHandler::map(event);
    }

    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return match event.code {
            KeyCode::Char('c') => Action::Quit,
            KeyCode::Char('a') => Action::Home,
            KeyCode::Char('e') => Action::End,
            _ => Action::None,
        };
    }

    match event.code {
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::Left => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Enter => Action::Select,
        KeyCode::Esc => Action::Back,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Delete,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Char(c) => Action::Character(c),
        _ => Action::None,
    }
}

/// Characters allowed into path/template text fields. Control codes, bidi
/// overrides, and zero-width characters are rejected — they corrupt stored
/// paths and enable filename spoofing once TUI-004 writes to disk.
fn is_text_char(c: char) -> bool {
    !c.is_control()
        && !matches!(
            c,
            '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}'
        )
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

        // single + agent operator -> index + module (wildcard arm)
        let mut agent = WizardState::default();
        agent.handle(Action::Down);
        agent.handle(Action::Down);
        at_templates(&mut agent);
        assert_eq!(
            agent.selected_templates(),
            &[Template::Module, Template::Index]
        );
    }

    #[test]
    fn manual_template_selection_survives_back_and_forward() {
        let mut state = WizardState::default();
        at_templates(&mut state);

        state.handle(Action::Toggle); // deselect quickstart -> touched
        state.handle(Action::Select); // -> AiTooling
        state.handle(Action::Back); // -> Templates again

        assert_eq!(state.selected_templates(), &[]);
    }

    #[test]
    fn enter_with_empty_custom_path_deselects_custom() {
        let mut state = WizardState::default();
        at_templates(&mut state);

        state.handle(Action::Up); // wrap to custom row
        state.handle(Action::Toggle);
        assert!(state.is_editing());

        state.handle(Action::Select); // confirm without typing

        assert!(!state.is_editing());
        assert!(!state.selected_templates().contains(&Template::Custom));
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

    fn at_paths(state: &mut WizardState) {
        for _ in 0..4 {
            state.handle(Action::Select);
        }
        assert_eq!(state.step(), WizardStep::Paths);
    }

    #[test]
    fn paths_step_has_three_fields_with_defaults() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        assert_eq!(state.plans_dir(), "plans/");
        assert_eq!(state.docs_dir(), "docs/");
        assert_eq!(state.tooling_root(), ".aps/");
        assert!(state.is_editing());
    }

    #[test]
    fn arrows_move_path_focus_with_wrap() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        assert_eq!(state.path_focus(), 0);
        state.handle(Action::Down);
        assert_eq!(state.path_focus(), 1);
        state.handle(Action::Down);
        assert_eq!(state.path_focus(), 2);
        state.handle(Action::Down);
        assert_eq!(state.path_focus(), 0);
        state.handle(Action::Up);
        assert_eq!(state.path_focus(), 2);
    }

    #[test]
    fn typing_edits_focused_path_field_without_navigation_side_effects() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        state.handle(Action::Character('x'));
        assert_eq!(state.plans_dir(), "plans/x");

        state.handle(Action::Backspace);
        assert_eq!(state.plans_dir(), "plans/");

        // vim-aliased characters must insert, not navigate or quit
        assert_eq!(state.handle(Action::Character('j')), WizardEvent::Continue);
        assert_eq!(state.handle(Action::Character('q')), WizardEvent::Continue);
        assert_eq!(state.plans_dir(), "plans/jq");
        assert_eq!(state.path_focus(), 0);
        assert_eq!(state.step(), WizardStep::Paths);
    }

    #[test]
    fn cursor_keys_operate_on_the_focused_path_field() {
        let mut state = WizardState::default();
        at_paths(&mut state);
        state.handle(Action::Down); // focus docs dir

        state.handle(Action::Home);
        state.handle(Action::Character('m'));
        assert_eq!(state.docs_dir(), "mdocs/");

        state.handle(Action::Delete);
        assert_eq!(state.docs_dir(), "mocs/");

        state.handle(Action::End);
        state.handle(Action::Left);
        state.handle(Action::Backspace);
        assert_eq!(state.docs_dir(), "moc/");
    }

    #[test]
    fn empty_path_field_blocks_advance() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        assert_eq!(state.plans_dir(), "");

        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Paths);
        assert!(state.path_error.is_some());

        state.handle(Action::Character('p'));
        assert!(state.path_error.is_none());

        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Components);
    }

    #[test]
    fn control_and_bidi_characters_are_rejected() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        state.handle(Action::Character('\u{1b}')); // escape
        state.handle(Action::Character('\u{202E}')); // bidi override
        state.handle(Action::Character('\u{200B}')); // zero-width space

        assert_eq!(state.plans_dir(), "plans/");
    }

    #[test]
    fn map_key_passes_text_keys_through_when_editing() {
        let plain = |code| KeyEvent::new(code, KeyModifiers::empty());

        assert_eq!(map_key(plain(KeyCode::Char('j')), false), Action::Down);
        assert_eq!(
            map_key(plain(KeyCode::Char('j')), true),
            Action::Character('j')
        );
        assert_eq!(
            map_key(plain(KeyCode::Char('q')), true),
            Action::Character('q')
        );
        assert_eq!(
            map_key(plain(KeyCode::Char(' ')), true),
            Action::Character(' ')
        );
        assert_eq!(map_key(plain(KeyCode::Esc), true), Action::Back);
        assert_eq!(map_key(plain(KeyCode::Enter), true), Action::Select);
        assert_eq!(map_key(plain(KeyCode::Backspace), true), Action::Backspace);

        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(ctrl_c, true), Action::Quit);

        let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let ctrl_e = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert_eq!(map_key(ctrl_a, true), Action::Home);
        assert_eq!(map_key(ctrl_e, true), Action::End);
    }

    fn at_components(state: &mut WizardState) {
        for _ in 0..5 {
            state.handle(Action::Select);
        }
        assert_eq!(state.step(), WizardStep::Components);
    }

    #[test]
    fn components_default_to_all_enabled() {
        let mut state = WizardState::default();
        at_components(&mut state);

        assert_eq!(
            state.selected_components(),
            &[
                Component::LintRules,
                Component::ApsRules,
                Component::ProjectContext,
                Component::DesignsDir,
                Component::DecisionsDir,
            ]
        );
    }

    #[test]
    fn space_toggles_highlighted_component() {
        let mut state = WizardState::default();
        at_components(&mut state);

        state.handle(Action::Toggle); // lint rules highlighted first
        assert!(
            !state
                .selected_components()
                .contains(&Component::LintRules)
        );

        state.handle(Action::Toggle);
        assert!(state.selected_components().contains(&Component::LintRules));
    }

    #[test]
    fn directory_preview_reflects_paths_templates_and_components() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        state.handle(Action::Character('x')); // plans/ -> plans/x
        state.handle(Action::Select); // -> Components

        let preview = state.directory_preview();
        assert!(preview.contains("plans/x"));
        assert!(preview.contains("quickstart.aps.md")); // solo single default
        assert!(preview.contains("designs/"));
        assert!(preview.contains(".aps/"));
        assert!(preview.contains("lint/"));

        // deselect designs/ (index 3) and lint rules (index 0)
        for _ in 0..3 {
            state.handle(Action::Down);
        }
        state.handle(Action::Toggle);
        state.handle(Action::Up);
        state.handle(Action::Up);
        state.handle(Action::Up);
        state.handle(Action::Toggle);

        let preview = state.directory_preview();
        assert!(!preview.contains("designs/"));
        assert!(!preview.contains("lint/"));
    }

    #[test]
    fn directory_preview_updates_as_paths_change() {
        let mut state = WizardState::default();
        at_paths(&mut state);

        let before = state.directory_preview();
        state.handle(Action::Character('z'));
        let after = state.directory_preview();

        assert_ne!(before, after);
        assert!(after.contains("plans/z"));
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

use eddacraft_tui::prelude::Action;
use eddacraft_tui::prelude::{EddaCraftTheme, KeyHandler, Select, SelectItem, SelectState};
use eddacraft_tui::prelude::{ShellBranding, render_shell};
use eddacraft_tui::prelude::{TextInput, TextInputState};
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
pub enum PathField {
    PlansDir,
    DocsDir,
    ToolingRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Profile,
    ProjectShape,
    AiTooling,
    ToolConfig,
    Templates,
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
    template_index: usize,
    selected_templates: Vec<Template>,
    templates_initialized: bool,
    custom_template_enabled: bool,
    editing_custom_template: bool,
    custom_template: TextInputState,
    path_field_index: usize,
    plans_dir: TextInputState,
    docs_dir: TextInputState,
    tooling_root: TextInputState,
    component_index: usize,
    selected_components: Vec<Component>,
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
            template_index: 0,
            selected_templates: Vec::new(),
            templates_initialized: false,
            custom_template_enabled: false,
            editing_custom_template: false,
            custom_template: TextInputState::default(),
            path_field_index: 0,
            plans_dir: text_input_with(Self::DEFAULT_PLANS_DIR),
            docs_dir: text_input_with(Self::DEFAULT_DOCS_DIR),
            tooling_root: text_input_with(Self::DEFAULT_TOOLING_ROOT),
            component_index: 0,
            selected_components: Self::COMPONENTS.to_vec(),
        }
    }
}

fn text_input_with(value: &str) -> TextInputState {
    let mut state = TextInputState::default();
    for c in value.chars() {
        state.insert(c);
    }
    state
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
    const TEMPLATES: [Template; 4] = [
        Template::Quickstart,
        Template::Module,
        Template::Index,
        Template::MonorepoIndex,
    ];
    const COMPONENTS: [Component; 5] = [
        Component::LintRules,
        Component::ApsRules,
        Component::ProjectContext,
        Component::DesignsDir,
        Component::DecisionsDir,
    ];
    const PATH_FIELDS: [PathField; 3] = [
        PathField::PlansDir,
        PathField::DocsDir,
        PathField::ToolingRoot,
    ];
    const DEFAULT_PLANS_DIR: &'static str = "plans/";
    const DEFAULT_DOCS_DIR: &'static str = "docs/";
    const DEFAULT_TOOLING_ROOT: &'static str = ".aps/";

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

    /// True while the wizard is consuming raw character input (custom
    /// template path editing or the Paths step). The event loop maps keys
    /// in text mode instead of the vim-style navigation mode.
    pub fn text_entry_active(&self) -> bool {
        self.editing_custom_template || self.step == WizardStep::Paths
    }

    pub fn handle(&mut self, action: Action) -> WizardEvent {
        if self.text_entry_active() {
            return self.handle_text(action);
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

    fn handle_text(&mut self, action: Action) -> WizardEvent {
        match action {
            Action::Quit => return WizardEvent::Quit,
            Action::Up => self.move_path_field(false),
            Action::Down => self.move_path_field(true),
            Action::Left => self.current_text_mut().move_left(),
            Action::Right => self.current_text_mut().move_right(),
            Action::Home => self.current_text_mut().home(),
            Action::End => self.current_text_mut().end(),
            Action::Backspace => self.current_text_mut().backspace(),
            Action::Delete => self.current_text_mut().delete(),
            Action::Character(c) => self.current_text_mut().insert(c),
            Action::Back => {
                if self.editing_custom_template {
                    self.finish_custom_template_edit();
                } else {
                    self.back();
                }
            }
            Action::Select => {
                if self.editing_custom_template {
                    self.finish_custom_template_edit();
                } else if self.path_field_index + 1 < Self::PATH_FIELDS.len() {
                    self.path_field_index += 1;
                } else {
                    return self.advance();
                }
            }
            _ => {}
        }

        WizardEvent::Continue
    }

    fn current_text_mut(&mut self) -> &mut TextInputState {
        if self.editing_custom_template {
            return &mut self.custom_template;
        }

        match Self::PATH_FIELDS[self.path_field_index] {
            PathField::PlansDir => &mut self.plans_dir,
            PathField::DocsDir => &mut self.docs_dir,
            PathField::ToolingRoot => &mut self.tooling_root,
        }
    }

    fn move_path_field(&mut self, forward: bool) {
        if self.step == WizardStep::Paths && !self.editing_custom_template {
            self.path_field_index =
                move_index(self.path_field_index, Self::PATH_FIELDS.len(), forward);
        }
    }

    fn finish_custom_template_edit(&mut self) {
        self.editing_custom_template = false;
        if self.custom_template.value.trim().is_empty() {
            self.custom_template_enabled = false;
        }
    }

    fn advance(&mut self) -> WizardEvent {
        self.step = match self.step {
            WizardStep::Profile => WizardStep::ProjectShape,
            WizardStep::ProjectShape => WizardStep::AiTooling,
            WizardStep::AiTooling if self.selected_tools.is_empty() => WizardStep::Templates,
            WizardStep::AiTooling => WizardStep::ToolConfig,
            WizardStep::ToolConfig => WizardStep::Templates,
            WizardStep::Templates => WizardStep::Paths,
            WizardStep::Paths => {
                self.restore_default_paths();
                WizardStep::Components
            }
            WizardStep::Components => WizardStep::Done,
            WizardStep::Done => return WizardEvent::Complete,
        };

        if self.step == WizardStep::Templates && !self.templates_initialized {
            self.selected_templates = Self::default_templates(self.profile, self.project_shape);
            self.templates_initialized = true;
        }

        WizardEvent::Continue
    }

    fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Profile => WizardStep::Profile,
            WizardStep::ProjectShape => WizardStep::Profile,
            WizardStep::AiTooling => WizardStep::ProjectShape,
            WizardStep::ToolConfig => WizardStep::AiTooling,
            WizardStep::Templates if self.tool_configs.is_empty() => WizardStep::AiTooling,
            WizardStep::Templates => WizardStep::ToolConfig,
            WizardStep::Paths => WizardStep::Templates,
            WizardStep::Components => WizardStep::Paths,
            WizardStep::Done => WizardStep::Components,
        };
    }

    /// Empty path fields fall back to their defaults when leaving the step,
    /// so the scaffold never receives a blank target directory.
    fn restore_default_paths(&mut self) {
        for (input, default) in [
            (&mut self.plans_dir, Self::DEFAULT_PLANS_DIR),
            (&mut self.docs_dir, Self::DEFAULT_DOCS_DIR),
            (&mut self.tooling_root, Self::DEFAULT_TOOLING_ROOT),
        ] {
            if input.value.trim().is_empty() {
                *input = text_input_with(default);
            }
        }
    }

    fn default_templates(profile: Profile, shape: ProjectShape) -> Vec<Template> {
        let mut templates = vec![match shape {
            ProjectShape::SingleProject => Template::Index,
            ProjectShape::Monorepo => Template::MonorepoIndex,
        }];
        templates.push(match profile {
            Profile::Solo => Template::Quickstart,
            Profile::Team | Profile::AgentOperator => Template::Module,
        });
        templates.sort_by_key(|template| Self::template_order(*template));
        templates
    }

    fn template_order(template: Template) -> usize {
        Self::TEMPLATES
            .iter()
            .position(|candidate| *candidate == template)
            .unwrap_or(Self::TEMPLATES.len())
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
                // +1 for the trailing "custom template path" entry.
                self.template_index =
                    move_index(self.template_index, Self::TEMPLATES.len() + 1, forward);
            }
            WizardStep::Components => {
                self.component_index =
                    move_index(self.component_index, Self::COMPONENTS.len(), forward);
            }
            WizardStep::Paths | WizardStep::Done => {}
        }
    }

    fn toggle_current(&mut self) {
        match self.step {
            WizardStep::AiTooling => self.toggle_tool(Self::AI_TOOLS[self.ai_tool_index]),
            WizardStep::Templates => {
                if self.template_index < Self::TEMPLATES.len() {
                    self.toggle_template(Self::TEMPLATES[self.template_index]);
                } else {
                    self.custom_template_enabled = !self.custom_template_enabled;
                    self.editing_custom_template = self.custom_template_enabled;
                }
            }
            WizardStep::Components => {
                self.toggle_component(Self::COMPONENTS[self.component_index]);
            }
            _ => {}
        }
    }

    pub fn toggle_template(&mut self, template: Template) {
        if let Some(index) = self
            .selected_templates
            .iter()
            .position(|selected| *selected == template)
        {
            self.selected_templates.remove(index);
        } else {
            self.selected_templates.push(template);
            self.selected_templates
                .sort_by_key(|template| Self::template_order(*template));
        }
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
            self.selected_components.sort_by_key(|component| {
                Self::COMPONENTS
                    .iter()
                    .position(|candidate| candidate == component)
                    .unwrap_or(Self::COMPONENTS.len())
            });
        }
    }

    pub fn selected_templates(&self) -> &[Template] {
        &self.selected_templates
    }

    pub fn selected_components(&self) -> &[Component] {
        &self.selected_components
    }

    pub fn custom_template_path(&self) -> Option<&str> {
        let path = self.custom_template.value.trim();
        (self.custom_template_enabled && !path.is_empty()).then_some(path)
    }

    pub fn plans_dir(&self) -> &str {
        &self.plans_dir.value
    }

    pub fn docs_dir(&self) -> &str {
        &self.docs_dir.value
    }

    pub fn tooling_root(&self) -> &str {
        &self.tooling_root.value
    }

    /// Render the directory tree that the current path selections would
    /// produce. Pure so the preview is unit-testable without a terminal.
    pub fn directory_preview(&self) -> String {
        let plans = display_dir(&self.plans_dir.value, Self::DEFAULT_PLANS_DIR);
        let docs = display_dir(&self.docs_dir.value, Self::DEFAULT_DOCS_DIR);
        let tooling = display_dir(&self.tooling_root.value, Self::DEFAULT_TOOLING_ROOT);

        format!(
            "<project>/\n\
             ├── {plans}\n\
             │   ├── index.aps.md\n\
             │   ├── aps-rules.md\n\
             │   ├── designs/\n\
             │   └── decisions/\n\
             ├── {docs}\n\
             └── {tooling}"
        )
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
            // Text-entry steps need raw characters; the vim-style handler
            // would swallow h/j/k/l/q as navigation.
            let action = if state.text_entry_active() {
                map_text_key(key)
            } else {
                KeyHandler::map(key)
            };
            match state.handle(action) {
                WizardEvent::Continue => {}
                WizardEvent::Complete | WizardEvent::Quit => return Ok(()),
            }
        }
    }
}

fn map_text_key(event: crossterm::event::KeyEvent) -> Action {
    use crossterm::event::{KeyCode, KeyModifiers};

    if event.modifiers.contains(KeyModifiers::CONTROL) {
        return match event.code {
            KeyCode::Char('c') => Action::Quit,
            _ => Action::None,
        };
    }

    match event.code {
        KeyCode::Up => Action::Up,
        KeyCode::Down | KeyCode::Tab => Action::Down,
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

fn render(frame: &mut Frame<'_>, theme: &EddaCraftTheme, state: &mut WizardState) {
    let hints = if state.text_entry_active() {
        "type to edit  up/down field  enter next  esc back  ctrl+c quit"
    } else {
        "j/k navigate  space toggle  a agents  m model  h/l hooks  enter next  q quit"
    };
    let content = render_shell(
        frame,
        frame.area(),
        ShellBranding::Anvil,
        "APS",
        "Init",
        hints,
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

fn render_templates(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let chunks = Layout::vertical([Constraint::Min(8), Constraint::Length(3)]).split(area);

    let mut items: Vec<SelectItem> = WizardState::TEMPLATES
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
    let custom_label = match state.custom_template_path() {
        Some(path) => format!("[x] Custom template path: {path}"),
        None if state.custom_template_enabled => "[x] Custom template path: (typing…)".to_string(),
        None => "[ ] Custom template path".to_string(),
    };
    items.push(SelectItem::new(
        custom_label,
        "Bring your own template directory",
    ));

    render_select(
        frame,
        chunks[0],
        theme,
        "Templates",
        items,
        state.template_index,
    );

    if state.editing_custom_template {
        let input = TextInput::new(theme)
            .placeholder("path/to/template.md")
            .block(
                Block::default()
                    .title("Custom template path (enter to confirm, esc to cancel)")
                    .borders(Borders::ALL),
            );
        input.render(chunks[1], frame.buffer_mut(), &mut state.custom_template);
    } else {
        Paragraph::new("Defaults follow your profile and project shape; space toggles.")
            .block(Block::default().borders(Borders::ALL))
            .render(chunks[1], frame.buffer_mut());
    }
}

fn render_paths(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(9),
    ])
    .split(area);

    let preview = state.directory_preview();
    let focused = state.path_field_index;
    let fields: [(&str, &mut TextInputState); 3] = [
        ("Plans directory", &mut state.plans_dir),
        ("Docs location", &mut state.docs_dir),
        ("Tooling root", &mut state.tooling_root),
    ];

    for (index, (title, input_state)) in fields.into_iter().enumerate() {
        let marker = if index == focused { "▸ " } else { "  " };
        let input = TextInput::new(theme).block(
            Block::default()
                .title(format!("{marker}{title}"))
                .borders(Borders::ALL),
        );
        input.render(chunks[index], frame.buffer_mut(), input_state);
    }

    Paragraph::new(preview)
        .block(
            Block::default()
                .title("Resulting structure")
                .borders(Borders::ALL),
        )
        .render(chunks[3], frame.buffer_mut());
}

fn render_components(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
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
        area,
        theme,
        "Components",
        items,
        state.component_index,
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
    let templates = state
        .selected_templates()
        .iter()
        .map(|template| template.label())
        .collect::<Vec<_>>()
        .join(", ");
    let templates = match state.custom_template_path() {
        Some(path) => {
            if templates.is_empty() {
                format!("custom ({path})")
            } else {
                format!("{templates}, custom ({path})")
            }
        }
        None if templates.is_empty() => "none".to_string(),
        None => templates,
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
        "Profile: {}\nProject shape: {}\nAI tools: {}\nTemplates: {}\nPaths: plans={} docs={} tooling={}\nComponents: {}\n\nScaffold execution starts in TUI-004.",
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

impl Template {
    fn label(self) -> &'static str {
        match self {
            Self::Quickstart => "quickstart",
            Self::Module => "module",
            Self::Index => "index",
            Self::MonorepoIndex => "monorepo-index",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Quickstart => "single-file plan for small features",
            Self::Module => "full module plan with work items",
            Self::Index => "top-level plans/index.aps.md",
            Self::MonorepoIndex => "index with per-package plan links",
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
            Self::LintRules => "document validation via aps lint",
            Self::ApsRules => "authoring conventions for plans",
            Self::ProjectContext => "durable project background for agents",
            Self::DesignsDir => "design documents alongside plans",
            Self::DecisionsDir => "decision log directory",
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

fn display_dir(value: &str, default: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        default.to_string()
    } else {
        format!("{trimmed}/")
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

    /// Drive a default state to the given step via repeated Select.
    fn advance_to(state: &mut WizardState, step: WizardStep) {
        for _ in 0..16 {
            if state.step() == step {
                return;
            }
            assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        }
        panic!("never reached {step:?}, stuck at {:?}", state.step());
    }

    #[test]
    fn done_screen_renders_before_completion() {
        let mut state = WizardState::default();

        advance_to(&mut state, WizardStep::Done);

        assert_eq!(state.handle(Action::Select), WizardEvent::Complete);
    }

    #[test]
    fn full_flow_visits_template_path_and_component_steps() {
        let mut state = WizardState::default();

        let mut visited = vec![state.step()];
        while state.step() != WizardStep::Done {
            state.handle(Action::Select);
            if visited.last() != Some(&state.step()) {
                visited.push(state.step());
            }
        }

        assert_eq!(
            visited,
            vec![
                WizardStep::Profile,
                WizardStep::ProjectShape,
                WizardStep::AiTooling,
                WizardStep::Templates,
                WizardStep::Paths,
                WizardStep::Components,
                WizardStep::Done,
            ]
        );
    }

    #[test]
    fn templates_default_from_profile_and_shape() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        // Solo + single project → quickstart + index.
        assert_eq!(
            state.selected_templates(),
            &[Template::Quickstart, Template::Index]
        );

        let mut monorepo = WizardState::default();
        monorepo.handle(Action::Down); // profile: team
        monorepo.handle(Action::Select);
        monorepo.handle(Action::Down); // shape: monorepo
        advance_to(&mut monorepo, WizardStep::Templates);

        assert_eq!(
            monorepo.selected_templates(),
            &[Template::Module, Template::MonorepoIndex]
        );
    }

    #[test]
    fn space_toggles_template_under_cursor() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        assert!(state.selected_templates().contains(&Template::Quickstart));
        state.handle(Action::Toggle); // cursor starts on quickstart
        assert!(!state.selected_templates().contains(&Template::Quickstart));
    }

    #[test]
    fn custom_template_entry_takes_typed_path() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        // Move to the trailing custom entry and enable it.
        for _ in 0..WizardState::TEMPLATES.len() {
            state.handle(Action::Down);
        }
        state.handle(Action::Toggle);
        assert!(state.text_entry_active());

        for c in "tpl.md".chars() {
            state.handle(Action::Character(c));
        }
        state.handle(Action::Select);

        assert!(!state.text_entry_active());
        assert_eq!(state.custom_template_path(), Some("tpl.md"));
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn empty_custom_template_path_disables_custom_entry() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        for _ in 0..WizardState::TEMPLATES.len() {
            state.handle(Action::Down);
        }
        state.handle(Action::Toggle);
        state.handle(Action::Back); // esc with empty value

        assert_eq!(state.custom_template_path(), None);
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn paths_have_defaults_and_accept_edits() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        assert_eq!(state.plans_dir(), "plans/");
        assert_eq!(state.docs_dir(), "docs/");
        assert_eq!(state.tooling_root(), ".aps/");

        // Clear plans dir and type a custom path; q must insert, not quit.
        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        for c in "quarters/".chars() {
            assert_eq!(state.handle(Action::Character(c)), WizardEvent::Continue);
        }

        assert_eq!(state.plans_dir(), "quarters/");
        assert!(state.directory_preview().contains("quarters/"));
    }

    #[test]
    fn empty_path_restores_default_on_advance() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        assert_eq!(state.plans_dir(), "");

        advance_to(&mut state, WizardStep::Components);

        assert_eq!(state.plans_dir(), "plans/");
    }

    #[test]
    fn enter_walks_path_fields_then_advances() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Paths); // docs field
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Paths); // tooling field
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Components);
    }

    #[test]
    fn components_default_on_and_toggle_off() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Components);

        assert_eq!(state.selected_components(), &WizardState::COMPONENTS);

        state.handle(Action::Toggle); // cursor on lint rules
        assert!(!state.selected_components().contains(&Component::LintRules));
    }

    #[test]
    fn escape_steps_back_through_new_sections() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Done);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Components);
        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Paths);
        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Templates);
        state.handle(Action::Back);
        // No tools selected → skips ToolConfig on the way back too.
        assert_eq!(state.step(), WizardStep::AiTooling);
    }
}

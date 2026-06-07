use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
pub enum PlanTemplate {
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

/// Raw user input, trimmed but NOT validated or canonicalized. The scaffold
/// step (TUI-004) owns path policy: it must reject or confine absolute paths
/// and `..` components and decide `~` expansion before writing to disk. The
/// wizard's preview flags those shapes but does not block them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathsConfig {
    pub plans_dir: String,
    pub docs_dir: String,
    pub tooling_root: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            plans_dir: "plans/".to_string(),
            docs_dir: "docs/".to_string(),
            tooling_root: ".aps/".to_string(),
        }
    }
}

/// What the active text editor is bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditTarget {
    CustomTemplate,
    PathField(usize),
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
    selected_templates: Vec<PlanTemplate>,
    template_index: usize,
    templates_touched: bool,
    custom_template_path: String,
    paths: PathsConfig,
    path_field_index: usize,
    selected_components: Vec<Component>,
    component_index: usize,
    edit: Option<EditTarget>,
    edit_state: TextInputState,
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
            custom_template_path: String::new(),
            paths: PathsConfig::default(),
            path_field_index: 0,
            selected_components: Component::DEFAULTS.to_vec(),
            component_index: 0,
            edit: None,
            edit_state: TextInputState::default(),
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
    const TEMPLATES: [PlanTemplate; 4] = [
        PlanTemplate::Quickstart,
        PlanTemplate::Module,
        PlanTemplate::Index,
        PlanTemplate::MonorepoIndex,
    ];
    /// Rows on the Templates screen: the four templates plus the custom-path row.
    const TEMPLATE_ROWS: usize = Self::TEMPLATES.len() + 1;
    const PATH_FIELDS: usize = 3;

    pub fn step(&self) -> WizardStep {
        self.step
    }

    pub fn selected_templates(&self) -> &[PlanTemplate] {
        &self.selected_templates
    }

    pub fn custom_template_path(&self) -> &str {
        &self.custom_template_path
    }

    pub fn paths(&self) -> &PathsConfig {
        &self.paths
    }

    pub fn selected_components(&self) -> &[Component] {
        &self.selected_components
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Defaults informed by profile and project shape (user can override).
    fn template_defaults(profile: Profile, shape: ProjectShape) -> Vec<PlanTemplate> {
        match (profile, shape) {
            (_, ProjectShape::Monorepo) => {
                vec![PlanTemplate::Module, PlanTemplate::MonorepoIndex]
            }
            (Profile::Solo, ProjectShape::SingleProject) => vec![
                PlanTemplate::Quickstart,
                PlanTemplate::Module,
                PlanTemplate::Index,
            ],
            (_, ProjectShape::SingleProject) => {
                vec![PlanTemplate::Module, PlanTemplate::Index]
            }
        }
    }

    fn apply_template_defaults(&mut self) {
        if !self.templates_touched {
            self.selected_templates = Self::template_defaults(self.profile, self.project_shape);
        }
    }

    pub fn toggle_template(&mut self, template: PlanTemplate) {
        self.templates_touched = true;
        if let Some(index) = self
            .selected_templates
            .iter()
            .position(|selected| *selected == template)
        {
            self.selected_templates.remove(index);
        } else {
            // Index and MonorepoIndex scaffold the same file — mutually exclusive
            match template {
                PlanTemplate::Index => self
                    .selected_templates
                    .retain(|selected| *selected != PlanTemplate::MonorepoIndex),
                PlanTemplate::MonorepoIndex => self
                    .selected_templates
                    .retain(|selected| *selected != PlanTemplate::Index),
                _ => {}
            }
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
            self.selected_components
                .sort_by_key(|component| Self::component_order(*component));
        }
    }

    fn template_order(template: PlanTemplate) -> usize {
        match template {
            PlanTemplate::Custom => Self::TEMPLATES.len(),
            other => Self::TEMPLATES
                .iter()
                .position(|candidate| *candidate == other)
                .unwrap_or(Self::TEMPLATES.len()),
        }
    }

    fn component_order(component: Component) -> usize {
        Component::ALL
            .iter()
            .position(|candidate| *candidate == component)
            .unwrap_or(Component::ALL.len())
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn set_project_shape(&mut self, project_shape: ProjectShape) {
        // Template overrides stick while the context is unchanged; an actual
        // shape change invalidates that context, so defaults re-derive
        if self.project_shape != project_shape {
            self.templates_touched = false;
        }
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
        if self.edit.is_some() {
            return self.handle_edit(action);
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
            WizardStep::ProjectShape => WizardStep::AiTooling,
            WizardStep::AiTooling if self.selected_tools.is_empty() => WizardStep::Templates,
            WizardStep::AiTooling => WizardStep::ToolConfig,
            WizardStep::ToolConfig => WizardStep::Templates,
            WizardStep::Templates => {
                // A custom template needs a path to mean anything
                if self.custom_template_path.trim().is_empty() {
                    self.selected_templates
                        .retain(|template| *template != PlanTemplate::Custom);
                }
                WizardStep::Paths
            }
            WizardStep::Paths => WizardStep::Components,
            WizardStep::Components => WizardStep::Done,
            WizardStep::Done => return WizardEvent::Complete,
        };

        if self.step == WizardStep::Templates {
            self.apply_template_defaults();
        }

        WizardEvent::Continue
    }

    fn back(&mut self) {
        // Navigating away always discards any open edit, even when back() is
        // called directly rather than routed through handle_edit()
        self.edit = None;
        self.step = match self.step {
            WizardStep::Profile => WizardStep::Profile,
            WizardStep::ProjectShape => WizardStep::Profile,
            WizardStep::AiTooling => WizardStep::ProjectShape,
            // Routing derives from selected_tools because the Templates step
            // never mutates it; if a future action arm changes tools while on
            // Templates, switch to recording the arrival step instead
            WizardStep::Templates if self.selected_tools.is_empty() => WizardStep::AiTooling,
            WizardStep::Templates => WizardStep::ToolConfig,
            WizardStep::ToolConfig => WizardStep::AiTooling,
            WizardStep::Paths => WizardStep::Templates,
            WizardStep::Components => WizardStep::Paths,
            WizardStep::Done => WizardStep::Components,
        };

        // Defaults refresh on backward entry too (still gated on touched)
        if self.step == WizardStep::Templates {
            self.apply_template_defaults();
        }
    }

    fn move_selection(&mut self, forward: bool) {
        match self.step {
            WizardStep::Profile => {
                self.profile_index = move_index(self.profile_index, Self::PROFILES.len(), forward);
                // A profile change invalidates template overrides (see
                // set_project_shape for the same rule)
                if self.profile != Self::PROFILES[self.profile_index] {
                    self.templates_touched = false;
                }
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
                self.template_index = move_index(self.template_index, Self::TEMPLATE_ROWS, forward);
            }
            WizardStep::Paths => {
                self.path_field_index =
                    move_index(self.path_field_index, Self::PATH_FIELDS, forward);
            }
            WizardStep::Components => {
                self.component_index =
                    move_index(self.component_index, Component::ALL.len(), forward);
            }
            WizardStep::Done => {}
        }
    }

    fn toggle_current(&mut self) {
        match self.step {
            WizardStep::AiTooling => self.toggle_tool(Self::AI_TOOLS[self.ai_tool_index]),
            WizardStep::Templates => {
                if self.template_index < Self::TEMPLATES.len() {
                    self.toggle_template(Self::TEMPLATES[self.template_index]);
                } else {
                    self.start_edit(EditTarget::CustomTemplate);
                }
            }
            WizardStep::Paths => self.start_edit(EditTarget::PathField(self.path_field_index)),
            WizardStep::Components => {
                self.toggle_component(Component::ALL[self.component_index]);
            }
            _ => {}
        }
    }

    /// Longest accepted input — PATH_MAX on Linux; nothing legitimate is close.
    const MAX_INPUT_LEN: usize = 4096;

    fn start_edit(&mut self, target: EditTarget) {
        debug_assert!(self.edit.is_none(), "nested edit sessions are a bug");
        let value = match target {
            EditTarget::CustomTemplate => self.custom_template_path.clone(),
            EditTarget::PathField(index) => self.path_field(index).to_string(),
        };
        self.edit_state = text_state(value);
        self.edit = Some(target);
    }

    fn handle_edit(&mut self, action: Action) -> WizardEvent {
        match action {
            Action::Quit => return WizardEvent::Quit,
            Action::Character(c) if self.edit_state.value.len() < Self::MAX_INPUT_LEN => {
                self.edit_state.insert(c);
            }
            Action::Backspace => self.edit_state.backspace(),
            Action::Delete => self.edit_state.delete(),
            Action::Left => self.edit_state.move_left(),
            Action::Right => self.edit_state.move_right(),
            Action::Home => self.edit_state.home(),
            Action::End => self.edit_state.end(),
            Action::Select => self.commit_edit(),
            Action::Back => self.edit = None, // cancel, discard buffer
            _ => {}
        }
        WizardEvent::Continue
    }

    fn commit_edit(&mut self) {
        let Some(target) = self.edit.take() else {
            return;
        };
        let value = self.edit_state.value.trim().to_string();
        let has_path = !value.is_empty();
        match target {
            EditTarget::CustomTemplate => {
                // Custom path may be empty (means: no custom template) —
                // unlike path fields there is no default to fall back to
                self.templates_touched = true;
                self.custom_template_path = value;
                let has_custom = self.selected_templates.contains(&PlanTemplate::Custom);
                if has_path && !has_custom {
                    self.selected_templates.push(PlanTemplate::Custom);
                } else if !has_path && has_custom {
                    self.selected_templates
                        .retain(|template| *template != PlanTemplate::Custom);
                }
            }
            EditTarget::PathField(index) => {
                // Paths fall back to defaults rather than going empty
                let committed = if has_path {
                    value
                } else {
                    Self::path_default(index).to_string()
                };
                *self.path_field_mut(index) = committed;
            }
        }
    }

    fn path_field(&self, index: usize) -> &str {
        debug_assert!(index < Self::PATH_FIELDS);
        match index {
            0 => &self.paths.plans_dir,
            1 => &self.paths.docs_dir,
            _ => &self.paths.tooling_root,
        }
    }

    fn path_field_mut(&mut self, index: usize) -> &mut String {
        debug_assert!(index < Self::PATH_FIELDS);
        match index {
            0 => &mut self.paths.plans_dir,
            1 => &mut self.paths.docs_dir,
            _ => &mut self.paths.tooling_root,
        }
    }

    fn path_default(index: usize) -> &'static str {
        debug_assert!(index < Self::PATH_FIELDS);
        match index {
            0 => "plans/",
            1 => "docs/",
            _ => ".aps/",
        }
    }

    /// Effective paths, with the live edit buffer overriding its field so the
    /// directory preview updates as the user types.
    fn effective_paths(&self) -> PathsConfig {
        let mut paths = self.paths.clone();
        if let Some(EditTarget::PathField(index)) = self.edit {
            let value = self.edit_state.value.trim();
            if !value.is_empty() {
                match index {
                    0 => paths.plans_dir = value.to_string(),
                    1 => paths.docs_dir = value.to_string(),
                    _ => paths.tooling_root = value.to_string(),
                }
            }
        }
        paths
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
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(&mut terminal);

    // Best-effort cleanup: a failure restoring one mode must not skip the rest
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableBracketedPaste,
        crossterm::terminal::LeaveAlternateScreen
    );
    let _ = terminal.show_cursor();

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let theme = EddaCraftTheme;
    let mut state = WizardState::default();

    loop {
        terminal.draw(|frame| render(frame, &theme, &mut state))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            let key = match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => key,
                // Bracketed paste: insert into the active edit, ignore otherwise
                // (without this, a pasted trailing newline would commit the
                // field and spill the rest into navigation, where 'q' quits)
                crossterm::event::Event::Paste(text) => {
                    if state.is_editing() {
                        for c in text.chars().filter(|c| !c.is_control()) {
                            state.handle(Action::Character(c));
                        }
                    }
                    continue;
                }
                _ => continue,
            };
            // Windows terminals also report key releases — only act on presses
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            // KeyHandler maps j/k/h/l/q/space to navigation, which would
            // swallow typed characters — use a literal mapping while editing
            let action = if state.is_editing() {
                map_edit_key(key)
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

/// Literal key mapping for text editing: characters insert, Enter commits,
/// Esc cancels. Only Ctrl-C still quits.
fn map_edit_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Action::Quit,
            _ => Action::None,
        };
    }
    match key.code {
        KeyCode::Enter => Action::Select,
        KeyCode::Esc => Action::Back,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Delete,
        KeyCode::Left => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Char(c) => Action::Character(c),
        _ => Action::None,
    }
}

fn render(frame: &mut Frame<'_>, theme: &EddaCraftTheme, state: &mut WizardState) {
    let hint = if state.is_editing() {
        "type to edit  enter save  esc cancel"
    } else {
        match state.step() {
            WizardStep::ToolConfig => {
                "j/k navigate  a agents  m model  h/l hooks  enter next  esc back  q quit"
            }
            WizardStep::Paths => "j/k field  space edit  enter next  esc back  q quit",
            _ => "j/k navigate  space toggle  enter next  esc back  q quit",
        }
    };
    let content = render_shell(
        frame,
        frame.area(),
        ShellBranding::Anvil,
        "APS",
        "Init",
        hint,
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
    let custom_selected = state.selected_templates().contains(&PlanTemplate::Custom);
    items.push(SelectItem::new(
        format!(
            "{} custom path…",
            if custom_selected { "[x]" } else { "[ ]" }
        ),
        PlanTemplate::Custom.description(),
    ));

    render_select(
        frame,
        chunks[0],
        theme,
        "Templates",
        items,
        state.template_index,
    );

    let editing_custom = state.edit == Some(EditTarget::CustomTemplate);
    let mut input_state = if editing_custom {
        state.edit_state.clone()
    } else {
        text_state(state.custom_template_path().to_string())
    };
    let input = TextInput::new(theme)
        .placeholder("path/to/template.md (space on the custom row to edit)")
        .block(
            Block::default()
                .title("Custom Template Path")
                .borders(Borders::ALL),
        );
    input.render(chunks[1], frame.buffer_mut(), &mut input_state);
}

fn render_paths(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &EddaCraftTheme,
    state: &mut WizardState,
) {
    let columns =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);
    let left = Layout::vertical([Constraint::Min(6), Constraint::Length(3)]).split(columns[0]);

    let labels = ["Plans directory", "Docs location", "Tooling root"];
    let items = labels
        .iter()
        .enumerate()
        .map(|(index, label)| SelectItem::new(format!("{label}: {}", state.path_field(index)), ""))
        .collect();
    render_select(
        frame,
        left[0],
        theme,
        "Paths",
        items,
        state.path_field_index,
    );

    let editing_path = matches!(state.edit, Some(EditTarget::PathField(_)));
    // When not editing, the input previews the field under the cursor (the
    // one a Toggle would start editing), not the last-committed field
    let mut input_state = if editing_path {
        state.edit_state.clone()
    } else {
        text_state(state.path_field(state.path_field_index).to_string())
    };
    let input = TextInput::new(theme)
        .placeholder("space to edit the selected path")
        .block(Block::default().title("Edit Path").borders(Borders::ALL));
    input.render(left[1], frame.buffer_mut(), &mut input_state);

    // Live preview: reflects the edit buffer as the user types
    let preview = preview_tree(
        &state.effective_paths(),
        state.selected_templates(),
        state.selected_components(),
    );
    Paragraph::new(preview)
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

    let items = Component::ALL
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

    let preview = preview_tree(
        state.paths(),
        state.selected_templates(),
        state.selected_components(),
    );
    Paragraph::new(preview)
        .block(Block::default().title("Preview").borders(Borders::ALL))
        .render(columns[1], frame.buffer_mut());
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
            .map(|template| {
                if *template == PlanTemplate::Custom {
                    format!("custom ({})", state.custom_template_path())
                } else {
                    template.label().to_string()
                }
            })
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
        "Profile: {}\nProject shape: {}\nAI tools: {}\nTemplates: {}\nPaths: plans={} docs={} tooling={}\nComponents: {}\n\nScaffold execution starts in TUI-004.",
        state.profile().label(),
        state.project_shape.label(),
        tools,
        templates,
        state.paths().plans_dir,
        state.paths().docs_dir,
        state.paths().tooling_root,
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

impl PlanTemplate {
    fn label(self) -> &'static str {
        match self {
            Self::Quickstart => "quickstart",
            Self::Module => "module",
            Self::Index => "index",
            Self::MonorepoIndex => "monorepo-index",
            Self::Custom => "custom",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Quickstart => "Try APS in 5 minutes — minimal single-file plan",
            Self::Module => "Bounded module with work items",
            Self::Index => "Root plan for a single project",
            Self::MonorepoIndex => "Root plan with package views for monorepos",
            Self::Custom => "Your own template file",
        }
    }
}

impl Component {
    pub const ALL: [Component; 5] = [
        Component::LintRules,
        Component::ApsRules,
        Component::ProjectContext,
        Component::DesignsDir,
        Component::DecisionsDir,
    ];
    /// Everything on by default except decisions/, which APS treats as optional.
    pub const DEFAULTS: [Component; 4] = [
        Component::LintRules,
        Component::ApsRules,
        Component::ProjectContext,
        Component::DesignsDir,
    ];

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
            Self::LintRules => "Validation rules for aps lint",
            Self::ApsRules => "Agent guidance that travels with the plan",
            Self::ProjectContext => "User-owned project context for agents",
            Self::DesignsDir => "Home for design documents",
            Self::DecisionsDir => "Home for ADRs (optional)",
        }
    }
}

/// Render the directory structure that the current selections would scaffold.
/// Pure so the live preview is testable without a terminal.
fn preview_tree(
    paths: &PathsConfig,
    templates: &[PlanTemplate],
    components: &[Component],
) -> String {
    // Paths that escape (or might escape) the project root are shown verbatim
    // with a warning instead of being normalized into a misleading nested
    // entry — this preview is the user's confirmation surface
    let dir = |raw: &str| {
        let cleaned = raw.trim();
        if cleaned.starts_with('/')
            || cleaned.starts_with('~')
            || cleaned.split('/').any(|part| part == "..")
        {
            return format!("{cleaned}  (!) outside project root");
        }
        let trimmed = cleaned.trim_matches('/');
        if trimmed.is_empty() {
            "./".to_string()
        } else {
            format!("{trimmed}/")
        }
    };
    let plans = dir(&paths.plans_dir);
    let docs = dir(&paths.docs_dir);
    let tooling = dir(&paths.tooling_root);

    let mut plan_entries: Vec<String> = Vec::new();
    if templates.contains(&PlanTemplate::Index) {
        plan_entries.push("index.aps.md".to_string());
    }
    if templates.contains(&PlanTemplate::MonorepoIndex) {
        plan_entries.push("index.aps.md  (monorepo)".to_string());
    }
    if templates.contains(&PlanTemplate::Quickstart) {
        plan_entries.push("quickstart.aps.md".to_string());
    }
    if components.contains(&Component::ApsRules) {
        plan_entries.push("aps-rules.md".to_string());
    }
    if components.contains(&Component::ProjectContext) {
        plan_entries.push("project-context.md".to_string());
    }
    if templates.contains(&PlanTemplate::Module) {
        plan_entries.push("modules/".to_string());
    }
    if components.contains(&Component::DesignsDir) {
        plan_entries.push("designs/".to_string());
    }
    if components.contains(&Component::DecisionsDir) {
        plan_entries.push("decisions/".to_string());
    }

    let mut tree = String::from("project/\n");
    tree.push_str(&format!("├── {plans}\n"));
    for (i, entry) in plan_entries.iter().enumerate() {
        let branch = if i + 1 == plan_entries.len() {
            "└──"
        } else {
            "├──"
        };
        tree.push_str(&format!("│   {branch} {entry}\n"));
    }
    tree.push_str(&format!("├── {docs}\n"));
    tree.push_str(&format!("└── {tooling}\n"));
    let mut tooling_entries: Vec<&str> = vec!["bin/", "lib/"];
    if components.contains(&Component::LintRules) {
        tooling_entries.push("lib/rules/");
    }
    for (i, entry) in tooling_entries.iter().enumerate() {
        let branch = if i + 1 == tooling_entries.len() {
            "└──"
        } else {
            "├──"
        };
        tree.push_str(&format!("    {branch} {entry}\n"));
    }
    tree
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

/// TextInputState with a value and the cursor at the end (its `cursor` field
/// is private, so struct-update syntax can't be used).
fn text_state(value: String) -> TextInputState {
    let mut state = TextInputState::default();
    state.value = value;
    state.end();
    state
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
        assert_eq!(state.step(), WizardStep::Templates);
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Paths);
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Components);
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Done);

        assert_eq!(state.handle(Action::Select), WizardEvent::Complete);
    }

    /// Drive the wizard from Profile to the Templates step.
    fn advance_to_templates(state: &mut WizardState) {
        state.handle(Action::Select); // Profile -> ProjectShape
        state.handle(Action::Select); // ProjectShape -> AiTooling
        state.handle(Action::Select); // AiTooling (empty) -> Templates
        assert_eq!(state.step(), WizardStep::Templates);
    }

    #[test]
    fn template_defaults_follow_profile_and_shape() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        // Solo + single project
        assert_eq!(
            state.selected_templates(),
            &[
                PlanTemplate::Quickstart,
                PlanTemplate::Module,
                PlanTemplate::Index,
            ]
        );

        // Monorepo swaps index for monorepo-index (untouched defaults refresh)
        let mut state = WizardState::default();
        state.set_project_shape(ProjectShape::Monorepo);
        advance_to_templates(&mut state);
        assert_eq!(
            state.selected_templates(),
            &[PlanTemplate::Module, PlanTemplate::MonorepoIndex]
        );
    }

    #[test]
    fn touched_templates_survive_default_refresh() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        state.toggle_template(PlanTemplate::Quickstart); // deselect
        state.handle(Action::Back); // back to AiTooling
        state.handle(Action::Select); // forward again

        assert_eq!(
            state.selected_templates(),
            &[PlanTemplate::Module, PlanTemplate::Index]
        );
    }

    #[test]
    fn space_toggles_template_under_cursor() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        state.handle(Action::Toggle); // quickstart row, was selected by default
        assert!(
            !state
                .selected_templates()
                .contains(&PlanTemplate::Quickstart)
        );

        state.handle(Action::Toggle);
        assert!(
            state
                .selected_templates()
                .contains(&PlanTemplate::Quickstart)
        );
    }

    #[test]
    fn custom_template_path_edits_commit_and_cancel() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        // Move to the custom row (5th) and start editing
        for _ in 0..4 {
            state.handle(Action::Down);
        }
        state.handle(Action::Toggle);
        assert!(state.is_editing());

        for c in "tpl/x.md".chars() {
            state.handle(Action::Character(c));
        }
        state.handle(Action::Select); // commit

        assert!(!state.is_editing());
        assert_eq!(state.custom_template_path(), "tpl/x.md");
        assert!(state.selected_templates().contains(&PlanTemplate::Custom));

        // Cancel an edit: buffer discarded
        state.handle(Action::Toggle);
        state.handle(Action::Character('z'));
        state.handle(Action::Back);
        assert_eq!(state.custom_template_path(), "tpl/x.md");

        // Clearing the path deselects custom
        state.handle(Action::Toggle);
        for _ in 0.."tpl/x.md".len() {
            state.handle(Action::Backspace);
        }
        state.handle(Action::Select);
        assert!(!state.selected_templates().contains(&PlanTemplate::Custom));
    }

    #[test]
    fn editing_mode_captures_navigation_characters() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);
        state.handle(Action::Select); // -> Paths
        assert_eq!(state.step(), WizardStep::Paths);

        state.handle(Action::Toggle); // edit plans dir
        assert!(state.is_editing());
        // 'q' and 'j' must insert, not quit/navigate (map_edit_key sends
        // Character for them; verify the state accepts them as text)
        state.handle(Action::Character('q'));
        state.handle(Action::Character('j'));
        state.handle(Action::Select);

        assert_eq!(state.paths().plans_dir, "plans/qj");
    }

    #[test]
    fn path_edits_update_values_and_empty_restores_default() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);
        state.handle(Action::Select); // -> Paths

        assert_eq!(state.paths().plans_dir, "plans/");

        // Edit plans dir to a custom location
        state.handle(Action::Toggle);
        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        for c in "specs/".chars() {
            state.handle(Action::Character(c));
        }
        state.handle(Action::Select);
        assert_eq!(state.paths().plans_dir, "specs/");

        // Emptying a field falls back to its default
        state.handle(Action::Down); // docs field
        state.handle(Action::Toggle);
        for _ in 0.."docs/".len() {
            state.handle(Action::Backspace);
        }
        state.handle(Action::Select);
        assert_eq!(state.paths().docs_dir, "docs/");
    }

    #[test]
    fn components_default_and_toggle() {
        let mut state = WizardState::default();
        assert_eq!(state.selected_components(), &Component::DEFAULTS);

        advance_to_templates(&mut state);
        state.handle(Action::Select); // -> Paths
        state.handle(Action::Select); // -> Components
        assert_eq!(state.step(), WizardStep::Components);

        // Toggle decisions/ on (last row)
        for _ in 0..4 {
            state.handle(Action::Down);
        }
        state.handle(Action::Toggle);
        assert!(
            state
                .selected_components()
                .contains(&Component::DecisionsDir)
        );

        // Toggle lint rules off (first row)
        state.handle(Action::Down); // wraps to first
        state.handle(Action::Toggle);
        assert!(!state.selected_components().contains(&Component::LintRules));
    }

    #[test]
    fn preview_reflects_paths_templates_and_components() {
        let paths = PathsConfig {
            plans_dir: "specs/".to_string(),
            docs_dir: "documentation/".to_string(),
            tooling_root: ".tooling/".to_string(),
        };
        let preview = preview_tree(
            &paths,
            &[PlanTemplate::Module, PlanTemplate::Index],
            &[Component::ApsRules, Component::DecisionsDir],
        );

        assert!(preview.contains("specs/"));
        assert!(preview.contains("documentation/"));
        assert!(preview.contains(".tooling/"));
        assert!(preview.contains("index.aps.md"));
        assert!(preview.contains("modules/"));
        assert!(preview.contains("aps-rules.md"));
        assert!(preview.contains("decisions/"));
        assert!(!preview.contains("quickstart.aps.md"));
        assert!(!preview.contains("designs/"));
        assert!(!preview.contains("lib/rules/"));
    }

    #[test]
    fn custom_without_path_is_dropped_on_advance() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        state.toggle_template(PlanTemplate::Custom); // selected, no path
        state.handle(Action::Select); // -> Paths

        assert!(!state.selected_templates().contains(&PlanTemplate::Custom));
    }

    #[test]
    fn shape_change_after_touch_refreshes_defaults() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        // Touch the selection, then change shape — context change re-derives
        state.toggle_template(PlanTemplate::Quickstart);
        state.handle(Action::Back); // -> AiTooling
        state.handle(Action::Back); // -> ProjectShape
        state.handle(Action::Down); // single -> monorepo
        state.handle(Action::Select); // -> AiTooling
        state.handle(Action::Select); // -> Templates

        assert_eq!(
            state.selected_templates(),
            &[PlanTemplate::Module, PlanTemplate::MonorepoIndex]
        );
    }

    #[test]
    fn back_from_templates_with_no_tools_returns_to_ai_tooling() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::AiTooling);
    }

    #[test]
    fn defaults_refresh_on_backward_entry_too() {
        let mut state = WizardState::default();
        state.toggle_tool(AiTool::ClaudeCode);
        state.handle(Action::Select); // -> ProjectShape
        state.handle(Action::Select); // -> AiTooling
        state.handle(Action::Select); // -> ToolConfig
        state.handle(Action::Select); // -> Templates
        state.handle(Action::Select); // -> Paths

        // Back must land on Templates with (refreshed) defaults intact
        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Templates);
        assert!(!state.selected_templates().is_empty());
    }

    #[test]
    fn index_and_monorepo_index_are_mutually_exclusive() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);

        // Solo+single default includes Index
        assert!(state.selected_templates().contains(&PlanTemplate::Index));
        state.toggle_template(PlanTemplate::MonorepoIndex);
        assert!(
            state
                .selected_templates()
                .contains(&PlanTemplate::MonorepoIndex)
        );
        assert!(!state.selected_templates().contains(&PlanTemplate::Index));

        state.toggle_template(PlanTemplate::Index);
        assert!(state.selected_templates().contains(&PlanTemplate::Index));
        assert!(
            !state
                .selected_templates()
                .contains(&PlanTemplate::MonorepoIndex)
        );
    }

    #[test]
    fn effective_paths_reflects_live_edit_buffer() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);
        state.handle(Action::Select); // -> Paths

        state.handle(Action::Toggle); // edit plans dir
        for c in "x".chars() {
            state.handle(Action::Character(c));
        }
        // Uncommitted: stored value unchanged, effective value live
        assert_eq!(state.paths().plans_dir, "plans/");
        assert_eq!(state.effective_paths().plans_dir, "plans/x");
    }

    #[test]
    fn tool_configs_survive_back_and_readvance() {
        let mut state = WizardState::default();
        state.toggle_tool(AiTool::ClaudeCode);
        state.handle(Action::Select); // -> ProjectShape
        state.handle(Action::Select); // -> AiTooling
        state.handle(Action::Select); // -> ToolConfig
        state.handle(Action::Character('a')); // customise: agents off

        state.handle(Action::Back); // -> AiTooling
        state.handle(Action::Select); // -> ToolConfig again

        assert!(!state.tool_configs()[0].install_agents);
    }

    #[test]
    fn preview_flags_paths_outside_project_root() {
        let paths = PathsConfig {
            plans_dir: "/etc/cron.d".to_string(),
            docs_dir: "../escape".to_string(),
            tooling_root: "~/aps".to_string(),
        };
        let preview = preview_tree(&paths, &[], &[]);

        assert!(preview.contains("/etc/cron.d  (!) outside project root"));
        assert!(preview.contains("../escape  (!) outside project root"));
        assert!(preview.contains("~/aps  (!) outside project root"));
        // Names merely containing dots are not flagged
        let safe = PathsConfig {
            plans_dir: "my..plans/".to_string(),
            ..PathsConfig::default()
        };
        assert!(!preview_tree(&safe, &[], &[]).contains("(!)"));
    }

    #[test]
    fn input_length_is_capped() {
        let mut state = WizardState::default();
        advance_to_templates(&mut state);
        state.handle(Action::Select); // -> Paths
        state.handle(Action::Toggle); // edit

        for _ in 0..(WizardState::MAX_INPUT_LEN + 100) {
            state.handle(Action::Character('a'));
        }
        state.handle(Action::Select);
        assert!(state.paths().plans_dir.len() <= WizardState::MAX_INPUT_LEN);
    }

    #[test]
    fn back_walks_through_new_steps() {
        let mut state = WizardState::default();
        state.toggle_tool(AiTool::ClaudeCode);
        state.handle(Action::Select); // -> ProjectShape
        state.handle(Action::Select); // -> AiTooling
        state.handle(Action::Select); // -> ToolConfig (tool selected)
        state.handle(Action::Select); // -> Templates
        state.handle(Action::Select); // -> Paths
        state.handle(Action::Select); // -> Components
        assert_eq!(state.step(), WizardStep::Components);

        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Paths);
        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::Templates);
        state.handle(Action::Back);
        assert_eq!(state.step(), WizardStep::ToolConfig);
    }
}

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
use std::path::PathBuf;
use std::time::Duration;

use crate::scaffold::{self, ScaffoldRun, Selections, StepStatus};

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
    Grok,
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
    /// Federated nested-plans root (MONO-005): a federation `index.aps.md` plus
    /// starter child plans under `packages/<pkg>/plans/`.
    IndexNested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    LintRules,
    ApsRules,
    ProjectContext,
    DesignsDir,
    DecisionsDir,
    ReleasesDir,
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
    Review,
    Scaffold,
    Summary,
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
    path_error: Option<&'static str>,
    plans_dir: TextInputState,
    docs_dir: TextInputState,
    tooling_root: TextInputState,
    component_index: usize,
    selected_components: Vec<Component>,
    components_initialized: bool,
    root: PathBuf,
    scaffold_run: Option<ScaffoldRun>,
    scaffold_error: Option<String>,
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
            path_error: None,
            plans_dir: text_input_with(Self::DEFAULT_PLANS_DIR),
            docs_dir: text_input_with(Self::DEFAULT_DOCS_DIR),
            tooling_root: text_input_with(Self::DEFAULT_TOOLING_ROOT),
            component_index: 0,
            selected_components: Self::COMPONENTS.to_vec(),
            components_initialized: false,
            root: PathBuf::from("."),
            scaffold_run: None,
            scaffold_error: None,
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
        AiTool::Grok,
        AiTool::Generic,
    ];
    const TEMPLATES: [Template; 5] = [
        Template::Quickstart,
        Template::Module,
        Template::Index,
        Template::MonorepoIndex,
        Template::IndexNested,
    ];
    const COMPONENTS: [Component; 6] = [
        Component::LintRules,
        Component::ApsRules,
        Component::ProjectContext,
        Component::DesignsDir,
        Component::DecisionsDir,
        Component::ReleasesDir,
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

        // Scaffold execution is not interruptible except by quitting.
        if self.step == WizardStep::Scaffold {
            return match action {
                Action::Quit => WizardEvent::Quit,
                _ => WizardEvent::Continue,
            };
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
            Action::Backspace => {
                self.path_error = None;
                self.current_text_mut().backspace();
            }
            Action::Delete => {
                self.path_error = None;
                self.current_text_mut().delete();
            }
            Action::Character(c) if is_text_char(c) => {
                self.path_error = None;
                self.current_text_mut().insert(c);
            }
            // Control, bidi-override, and zero-width characters are dropped:
            // they corrupt stored paths and enable filename spoofing once the
            // scaffold writes to disk.
            Action::Character(_) => {}
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
                } else if let Some(reason) = self.first_invalid_path() {
                    self.path_error = Some(reason);
                } else {
                    return self.advance();
                }
            }
            _ => {}
        }

        WizardEvent::Continue
    }

    /// Insert pasted text into the focused text field. Only the first line
    /// is taken and unsafe characters are stripped — the same choke point as
    /// typed input. No-op outside text entry, so a paste can never navigate
    /// or trigger scaffold execution.
    pub fn paste(&mut self, text: &str) {
        if !self.text_entry_active() {
            return;
        }

        self.path_error = None;
        for c in sanitize_paste(text).chars() {
            self.current_text_mut().insert(c);
        }
    }

    /// Check the three target directories before leaving the Paths step.
    /// Empty values are fine (the scaffold substitutes defaults); absolute
    /// and parent-traversing paths would let the scaffold write outside the
    /// project root.
    fn first_invalid_path(&self) -> Option<&'static str> {
        [&self.plans_dir, &self.docs_dir, &self.tooling_root]
            .into_iter()
            .find_map(|input| invalid_path_reason(&input.value))
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
            WizardStep::Components => WizardStep::Review,
            WizardStep::Review => {
                self.start_scaffold();
                WizardStep::Scaffold
            }
            WizardStep::Scaffold => WizardStep::Scaffold, // advanced by scaffold_tick
            WizardStep::Summary => return WizardEvent::Complete,
        };

        if self.step == WizardStep::Templates && !self.templates_initialized {
            self.selected_templates = Self::default_templates(self.profile, self.project_shape);
            self.templates_initialized = true;
        }

        if self.step == WizardStep::Components && !self.components_initialized {
            self.selected_components = Self::default_components(self.profile);
            self.components_initialized = true;
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
            WizardStep::Review => WizardStep::Components,
            // Scaffolding has side effects; there is no going back.
            WizardStep::Scaffold => WizardStep::Scaffold,
            WizardStep::Summary => WizardStep::Summary,
        };
    }

    /// Collect the wizard's selections for scaffold planning. Also the
    /// hand-off point for the non-interactive path (TUI-005).
    pub fn selections(&self) -> Selections {
        Selections {
            profile: self.profile,
            shape: self.project_shape,
            tools: self.tool_configs.clone(),
            templates: self.selected_templates.clone(),
            custom_template: self.custom_template_path().map(str::to_string),
            plans_dir: self.plans_dir.value.clone(),
            docs_dir: self.docs_dir.value.clone(),
            tooling_root: self.tooling_root.value.clone(),
            components: self.selected_components.clone(),
            // Serialization stamps the running binary's version (D-035).
            cli_version: None,
        }
    }

    fn start_scaffold(&mut self) {
        let selections = self.selections();
        match scaffold::check_target(&self.root, &selections) {
            Ok(()) => {
                self.scaffold_run = Some(ScaffoldRun::new(self.root.clone(), &selections));
            }
            Err(message) => {
                self.scaffold_error = Some(message);
            }
        }
    }

    /// Drive scaffold execution one step at a time so the event loop can
    /// redraw between steps. No-op outside the Scaffold step.
    pub fn scaffold_tick(&mut self) {
        if self.step != WizardStep::Scaffold {
            return;
        }

        match &mut self.scaffold_run {
            Some(run) => {
                if !run.run_next() {
                    self.step = WizardStep::Summary;
                }
            }
            // Target check failed; the summary screen shows the error.
            None => self.step = WizardStep::Summary,
        }
    }

    pub fn scaffold_run(&self) -> Option<&ScaffoldRun> {
        self.scaffold_run.as_ref()
    }

    pub fn scaffold_error(&self) -> Option<&str> {
        self.scaffold_error.as_deref()
    }

    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        Self {
            root,
            ..Self::default()
        }
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

    /// Components selected by default for a profile. Solo keeps the wizard
    /// minimal — release planning is opt-in — while Team and AI-agent
    /// operators get `releases/` on by default (REL-002). Every component
    /// stays toggleable; this only sets the initial check state.
    fn default_components(profile: Profile) -> Vec<Component> {
        Self::COMPONENTS
            .iter()
            .copied()
            .filter(|component| *component != Component::ReleasesDir || profile != Profile::Solo)
            .collect()
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
            WizardStep::Paths | WizardStep::Review | WizardStep::Scaffold | WizardStep::Summary => {
            }
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
            // Index, MonorepoIndex, and IndexNested all scaffold the root
            // index.aps.md — selecting one deselects the others so the scaffold
            // never writes one over another (council-b2bd78ac C-001; MONO-005).
            if matches!(
                template,
                Template::Index | Template::MonorepoIndex | Template::IndexNested
            ) {
                self.selected_templates.retain(|selected| {
                    !matches!(
                        selected,
                        Template::Index | Template::MonorepoIndex | Template::IndexNested
                    )
                });
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
             │   ├── decisions/\n\
             │   └── releases/\n\
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
    pub fn default_for(tool: AiTool) -> Self {
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

/// Characters allowed into path/template text fields. Control codes, bidi
/// overrides, and zero-width characters are rejected — they corrupt stored
/// paths and enable filename spoofing when the scaffold writes to disk.
fn is_text_char(c: char) -> bool {
    !c.is_control()
        && !matches!(
            c,
            '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}'
        )
}

/// Why a target directory value is unusable, or None if it is fine.
/// Mirrors the defense-in-depth check in scaffold::normalize_dir.
fn invalid_path_reason(value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None; // empty means "use the default"
    }
    let path = std::path::Path::new(trimmed);
    if path.is_absolute() {
        return Some("Paths must be relative to the project root");
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Some("Paths may not contain ..");
    }
    None
}

/// Upper bound on characters accepted from a single paste — keeps a
/// pathological single-line clipboard from freezing the UI or flooding the
/// field.
const MAX_PASTE_CHARS: usize = 4096;

/// Reduce pasted text to something safe for a single-line path field: the
/// first line only, with control/bidi/zero-width characters stripped,
/// capped at MAX_PASTE_CHARS.
fn sanitize_paste(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| is_text_char(*c))
        .take(MAX_PASTE_CHARS)
        .collect()
}

/// Restores the terminal on drop — including on panic or early error — so a
/// wizard crash never leaves the user's shell in raw mode inside the
/// alternate screen. Cleanup is best-effort: a failing step must not prevent
/// the remaining ones.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::event::DisableBracketedPaste,
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
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste
    )?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    run_loop(&mut terminal)
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let theme = EddaCraftTheme;
    let mut state = WizardState::default();

    loop {
        terminal.draw(|frame| render(frame, &theme, &mut state))?;

        // Execute one scaffold step per frame so progress renders smoothly.
        if state.step() == WizardStep::Scaffold {
            state.scaffold_tick();
            if crossterm::event::poll(Duration::from_millis(10))? {
                // Paste (and any non-key event) is intentionally discarded
                // here — the scaffold step is non-interactive.
                let crossterm::event::Event::Key(key) = crossterm::event::read()? else {
                    continue;
                };
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                if state.handle(KeyHandler::map(key)) == WizardEvent::Quit {
                    return Ok(());
                }
            }
            continue;
        }

        if crossterm::event::poll(Duration::from_millis(250))? {
            let key = match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => key,
                // Bracketed paste delivers the clipboard as one event
                // instead of replayed keystrokes — without this, pasted
                // newlines arrive as Enter and walk the wizard forward.
                crossterm::event::Event::Paste(text) => {
                    state.paste(&text);
                    continue;
                }
                _ => continue,
            };
            // Windows terminals report key releases too; acting on both would
            // double every keystroke and navigation step.
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
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
            KeyCode::Char('a') => Action::Home,
            KeyCode::Char('e') => Action::End,
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
        WizardStep::Review => render_review(frame, content, state),
        WizardStep::Scaffold => render_scaffold(frame, content, state),
        WizardStep::Summary => render_summary(frame, content, state),
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

    let preview = match state.path_error {
        Some(error) => format!("⚠ {error}\n\n{}", state.directory_preview()),
        None => state.directory_preview(),
    };
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

fn render_review(frame: &mut Frame<'_>, area: Rect, state: &WizardState) {
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
        "Profile: {}\nProject shape: {}\nAI tools: {}\nTemplates: {}\nPaths: plans={} docs={} tooling={}\nComponents: {}\n\nPress enter to scaffold, esc to go back.",
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
        .block(Block::default().title("Review").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_scaffold(frame: &mut Frame<'_>, area: Rect, state: &WizardState) {
    let mut lines = Vec::new();
    if let Some(run) = state.scaffold_run() {
        let (done, total) = run.progress();
        lines.push(format!("Scaffolding… {done}/{total}"));
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
    } else if let Some(error) = state.scaffold_error() {
        lines.push(format!("Cannot scaffold: {error}"));
    }

    Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Scaffold").borders(Borders::ALL))
        .render(area, frame.buffer_mut());
}

fn render_summary(frame: &mut Frame<'_>, area: Rect, state: &WizardState) {
    let mut lines = Vec::new();

    if let Some(error) = state.scaffold_error() {
        lines.push(format!("Scaffold aborted: {error}"));
    } else if let Some(run) = state.scaffold_run() {
        let failures = run.failures();
        if failures.is_empty() {
            lines.push("Scaffold complete.".to_string());
        } else {
            lines.push(format!(
                "Scaffold finished with {} error(s):",
                failures.len()
            ));
            for (label, message) in &failures {
                lines.push(format!("  ✗ {label}: {message}"));
            }
        }
        lines.push(String::new());
        lines.push("Installed:".to_string());
        for (label, status) in run.steps() {
            if matches!(status, StepStatus::Done) {
                lines.push(format!("  ✓ {label}"));
            }
        }
    }

    let notes: Vec<&str> = state
        .tool_configs()
        .iter()
        .filter_map(|config| scaffold::post_install_note(config.tool))
        .collect();
    if !notes.is_empty() {
        lines.push(String::new());
        lines.push("Next steps:".to_string());
        for note in notes {
            lines.push(format!("  - {note}"));
        }
    }
    lines.push(String::new());
    lines.push(format!(
        "  - Edit {}index.aps.md to define your plan",
        state.plans_dir()
    ));
    lines.push("  - Docs: https://github.com/EddaCraft/anvil-plan-spec".to_string());
    lines.push(String::new());
    lines.push("Press enter to finish.".to_string());

    Paragraph::new(lines.join("\n"))
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
            Self::Grok => "Grok",
            Self::Generic => "Generic",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::ClaudeCode => "skills, agents, and optional hooks",
            Self::Copilot => "repo instructions and custom agents",
            Self::Codex => "skills plus agent role config",
            Self::OpenCode => "skills and subagents",
            Self::Grok => "AGENTS.md + auto-discovered skills",
            Self::Generic => "plain APS files only",
        }
    }

    fn supports_agents(self) -> bool {
        // Grok Build ships no bespoke agent files — it discovers the
        // Codex-shared .agents/skills/ and the AGENTS.md family (D-040).
        !matches!(self, Self::Generic | Self::Grok)
    }

    fn default_model(self) -> ModelPreference {
        match self {
            // Anthropic-native tools: prefer Opus for planner/conductor work.
            Self::ClaudeCode | Self::OpenCode => ModelPreference::Opus,
            // Codex runs on OpenAI. Opus/Sonnet are not Codex models — leave
            // "default" so the parent session's model wins.
            Self::Codex => ModelPreference::Default,
            // Copilot/Grok/Generic do not expose an APS-driven model choice.
            Self::Copilot | Self::Grok | Self::Generic => ModelPreference::Default,
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
            Self::IndexNested => "index-nested",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Quickstart => "single-file plan for small features",
            Self::Module => "full module plan with work items",
            Self::Index => "top-level plans/index.aps.md",
            Self::MonorepoIndex => "index with per-package plan links",
            Self::IndexNested => "federated root + starter child plans",
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
            Self::ReleasesDir => "releases/ directory",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::LintRules => "document validation via aps lint",
            Self::ApsRules => "authoring conventions for plans",
            Self::ProjectContext => "durable project background for agents",
            Self::DesignsDir => "design documents alongside plans",
            Self::DecisionsDir => "decision log directory",
            Self::ReleasesDir => "release plans with README + template",
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
    fn tool_model_defaults_match_vendor() {
        // Claude Code / OpenCode ship Anthropic model IDs in agent frontmatter.
        assert_eq!(AiTool::ClaudeCode.default_model(), ModelPreference::Opus);
        assert_eq!(AiTool::OpenCode.default_model(), ModelPreference::Opus);
        // Codex is OpenAI — not sonnet/opus.
        assert_eq!(AiTool::Codex.default_model(), ModelPreference::Default);
        assert_eq!(AiTool::Copilot.default_model(), ModelPreference::Default);
        assert_eq!(AiTool::Grok.default_model(), ModelPreference::Default);
        assert_eq!(AiTool::Generic.default_model(), ModelPreference::Default);
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

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("aps-wizard-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn review_screen_renders_before_scaffold() {
        let root = temp_root("review");
        let mut state = WizardState::with_root(root.clone());

        advance_to(&mut state, WizardStep::Review);

        // Enter starts the scaffold; ticking drives it to the summary.
        assert_eq!(state.handle(Action::Select), WizardEvent::Continue);
        assert_eq!(state.step(), WizardStep::Scaffold);
        while state.step() == WizardStep::Scaffold {
            state.scaffold_tick();
        }
        assert_eq!(state.step(), WizardStep::Summary);
        assert_eq!(state.handle(Action::Select), WizardEvent::Complete);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn full_flow_visits_template_path_and_component_steps() {
        let mut state = WizardState::default();

        let mut visited = vec![state.step()];
        while state.step() != WizardStep::Review {
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
                WizardStep::Review,
            ]
        );
    }

    #[test]
    fn scaffold_writes_selected_structure_to_root() {
        let root = temp_root("scaffold");
        let mut state = WizardState::with_root(root.clone());

        advance_to(&mut state, WizardStep::Review);
        state.handle(Action::Select);
        while state.step() == WizardStep::Scaffold {
            state.scaffold_tick();
        }

        let run = state.scaffold_run().expect("scaffold ran");
        assert!(run.failures().is_empty(), "failures: {:?}", run.failures());
        assert!(root.join("plans/index.aps.md").exists());
        assert!(root.join(".aps/config.yml").exists());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn existing_plans_dir_surfaces_error_on_summary() {
        let root = temp_root("collision");
        std::fs::create_dir_all(root.join("plans")).unwrap();
        let mut state = WizardState::with_root(root.clone());

        advance_to(&mut state, WizardStep::Review);
        state.handle(Action::Select);
        while state.step() == WizardStep::Scaffold {
            state.scaffold_tick();
        }

        assert_eq!(state.step(), WizardStep::Summary);
        assert!(state.scaffold_error().is_some());
        assert!(state.scaffold_run().is_none());

        std::fs::remove_dir_all(&root).unwrap();
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
    fn index_and_monorepo_index_are_mutually_exclusive() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        // Solo+single starts with Index selected.
        assert!(state.selected_templates().contains(&Template::Index));

        state.toggle_template(Template::MonorepoIndex);
        assert!(
            state
                .selected_templates()
                .contains(&Template::MonorepoIndex)
        );
        assert!(!state.selected_templates().contains(&Template::Index));

        // And back the other way.
        state.toggle_template(Template::Index);
        assert!(state.selected_templates().contains(&Template::Index));
        assert!(
            !state
                .selected_templates()
                .contains(&Template::MonorepoIndex)
        );

        // At most one index template is ever selected.
        let index_count = state
            .selected_templates()
            .iter()
            .filter(|t| matches!(t, Template::Index | Template::MonorepoIndex))
            .count();
        assert_eq!(index_count, 1);
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

        // Solo keeps release planning opt-in; everything else is on.
        assert_eq!(
            state.selected_components(),
            &WizardState::default_components(Profile::Solo)
        );
        assert!(
            !state
                .selected_components()
                .contains(&Component::ReleasesDir)
        );

        state.handle(Action::Toggle); // cursor on lint rules
        assert!(!state.selected_components().contains(&Component::LintRules));
    }

    #[test]
    fn releases_dir_default_is_profile_gated() {
        // Solo: releases/ off by default, but still toggleable on.
        let mut solo = WizardState::default();
        advance_to(&mut solo, WizardStep::Components);
        assert!(!solo.selected_components().contains(&Component::ReleasesDir));

        // Team: releases/ on by default.
        let mut team = WizardState::default();
        team.handle(Action::Down); // profile: team
        advance_to(&mut team, WizardStep::Components);
        assert!(team.selected_components().contains(&Component::ReleasesDir));

        // AI agent operator: on by default too.
        assert!(
            WizardState::default_components(Profile::AgentOperator)
                .contains(&Component::ReleasesDir)
        );
    }

    #[test]
    fn escape_steps_back_through_new_sections() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Review);

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

    #[test]
    fn control_and_bidi_characters_are_rejected_in_text_entry() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        state.handle(Action::Character('\u{1b}')); // escape
        state.handle(Action::Character('\u{202E}')); // bidi override
        state.handle(Action::Character('\u{200B}')); // zero-width space

        assert_eq!(state.plans_dir.value, "plans/");
    }

    #[test]
    fn map_text_key_supports_ctrl_a_and_ctrl_e() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let ctrl = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);

        assert_eq!(map_text_key(ctrl('a')), Action::Home);
        assert_eq!(map_text_key(ctrl('e')), Action::End);
        assert_eq!(map_text_key(ctrl('c')), Action::Quit);
    }

    #[test]
    fn unsafe_paths_block_advance_with_error() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        // Overwrite the plans dir with a traversal path.
        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        for c in "../evil".chars() {
            state.handle(Action::Character(c));
        }

        // Enter walks the remaining fields, then attempts to advance.
        state.handle(Action::Select);
        state.handle(Action::Select);
        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::Paths);
        assert!(state.path_error.is_some());

        // Fix the offending field: focus back to plans dir, retype.
        state.handle(Action::Up);
        state.handle(Action::Up);
        for _ in 0.."../evil".len() {
            state.handle(Action::Backspace);
        }
        assert!(state.path_error.is_none());
        for c in "plans/".chars() {
            state.handle(Action::Character(c));
        }

        state.handle(Action::Select);
        state.handle(Action::Select);
        state.handle(Action::Select);
        assert_eq!(state.step(), WizardStep::Components);
    }

    #[test]
    fn absolute_paths_block_advance_with_error() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        for _ in 0.."plans/".len() {
            state.handle(Action::Backspace);
        }
        for c in "/etc/aps".chars() {
            state.handle(Action::Character(c));
        }

        state.handle(Action::Select);
        state.handle(Action::Select);
        state.handle(Action::Select);

        assert_eq!(state.step(), WizardStep::Paths);
        assert!(state.path_error.is_some());
    }

    #[test]
    fn sanitize_paste_takes_first_line_and_strips_unsafe_chars() {
        assert_eq!(sanitize_paste("docs/plans\nrm -rf /\nboom"), "docs/plans");
        assert_eq!(sanitize_paste("line1\r\nline2"), "line1");
        assert_eq!(sanitize_paste("a\u{202E}b\u{1b}c\u{200B}d"), "abcd");
        assert_eq!(sanitize_paste("plain/path"), "plain/path");
        assert_eq!(sanitize_paste(""), "");
    }

    #[test]
    fn sanitize_paste_caps_pathological_length() {
        let huge = "x".repeat(MAX_PASTE_CHARS * 3);
        assert_eq!(sanitize_paste(&huge).chars().count(), MAX_PASTE_CHARS);
    }

    #[test]
    fn paste_inserts_into_focused_field_without_advancing() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Paths);

        state.paste("custom\nrm -rf /\nmore");

        assert_eq!(state.plans_dir.value, "plans/custom");
        assert_eq!(state.step(), WizardStep::Paths);
    }

    #[test]
    fn paste_targets_the_custom_template_field_while_editing() {
        let mut state = WizardState::default();
        advance_to(&mut state, WizardStep::Templates);

        // select the custom template row to open its input
        state.handle(Action::Up);
        state.handle(Action::Toggle);
        assert!(state.text_entry_active());

        state.paste("my/template.md");

        assert_eq!(state.custom_template.value, "my/template.md");
    }

    #[test]
    fn paste_is_ignored_outside_text_entry() {
        let mut state = WizardState::default();
        assert_eq!(state.step(), WizardStep::Profile);

        state.paste("evil");

        assert_eq!(state.plans_dir.value, "plans/");
        assert_eq!(state.custom_template.value, "");
    }
}

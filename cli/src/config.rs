//! Non-interactive init: flag handling and config-file replay (TUI-005).
//!
//! `aps init --non-interactive` builds [`Selections`] from CLI flags;
//! `aps init --from <config.yml>` replays a configuration previously
//! written by the scaffold (see [`crate::scaffold::config_yaml`]). The
//! parser covers exactly that emitted shape — a deliberate subset of
//! YAML — so no serde dependency is needed.

use std::path::Path;

use crate::scaffold::{self, ScaffoldRun, Selections, StepStatus};
use crate::wizard::{
    AiTool, Component, HookVerbosity, ModelPreference, Profile, ProjectShape, Template, ToolConfig,
};

// --- Key parsing (inverse of scaffold::*_key) --------------------------------

pub fn profile_from_key(key: &str) -> Result<Profile, String> {
    match key {
        "solo" => Ok(Profile::Solo),
        "team" => Ok(Profile::Team),
        "agent-operator" => Ok(Profile::AgentOperator),
        other => Err(format!(
            "unknown profile '{other}' (expected solo, team, or agent-operator)"
        )),
    }
}

pub fn shape_from_key(key: &str) -> Result<ProjectShape, String> {
    match key {
        "single" => Ok(ProjectShape::SingleProject),
        "monorepo" => Ok(ProjectShape::Monorepo),
        other => Err(format!(
            "unknown shape '{other}' (expected single or monorepo)"
        )),
    }
}

pub fn template_from_key(key: &str) -> Result<Template, String> {
    match key {
        "quickstart" => Ok(Template::Quickstart),
        "module" => Ok(Template::Module),
        "index" => Ok(Template::Index),
        "monorepo-index" => Ok(Template::MonorepoIndex),
        other => Err(format!(
            "unknown template '{other}' (expected quickstart, module, index, or monorepo-index)"
        )),
    }
}

pub fn component_from_key(key: &str) -> Result<Component, String> {
    match key {
        "lint-rules" => Ok(Component::LintRules),
        "aps-rules" => Ok(Component::ApsRules),
        "project-context" => Ok(Component::ProjectContext),
        "designs-dir" => Ok(Component::DesignsDir),
        "decisions-dir" => Ok(Component::DecisionsDir),
        other => Err(format!("unknown component '{other}'")),
    }
}

pub fn tool_from_key(key: &str) -> Result<AiTool, String> {
    match key {
        "claude-code" => Ok(AiTool::ClaudeCode),
        "copilot" => Ok(AiTool::Copilot),
        "codex" => Ok(AiTool::Codex),
        "opencode" => Ok(AiTool::OpenCode),
        "gemini" => Ok(AiTool::Gemini),
        "generic" => Ok(AiTool::Generic),
        other => Err(format!("unknown tool '{other}'")),
    }
}

pub fn hooks_from_key(key: &str) -> Result<HookVerbosity, String> {
    match key {
        "full" => Ok(HookVerbosity::Full),
        "minimal" => Ok(HookVerbosity::Minimal),
        "none" => Ok(HookVerbosity::None),
        other => Err(format!(
            "unknown hook verbosity '{other}' (expected full, minimal, or none)"
        )),
    }
}

pub fn model_from_key(key: &str) -> Result<ModelPreference, String> {
    match key {
        "default" => Ok(ModelPreference::Default),
        "opus" => Ok(ModelPreference::Opus),
        "sonnet" => Ok(ModelPreference::Sonnet),
        other => Err(format!(
            "unknown model '{other}' (expected default, opus, or sonnet)"
        )),
    }
}

// --- Config file parsing ------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Top,
    Templates,
    Components,
    Tools,
}

/// Parse a config.yml previously written by the scaffold back into
/// [`Selections`]. Unknown keys are ignored for forward compatibility;
/// unknown enum values are errors.
pub fn parse_config(text: &str) -> Result<Selections, String> {
    let mut selections = Selections {
        profile: Profile::Solo,
        shape: ProjectShape::SingleProject,
        tools: Vec::new(),
        templates: Vec::new(),
        custom_template: None,
        plans_dir: "plans/".to_string(),
        docs_dir: "docs/".to_string(),
        tooling_root: ".aps/".to_string(),
        components: Vec::new(),
    };
    let mut section = Section::Top;

    for (number, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let err = |message: String| format!("config line {}: {message}", number + 1);

        // Top-level keys reset the section.
        if !line.starts_with(' ') {
            let Some((key, value)) = line.split_once(':') else {
                return Err(err(format!("expected 'key: value', got '{line}'")));
            };
            let value = value.trim();
            section = Section::Top;
            match key {
                "profile" => selections.profile = profile_from_key(value).map_err(err)?,
                "shape" => selections.shape = shape_from_key(value).map_err(err)?,
                "plans_dir" => selections.plans_dir = value.to_string(),
                "docs_dir" => selections.docs_dir = value.to_string(),
                "tooling_root" => selections.tooling_root = value.to_string(),
                "custom_template" => selections.custom_template = Some(value.to_string()),
                "templates" => section = Section::Templates,
                "components" => section = Section::Components,
                "tools" => section = Section::Tools,
                _ => {} // forward compatibility
            }
            continue;
        }

        match section {
            Section::Templates => {
                let item = trimmed.trim_start_matches("- ").trim();
                selections
                    .templates
                    .push(template_from_key(item).map_err(err)?);
            }
            Section::Components => {
                let item = trimmed.trim_start_matches("- ").trim();
                selections
                    .components
                    .push(component_from_key(item).map_err(err)?);
            }
            Section::Tools => {
                if let Some(value) = trimmed.strip_prefix("- name:") {
                    let tool = tool_from_key(value.trim()).map_err(err)?;
                    selections.tools.push(ToolConfig {
                        tool,
                        install_agents: false,
                        hooks: HookVerbosity::None,
                        model: ModelPreference::Default,
                    });
                } else {
                    let Some((key, value)) = trimmed.split_once(':') else {
                        return Err(err(format!("expected 'key: value', got '{trimmed}'")));
                    };
                    let value = value.trim();
                    let Some(config) = selections.tools.last_mut() else {
                        return Err(err("tool attribute before '- name:'".to_string()));
                    };
                    match key.trim() {
                        "agents" => {
                            config.install_agents = value
                                .parse()
                                .map_err(|_| err(format!("invalid bool '{value}'")))?;
                        }
                        "hooks" => config.hooks = hooks_from_key(value).map_err(err)?,
                        "model" => config.model = model_from_key(value).map_err(err)?,
                        _ => {}
                    }
                }
            }
            Section::Top => {
                return Err(err(format!("unexpected indented line '{trimmed}'")));
            }
        }
    }

    Ok(selections)
}

// --- Flag handling -------------------------------------------------------------

/// Raw flag values from clap, before validation.
#[derive(Debug, Default)]
pub struct InitFlags {
    pub profile: Option<String>,
    pub shape: Option<String>,
    pub tools: Vec<String>,
    pub templates: Vec<String>,
    pub custom_template: Option<String>,
    pub plans_dir: Option<String>,
    pub docs_dir: Option<String>,
    pub tooling_root: Option<String>,
    pub components: Vec<String>,
    pub hooks: Option<String>,
    pub model: Option<String>,
    pub no_agents: bool,
}

impl InitFlags {
    fn is_empty(&self) -> bool {
        self.profile.is_none()
            && self.shape.is_none()
            && self.tools.is_empty()
            && self.templates.is_empty()
            && self.custom_template.is_none()
            && self.plans_dir.is_none()
            && self.docs_dir.is_none()
            && self.tooling_root.is_none()
            && self.components.is_empty()
            && self.hooks.is_none()
            && self.model.is_none()
            && !self.no_agents
    }
}

/// Templates implied by profile + shape — mirrors the wizard's defaults.
pub fn default_templates(profile: Profile, shape: ProjectShape) -> Vec<Template> {
    let mut templates = vec![match profile {
        Profile::Solo => Template::Quickstart,
        Profile::Team | Profile::AgentOperator => Template::Module,
    }];
    templates.push(match shape {
        ProjectShape::SingleProject => Template::Index,
        ProjectShape::Monorepo => Template::MonorepoIndex,
    });
    templates
}

const ALL_COMPONENTS: [Component; 5] = [
    Component::LintRules,
    Component::ApsRules,
    Component::ProjectContext,
    Component::DesignsDir,
    Component::DecisionsDir,
];

/// Build [`Selections`] for a non-interactive run: start from the config
/// file when given, apply flag overrides, fill remaining gaps with the
/// same defaults the wizard uses.
pub fn build_selections(base: Option<Selections>, flags: &InitFlags) -> Result<Selections, String> {
    let from_config = base.is_some();
    let mut selections = base.unwrap_or(Selections {
        profile: Profile::Solo,
        shape: ProjectShape::SingleProject,
        tools: Vec::new(),
        templates: Vec::new(),
        custom_template: None,
        plans_dir: "plans/".to_string(),
        docs_dir: "docs/".to_string(),
        tooling_root: ".aps/".to_string(),
        components: ALL_COMPONENTS.to_vec(),
    });

    if let Some(profile) = &flags.profile {
        selections.profile = profile_from_key(profile)?;
    }
    if let Some(shape) = &flags.shape {
        selections.shape = shape_from_key(shape)?;
    }
    if !flags.templates.is_empty() {
        selections.templates = flags
            .templates
            .iter()
            .map(|key| template_from_key(key))
            .collect::<Result<_, _>>()?;
    } else if selections.templates.is_empty() && !from_config {
        selections.templates = default_templates(selections.profile, selections.shape);
    }
    if let Some(custom) = &flags.custom_template {
        selections.custom_template = Some(custom.clone());
    }
    if let Some(plans_dir) = &flags.plans_dir {
        selections.plans_dir = plans_dir.clone();
    }
    if let Some(docs_dir) = &flags.docs_dir {
        selections.docs_dir = docs_dir.clone();
    }
    if let Some(tooling_root) = &flags.tooling_root {
        selections.tooling_root = tooling_root.clone();
    }
    if !flags.components.is_empty() {
        selections.components = flags
            .components
            .iter()
            .map(|key| component_from_key(key))
            .collect::<Result<_, _>>()?;
    }
    if !flags.tools.is_empty() {
        selections.tools = flags
            .tools
            .iter()
            .map(|key| tool_from_key(key).map(ToolConfig::default_for))
            .collect::<Result<_, _>>()?;
    }

    // Global per-tool overrides.
    for config in &mut selections.tools {
        if let Some(hooks) = &flags.hooks {
            config.hooks = hooks_from_key(hooks)?;
        }
        if let Some(model) = &flags.model {
            config.model = model_from_key(model)?;
        }
        if flags.no_agents {
            config.install_agents = false;
        }
    }

    Ok(selections)
}

/// Decide whether `aps init` should run the TUI. Flags or a config file
/// force non-interactive mode; a missing terminal falls back to defaults.
pub fn wants_tui(non_interactive: bool, from: Option<&Path>, flags: &InitFlags, tty: bool) -> bool {
    !non_interactive && from.is_none() && flags.is_empty() && tty
}

/// Run the scaffold to completion, printing one line per step. Returns
/// an error summary when any step failed.
pub fn run_scaffold_console(root: &Path, selections: &Selections) -> Result<(), String> {
    scaffold::check_target(root, selections)?;

    let mut run = ScaffoldRun::new(root.to_path_buf(), selections);
    let mut index = 0;
    while run.run_next() {
        let (label, status) = run
            .steps()
            .nth(index)
            .expect("step exists for executed index");
        match status {
            StepStatus::Done => println!("==> {label} ... ok"),
            StepStatus::Failed(message) => println!("==> {label} ... FAILED: {message}"),
            StepStatus::Pending => {}
        }
        index += 1;
    }

    let failures = run.failures();
    if failures.is_empty() {
        println!("\nScaffold complete.");
        for config in &selections.tools {
            if let Some(note) = scaffold::post_install_note(config.tool) {
                println!("  - {note}");
            }
        }
        Ok(())
    } else {
        Err(format!("{} scaffold step(s) failed", failures.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::config_yaml;
    use std::fs;
    use std::path::PathBuf;

    fn sample_selections() -> Selections {
        Selections {
            profile: Profile::Team,
            shape: ProjectShape::Monorepo,
            tools: vec![
                ToolConfig {
                    tool: AiTool::ClaudeCode,
                    install_agents: true,
                    hooks: HookVerbosity::Minimal,
                    model: ModelPreference::Opus,
                },
                ToolConfig {
                    tool: AiTool::Codex,
                    install_agents: false,
                    hooks: HookVerbosity::None,
                    model: ModelPreference::Sonnet,
                },
            ],
            templates: vec![Template::Module, Template::MonorepoIndex],
            custom_template: Some("tpl.md".to_string()),
            plans_dir: "plans".to_string(),
            docs_dir: "docs".to_string(),
            tooling_root: ".aps".to_string(),
            components: vec![Component::LintRules, Component::ApsRules],
        }
    }

    #[test]
    fn config_yaml_round_trips_through_parser() {
        let original = sample_selections();
        let parsed = parse_config(&config_yaml(&original)).unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    fn parser_rejects_unknown_enum_values() {
        let err = parse_config("profile: wizard\n").unwrap_err();
        assert!(err.contains("unknown profile"), "got: {err}");

        let err = parse_config("templates:\n  - nope\n").unwrap_err();
        assert!(err.contains("unknown template"), "got: {err}");
    }

    #[test]
    fn parser_ignores_unknown_keys() {
        let parsed = parse_config("profile: solo\nfuture_key: whatever\n").unwrap();
        assert_eq!(parsed.profile, Profile::Solo);
    }

    #[test]
    fn flags_build_selections_with_smart_defaults() {
        let flags = InitFlags {
            profile: Some("solo".to_string()),
            ..InitFlags::default()
        };
        let selections = build_selections(None, &flags).unwrap();

        assert_eq!(selections.profile, Profile::Solo);
        assert_eq!(selections.shape, ProjectShape::SingleProject);
        assert_eq!(
            selections.templates,
            vec![Template::Quickstart, Template::Index]
        );
        assert_eq!(selections.components, ALL_COMPONENTS.to_vec());
        assert!(selections.tools.is_empty());
    }

    #[test]
    fn flags_override_config_base() {
        let base = sample_selections();
        let flags = InitFlags {
            plans_dir: Some("docs/plans".to_string()),
            hooks: Some("none".to_string()),
            ..InitFlags::default()
        };
        let selections = build_selections(Some(base.clone()), &flags).unwrap();

        assert_eq!(selections.plans_dir, "docs/plans");
        assert_eq!(selections.profile, base.profile);
        assert!(
            selections
                .tools
                .iter()
                .all(|config| config.hooks == HookVerbosity::None)
        );
    }

    #[test]
    fn tool_flags_get_default_tool_configs() {
        let flags = InitFlags {
            tools: vec!["claude-code".to_string(), "generic".to_string()],
            no_agents: true,
            ..InitFlags::default()
        };
        let selections = build_selections(None, &flags).unwrap();

        assert_eq!(selections.tools.len(), 2);
        assert!(selections.tools.iter().all(|c| !c.install_agents));
    }

    #[test]
    fn tui_only_when_interactive_and_unflagged() {
        let empty = InitFlags::default();
        let flagged = InitFlags {
            shape: Some("monorepo".to_string()),
            ..InitFlags::default()
        };

        assert!(wants_tui(false, None, &empty, true));
        assert!(!wants_tui(true, None, &empty, true));
        assert!(!wants_tui(false, Some(Path::new("c.yml")), &empty, true));
        assert!(!wants_tui(false, None, &flagged, true));
        assert!(!wants_tui(false, None, &empty, false));
    }

    #[test]
    fn console_scaffold_matches_config_replay() {
        let root = std::env::temp_dir().join(format!("aps-config-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        // First run from flags.
        let flags = InitFlags {
            profile: Some("team".to_string()),
            shape: Some("monorepo".to_string()),
            tools: vec!["claude-code".to_string()],
            ..InitFlags::default()
        };
        let selections = build_selections(None, &flags).unwrap();
        let first = root.join("first");
        fs::create_dir_all(&first).unwrap();
        run_scaffold_console(&first, &selections).unwrap();

        // Replay from the config the first run wrote.
        let config_text = fs::read_to_string(first.join(".aps/config.yml")).unwrap();
        let replayed = parse_config(&config_text).unwrap();
        let second = root.join("second");
        fs::create_dir_all(&second).unwrap();
        run_scaffold_console(&second, &replayed).unwrap();

        // Same files in both trees.
        let mut first_files = list_files(&first);
        let mut second_files = list_files(&second);
        first_files.sort();
        second_files.sort();
        assert_eq!(first_files, second_files);

        fs::remove_dir_all(&root).unwrap();
    }

    fn list_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    files.push(path.strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
        files
    }
}

//! Non-interactive init: flag handling and config-file replay (TUI-005).
//!
//! `aps init --non-interactive` builds [`Selections`] from CLI flags;
//! `aps init --from <config.yml>` replays a configuration previously
//! written by the scaffold (see [`crate::scaffold::config_yaml`]). The
//! parser covers exactly that emitted shape — a deliberate subset of
//! YAML — so no serde dependency is needed.

use std::path::{Path, PathBuf};

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
        "index-nested" => Ok(Template::IndexNested),
        other => Err(format!(
            "unknown template '{other}' (expected quickstart, module, index, monorepo-index, or index-nested)"
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
        "releases-dir" => Ok(Component::ReleasesDir),
        other => Err(format!("unknown component '{other}'")),
    }
}

pub fn tool_from_key(key: &str) -> Result<AiTool, String> {
    match key {
        "claude-code" => Ok(AiTool::ClaudeCode),
        "copilot" => Ok(AiTool::Copilot),
        "codex" => Ok(AiTool::Codex),
        "opencode" => Ok(AiTool::OpenCode),
        "grok" => Ok(AiTool::Grok),
        "gemini" => Err("'gemini' was retired in v0.7 (D-040); supported tools: \
             claude-code, copilot, codex, opencode, grok, generic"
            .to_string()),
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
        cli_version: None,
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
                "cli_version" => selections.cli_version = Some(value.to_string()),
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

const ALL_COMPONENTS: [Component; 6] = [
    Component::LintRules,
    Component::ApsRules,
    Component::ProjectContext,
    Component::DesignsDir,
    Component::DecisionsDir,
    Component::ReleasesDir,
];

/// Build [`Selections`] for a non-interactive run: start from the config
/// file when given, apply flag overrides, fill remaining gaps with the
/// same defaults the wizard uses.
pub fn build_selections(base: Option<Selections>, flags: &InitFlags) -> Result<Selections, String> {
    let from_config = base.is_some();
    let shape_overridden = flags.shape.is_some();
    let templates_overridden = !flags.templates.is_empty();
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
        cli_version: None,
    });

    if let Some(profile) = &flags.profile {
        selections.profile = profile_from_key(profile)?;
    }
    if let Some(shape) = &flags.shape {
        selections.shape = shape_from_key(shape)?;
    }
    if templates_overridden {
        selections.templates = flags
            .templates
            .iter()
            .map(|key| template_from_key(key))
            .collect::<Result<_, _>>()?;
    } else if selections.templates.is_empty() && !from_config {
        selections.templates = default_templates(selections.profile, selections.shape);
    }

    // Flags override replayed config. When only shape changes, carry the
    // inherited non-root templates forward but replace the old shape's root
    // index. Explicit --templates remain authoritative and are validated
    // below instead of being silently repaired.
    if shape_overridden
        && !templates_overridden
        && !selections.templates.contains(&Template::IndexNested)
    {
        selections.templates.retain(|template| {
            !matches!(
                template,
                Template::Index | Template::MonorepoIndex | Template::IndexNested
            )
        });
        selections.templates.push(match selections.shape {
            ProjectShape::SingleProject => Template::Index,
            ProjectShape::Monorepo => Template::MonorepoIndex,
        });
    }

    let has_plain_index = selections.templates.contains(&Template::Index);
    let has_monorepo_index = selections.templates.contains(&Template::MonorepoIndex);
    let has_nested_index = selections.templates.contains(&Template::IndexNested);
    let root_template_count = [has_plain_index, has_monorepo_index, has_nested_index]
        .into_iter()
        .filter(|selected| *selected)
        .count();
    if root_template_count > 1 {
        return Err(
            "templates conflict: choose exactly one of index, monorepo-index, or index-nested"
                .to_string(),
        );
    }
    if !has_nested_index {
        match selections.shape {
            ProjectShape::SingleProject if has_monorepo_index => {
                return Err(
                    "shape 'single' conflicts with template 'monorepo-index'; use 'index'"
                        .to_string(),
                );
            }
            ProjectShape::Monorepo if has_plain_index => {
                return Err(
                    "shape 'monorepo' conflicts with template 'index'; use 'monorepo-index'"
                        .to_string(),
                );
            }
            ProjectShape::SingleProject if !has_plain_index => {
                selections.templates.push(Template::Index);
            }
            ProjectShape::Monorepo if !has_monorepo_index => {
                selections.templates.push(Template::MonorepoIndex);
            }
            _ => {}
        }
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

    // Stamp the toolchain pin (D-035). A replayed config keeps its own
    // cli_version; an older config that predates the field inherits the
    // running binary's version, with a warning.
    if selections.cli_version.is_none() {
        if from_config {
            eprintln!(
                "warning: source config has no cli_version; stamping {}",
                crate::scaffold::CLI_VERSION
            );
        }
        selections.cli_version = Some(crate::scaffold::CLI_VERSION.to_string());
    }

    Ok(selections)
}

/// Decide whether `aps init` should run the TUI. Flags or a config file
/// force non-interactive mode; a missing terminal falls back to defaults.
pub fn wants_tui(non_interactive: bool, from: Option<&Path>, flags: &InitFlags, tty: bool) -> bool {
    !non_interactive && from.is_none() && flags.is_empty() && tty
}

// --- Runtime project config discovery (INSTALL-016) --------------------------

/// A project contract discovered by walking up for `.aps/config.yml`.
pub struct ProjectConfig {
    pub root: PathBuf,
    pub plans_dir: Option<String>,
    pub cli_version: Option<String>,
}

/// Read a top-level `key: value` scalar, ignoring indented keys and comments.
/// Tolerant of both the flat (Rust) and nested (bash) config shapes.
fn read_top_scalar(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':')
            && k.trim() == key
        {
            let v = v.trim().trim_matches(['"', '\'']).trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Walk up from `start` for the nearest `.aps/config.yml` and read its
/// contract fields. Returns `None` when no project config is found.
pub fn discover_project(start: &Path) -> Option<ProjectConfig> {
    let mut dir = Some(start);
    while let Some(current) = dir {
        let candidate = current.join(".aps/config.yml");
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).unwrap_or_default();
            return Some(ProjectConfig {
                root: current.to_path_buf(),
                plans_dir: read_top_scalar(&text, "plans_dir"),
                cli_version: read_top_scalar(&text, "cli_version"),
            });
        }
        dir = current.parent();
    }
    None
}

/// Resolve the effective plans directory for a project-scoped command:
///   APS_PLANS (MCP/manual override) > discovered plans_dir > "plans".
pub fn default_plans(start: &Path) -> String {
    if let Ok(env) = std::env::var("APS_PLANS")
        && !env.is_empty()
    {
        return env.trim_end_matches('/').to_string();
    }
    if let Some(project) = discover_project(start)
        && let Some(plans) = project.plans_dir
    {
        let plans = plans.trim_end_matches('/');
        return if project.root == start {
            plans.to_string()
        } else {
            project.root.join(plans).to_string_lossy().into_owned()
        };
    }
    "plans".to_string()
}

/// Warn when the project's `cli_version` pin differs from this binary. Under
/// `strict`, a mismatch returns `Err` so callers can exit non-zero (CI).
pub fn check_cli_version(start: &Path, strict: bool) -> Result<(), String> {
    let Some(project) = discover_project(start) else {
        return Ok(());
    };
    let Some(pin) = project.cli_version else {
        return Ok(());
    };
    if pin != crate::scaffold::CLI_VERSION {
        eprintln!(
            "warning: project pins cli_version {pin} but this CLI is {}",
            crate::scaffold::CLI_VERSION
        );
        if strict {
            return Err("cli_version mismatch under --strict".to_string());
        }
    }
    Ok(())
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
            components: vec![
                Component::LintRules,
                Component::ApsRules,
                Component::ReleasesDir,
            ],
            cli_version: Some("9.9.9".to_string()),
        }
    }

    #[test]
    fn config_yaml_round_trips_through_parser() {
        let original = sample_selections();
        let parsed = parse_config(&config_yaml(&original)).unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    fn parses_alternate_plans_dir_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test/fixtures/config/alt-plans-dir.yml");
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let parsed = parse_config(&text).unwrap();
        assert_eq!(parsed.plans_dir, "docs/plans/");
        assert_eq!(parsed.cli_version.as_deref(), Some("1.2.3"));
        assert_eq!(parsed.shape, ProjectShape::Monorepo);
        // Replaying it preserves the pinned toolchain version.
        let replayed = build_selections(Some(parsed), &InitFlags::default()).unwrap();
        assert_eq!(replayed.cli_version.as_deref(), Some("1.2.3"));
        assert_eq!(replayed.plans_dir, "docs/plans/");
    }

    fn discovery_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "aps-discover-{tag}-{}-{}",
            std::process::id(),
            tag.len()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".aps")).unwrap();
        root
    }

    #[test]
    fn discovers_plans_dir_and_version_walking_up() {
        let root = discovery_root("plansdir");
        fs::write(
            root.join(".aps/config.yml"),
            "cli_version: 1.2.3\nplans_dir: docs/plans/\n",
        )
        .unwrap();
        let nested = root.join("packages/app");
        fs::create_dir_all(&nested).unwrap();

        let project = discover_project(&nested).expect("config discovered by walking up");
        assert_eq!(project.cli_version.as_deref(), Some("1.2.3"));
        assert_eq!(project.plans_dir.as_deref(), Some("docs/plans/"));

        // From the project root, the plans dir is returned as-is.
        assert_eq!(default_plans(&root), "docs/plans");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn check_cli_version_respects_strict() {
        let root = discovery_root("strict");
        fs::write(root.join(".aps/config.yml"), "cli_version: 9.9.9\n").unwrap();
        // Non-strict warns but succeeds; strict fails.
        assert!(check_cli_version(&root, false).is_ok());
        assert!(check_cli_version(&root, true).is_err());

        // A matching pin never fails, even strict.
        fs::write(
            root.join(".aps/config.yml"),
            format!("cli_version: {}\n", crate::scaffold::CLI_VERSION),
        )
        .unwrap();
        assert!(check_cli_version(&root, true).is_ok());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_top_scalar_tolerates_nested_bash_shape() {
        // The bash installer writes a nested aps:/project: shape; the top-level
        // contract keys must still be readable.
        let text = "cli_version: \"0.3.0\"\nplans_dir: plans/\naps:\n  version: \"0.3.0\"\n";
        assert_eq!(
            read_top_scalar(text, "cli_version").as_deref(),
            Some("0.3.0")
        );
        assert_eq!(
            read_top_scalar(text, "plans_dir").as_deref(),
            Some("plans/")
        );
        // Indented keys are not treated as top-level.
        assert_eq!(read_top_scalar(text, "version"), None);
    }

    #[test]
    fn cli_version_is_stamped_and_replayed() {
        // A config without cli_version inherits the running binary's version.
        let stamped = build_selections(None, &InitFlags::default()).unwrap();
        assert_eq!(
            stamped.cli_version.as_deref(),
            Some(crate::scaffold::CLI_VERSION)
        );

        // An explicit cli_version round-trips and is preserved on replay.
        let pinned = parse_config("cli_version: 1.2.3\nprofile: solo\n").unwrap();
        assert_eq!(pinned.cli_version.as_deref(), Some("1.2.3"));
        let replayed = build_selections(Some(pinned), &InitFlags::default()).unwrap();
        assert_eq!(replayed.cli_version.as_deref(), Some("1.2.3"));
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
    fn flags_reject_root_index_that_conflicts_with_project_shape() {
        let flags = InitFlags {
            shape: Some("monorepo".to_string()),
            templates: vec!["index".to_string()],
            ..InitFlags::default()
        };

        let err = build_selections(None, &flags).unwrap_err();
        assert!(err.contains("monorepo-index"), "got: {err}");
    }

    #[test]
    fn shape_flag_replaces_inherited_root_template_in_both_directions() {
        let mut single = sample_selections();
        single.shape = ProjectShape::SingleProject;
        single.templates = vec![Template::Module, Template::Index];
        let to_monorepo = build_selections(
            Some(single),
            &InitFlags {
                shape: Some("monorepo".to_string()),
                ..InitFlags::default()
            },
        )
        .unwrap();
        assert_eq!(to_monorepo.shape, ProjectShape::Monorepo);
        assert!(to_monorepo.templates.contains(&Template::MonorepoIndex));
        assert!(!to_monorepo.templates.contains(&Template::Index));

        let mut monorepo = sample_selections();
        monorepo.shape = ProjectShape::Monorepo;
        monorepo.templates = vec![Template::Module, Template::MonorepoIndex];
        let to_single = build_selections(
            Some(monorepo),
            &InitFlags {
                shape: Some("single".to_string()),
                ..InitFlags::default()
            },
        )
        .unwrap();
        assert_eq!(to_single.shape, ProjectShape::SingleProject);
        assert!(to_single.templates.contains(&Template::Index));
        assert!(!to_single.templates.contains(&Template::MonorepoIndex));

        // A federated root is independent of the package-layout shape and is
        // therefore preserved when only --shape changes.
        let mut nested = sample_selections();
        nested.shape = ProjectShape::SingleProject;
        nested.templates = vec![Template::Module, Template::IndexNested];
        let nested_with_monorepo_shape = build_selections(
            Some(nested),
            &InitFlags {
                shape: Some("monorepo".to_string()),
                ..InitFlags::default()
            },
        )
        .unwrap();
        assert!(
            nested_with_monorepo_shape
                .templates
                .contains(&Template::IndexNested)
        );
        assert!(
            !nested_with_monorepo_shape
                .templates
                .contains(&Template::MonorepoIndex)
        );
    }

    #[test]
    fn flags_reject_multiple_root_templates_including_nested() {
        let flags = InitFlags {
            templates: vec!["index-nested".to_string(), "index".to_string()],
            ..InitFlags::default()
        };

        let err = build_selections(None, &flags).unwrap_err();
        assert!(err.contains("choose exactly one"), "got: {err}");
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

//! Native scaffold execution for `aps init` (TUI-004).
//!
//! The wizard collects [`Selections`]; [`plan_steps`] turns them into a
//! deterministic list of [`ScaffoldStep`]s (pure, unit-testable), and
//! [`ScaffoldRun`] executes one step per call so the TUI can redraw
//! between steps without threads.
//!
//! Template and skill content is embedded at compile time from the repo's
//! `scaffold/` and `templates/` trees, so the binary scaffolds offline.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::managed::{self, MANIFEST_NAME};
use crate::wizard::{
    AiTool, Component, HookVerbosity, ModelPreference, Profile, ProjectShape, Template, ToolConfig,
};

// --- Embedded assets -------------------------------------------------------

const INDEX_APS: &str = include_str!("../scaffold/plans/index.aps.md");
const APS_RULES: &str = include_str!("../scaffold/plans/aps-rules.md");
const PROJECT_CONTEXT: &str = include_str!("../scaffold/plans/project-context.md");
const MODULE_TEMPLATE: &str = include_str!("../scaffold/plans/modules/.module.template.md");
const SIMPLE_TEMPLATE: &str = include_str!("../scaffold/plans/modules/.simple.template.md");
const INDEX_MONOREPO_TEMPLATE: &str =
    include_str!("../scaffold/plans/modules/.index-monorepo.template.md");
const ACTIONS_TEMPLATE: &str = include_str!("../scaffold/plans/execution/.actions.template.md");
const DESIGN_TEMPLATE: &str = include_str!("../scaffold/designs/.design.template.md");
const QUICKSTART_TEMPLATE: &str = include_str!("../templates/quickstart.template.md");
const RELEASES_README: &str = include_str!("../scaffold/plans/releases/README.md");
const RELEASE_TEMPLATE: &str = include_str!("../templates/release.template.md");
// Federated nested-plans scaffold (MONO-005). The child module uses the same
// authoring template the bash CLI copies, so both produce byte-identical trees.
const INDEX_NESTED_TEMPLATE: &str = include_str!("../templates/index-nested.template.md");
const INDEX_CHILD_TEMPLATE: &str = include_str!("../templates/index-child.template.md");
const CHILD_MODULE_TEMPLATE: &str = include_str!("../templates/module.template.md");
/// Starter child packages a nested scaffold creates, each with a distinct
/// work-item prefix so bare IDs stay unique across trees (W020-clean).
const NESTED_CHILDREN: &[(&str, &str)] = &[("core", "CORE"), ("api", "API")];

// v2 skill: three files (hooks.md is v1-only), installed under the
// per-tool skill roots — never at the repo root (that's the v1 layout
// `aps migrate` removes).
pub const SKILL_FILES: [(&str, &str); 3] = [
    (
        "SKILL.md",
        include_str!("../scaffold/aps-planning/SKILL.md"),
    ),
    (
        "reference.md",
        include_str!("../scaffold/aps-planning/reference.md"),
    ),
    (
        "examples.md",
        include_str!("../scaffold/aps-planning/examples.md"),
    ),
];

/// Skill root read by Claude Code, Copilot, and OpenCode.
pub const CLAUDE_SKILL_DIR: &str = ".claude/skills/aps-planning";
/// Skill root discovered by Codex (`codex skills install`) and Grok Build.
pub const AGENTS_SKILL_DIR: &str = ".agents/skills/aps-planning";

/// Hook scripts live under `.aps/scripts/` (v2), not `aps-planning/scripts/`.
pub const HOOK_SCRIPTS: [(&str, &str); 12] = [
    (
        ".aps/scripts/install-hooks.sh",
        include_str!("../scaffold/aps-planning/scripts/install-hooks.sh"),
    ),
    (
        ".aps/scripts/init-session.sh",
        include_str!("../scaffold/aps-planning/scripts/init-session.sh"),
    ),
    (
        ".aps/scripts/check-complete.sh",
        include_str!("../scaffold/aps-planning/scripts/check-complete.sh"),
    ),
    (
        ".aps/scripts/pre-tool-check.sh",
        include_str!("../scaffold/aps-planning/scripts/pre-tool-check.sh"),
    ),
    (
        ".aps/scripts/post-tool-nudge.sh",
        include_str!("../scaffold/aps-planning/scripts/post-tool-nudge.sh"),
    ),
    (
        ".aps/scripts/enforce-plan-update.sh",
        include_str!("../scaffold/aps-planning/scripts/enforce-plan-update.sh"),
    ),
    (
        ".aps/scripts/install-hooks.ps1",
        include_str!("../scaffold/aps-planning/scripts/install-hooks.ps1"),
    ),
    (
        ".aps/scripts/init-session.ps1",
        include_str!("../scaffold/aps-planning/scripts/init-session.ps1"),
    ),
    (
        ".aps/scripts/check-complete.ps1",
        include_str!("../scaffold/aps-planning/scripts/check-complete.ps1"),
    ),
    (
        ".aps/scripts/pre-tool-check.ps1",
        include_str!("../scaffold/aps-planning/scripts/pre-tool-check.ps1"),
    ),
    (
        ".aps/scripts/post-tool-nudge.ps1",
        include_str!("../scaffold/aps-planning/scripts/post-tool-nudge.ps1"),
    ),
    (
        ".aps/scripts/enforce-plan-update.ps1",
        include_str!("../scaffold/aps-planning/scripts/enforce-plan-update.ps1"),
    ),
];

// --- Agent generation (install-time envelope + shared cores) -----------------

const PLANNER_CORE: &str = include_str!("../scaffold/agents/core/planner-core.md");
const LIBRARIAN_CORE: &str = include_str!("../scaffold/agents/core/librarian-core.md");
const CONDUCTOR_CORE: &str = include_str!("../scaffold/agents/core/conductor-core.md");

const PLANNER_DESC: &str = "Create, manage, execute, and review plans following the Anvil Plan Spec (APS) format, including initializing projects, modules, work items, action plans, validation, status tracking, and wave-based parallel execution";
const LIBRARIAN_DESC: &str = "Repository organizing, cleanup, documentation filing, archiving stale specs, detecting orphaned files, cross-reference maintenance, and general repo hygiene";
const CONDUCTOR_DESC: &str = "Coordinate APS execution through CLI-backed next-work selection, context packaging, dependency checks, validation, and learning capture";

/// APS agent roles installed for tools that support agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Planner,
    Librarian,
    Conductor,
}

impl AgentRole {
    const ALL: [AgentRole; 3] = [
        AgentRole::Planner,
        AgentRole::Librarian,
        AgentRole::Conductor,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Planner => "aps-planner",
            Self::Librarian => "aps-librarian",
            Self::Conductor => "aps-conductor",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Planner => PLANNER_DESC,
            Self::Librarian => LIBRARIAN_DESC,
            Self::Conductor => CONDUCTOR_DESC,
        }
    }

    fn core(self) -> &'static str {
        match self {
            Self::Planner => PLANNER_CORE,
            Self::Librarian => LIBRARIAN_CORE,
            Self::Conductor => CONDUCTOR_CORE,
        }
    }

    /// Heavy roles get the premium tier under `ModelPreference::Default`.
    fn is_heavy(self) -> bool {
        matches!(self, Self::Planner | Self::Conductor)
    }
}

/// Resolve the concrete model ID for a tool/role/preference triple.
///
/// Preference keys stay `default` / `opus` / `sonnet` in config for stability;
/// they act as tiers:
/// - `default` — role-weighted map per vendor
/// - `opus` — premium tier for every role
/// - `sonnet` — balanced tier for every role
///
/// Returns `None` when the tool does not expose a model field (Copilot).
pub fn resolve_model(
    tool: AiTool,
    role: AgentRole,
    preference: ModelPreference,
) -> Option<&'static str> {
    match tool {
        AiTool::ClaudeCode => Some(match preference {
            ModelPreference::Default if role.is_heavy() => "opus",
            ModelPreference::Default => "sonnet",
            ModelPreference::Opus => "opus",
            ModelPreference::Sonnet => "sonnet",
        }),
        // OpenCode is multi-provider: `default` omits `model` so each user's
        // configured provider/model wins. Explicit opus/sonnet pin OpenAI
        // (Codex-family) IDs — not Anthropic — when a fallback is requested.
        AiTool::OpenCode => match preference {
            ModelPreference::Default => None,
            ModelPreference::Opus => Some("openai/gpt-5.6-sol"),
            ModelPreference::Sonnet => Some("openai/gpt-5.6-terra"),
        },
        AiTool::Codex => Some(match preference {
            // OpenAI GPT-5.6 family (API ids).
            ModelPreference::Default if role.is_heavy() => "gpt-5.6-sol",
            ModelPreference::Default => "gpt-5.6-terra",
            ModelPreference::Opus => "gpt-5.6-sol",
            ModelPreference::Sonnet => "gpt-5.6-terra",
        }),
        AiTool::Copilot
        | AiTool::Grok
        | AiTool::Antigravity
        | AiTool::Amp
        | AiTool::GeminiCli
        | AiTool::Windsurf
        | AiTool::RooCode
        | AiTool::OpenClaw
        | AiTool::Generic => None,
    }
}

fn render_claude_code(role: AgentRole, model: &str) -> String {
    let tools = match role {
        AgentRole::Librarian => ["Read", "Write", "Edit", "Glob", "Grep", "Bash"].as_slice(),
        AgentRole::Planner | AgentRole::Conductor => {
            ["Read", "Write", "Edit", "Glob", "Grep", "Bash", "Task"].as_slice()
        }
    };
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "name: {}", role.name());
    let _ = writeln!(out, "description: {}", role.description());
    let _ = writeln!(out, "model: {model}");
    out.push_str("tools:\n");
    for tool in tools {
        let _ = writeln!(out, "  - {tool}");
    }
    out.push_str("---\n\n");
    out.push_str(role.core());
    out
}

fn render_copilot(role: AgentRole) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "name: {}", role.name());
    let _ = writeln!(out, "description: {}", role.description());
    out.push_str("---\n\n");
    out.push_str(role.core());
    out
}

fn render_opencode(role: AgentRole, model: Option<&str>) -> String {
    let steps = match role {
        AgentRole::Librarian => 30,
        AgentRole::Planner | AgentRole::Conductor => 50,
    };
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "description: {}", role.description());
    out.push_str("mode: subagent\n");
    if let Some(model) = model {
        let _ = writeln!(out, "model: {model}");
    }
    let _ = writeln!(out, "steps: {steps}");
    out.push_str("tools:\n");
    out.push_str("  read: true\n");
    out.push_str("  write: true\n");
    out.push_str("  edit: true\n");
    out.push_str("  glob: true\n");
    out.push_str("  grep: true\n");
    out.push_str("  bash: true\n");
    out.push_str("permission:\n");
    out.push_str("  edit: \"ask\"\n");
    out.push_str("  write: \"ask\"\n");
    out.push_str("  bash: \"ask\"\n");
    out.push_str("---\n\n");
    out.push_str(role.core());
    out
}

fn render_codex(role: AgentRole, model: &str) -> String {
    let comment = match role {
        AgentRole::Planner => "Planner",
        AgentRole::Librarian => "Librarian",
        AgentRole::Conductor => "Conductor",
    };
    let mut out = String::new();
    let _ = writeln!(out, "# APS {comment} — Codex Agent Role");
    out.push_str("#\n");
    out.push_str("# Codex discovers this role automatically from .codex/agents/.\n");
    out.push('\n');
    let _ = writeln!(out, "name = \"{}\"", role.name());
    let _ = writeln!(out, "description = \"{}\"", role.description());
    out.push('\n');
    let _ = writeln!(out, "model = \"{model}\"  # OpenAI model");
    out.push('\n');
    out.push_str("sandbox_mode = \"workspace-write\"\n");
    out.push('\n');
    out.push_str("developer_instructions = \"\"\"\n");
    out.push_str(role.core());
    if !role.core().ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\"\"\"\n");
    out
}

fn agent_dest(tool: AiTool, role: AgentRole) -> Option<&'static str> {
    match (tool, role) {
        (AiTool::ClaudeCode, AgentRole::Planner) => Some(".claude/agents/aps-planner.md"),
        (AiTool::ClaudeCode, AgentRole::Librarian) => Some(".claude/agents/aps-librarian.md"),
        (AiTool::ClaudeCode, AgentRole::Conductor) => Some(".claude/agents/aps-conductor.md"),
        (AiTool::Copilot, AgentRole::Planner) => Some(".github/agents/aps-planner.md"),
        (AiTool::Copilot, AgentRole::Librarian) => Some(".github/agents/aps-librarian.md"),
        (AiTool::Copilot, AgentRole::Conductor) => Some(".github/agents/aps-conductor.md"),
        (AiTool::Codex, AgentRole::Planner) => Some(".codex/agents/aps-planner.toml"),
        (AiTool::Codex, AgentRole::Librarian) => Some(".codex/agents/aps-librarian.toml"),
        (AiTool::Codex, AgentRole::Conductor) => Some(".codex/agents/aps-conductor.toml"),
        (AiTool::OpenCode, AgentRole::Planner) => Some(".opencode/agent/aps-planner.md"),
        (AiTool::OpenCode, AgentRole::Librarian) => Some(".opencode/agent/aps-librarian.md"),
        (AiTool::OpenCode, AgentRole::Conductor) => Some(".opencode/agent/aps-conductor.md"),
        (
            AiTool::Grok
            | AiTool::Antigravity
            | AiTool::Amp
            | AiTool::GeminiCli
            | AiTool::Windsurf
            | AiTool::RooCode
            | AiTool::OpenClaw
            | AiTool::Generic,
            _,
        ) => None,
    }
}

fn render_agent(tool: AiTool, role: AgentRole, preference: ModelPreference) -> Option<String> {
    match tool {
        AiTool::ClaudeCode => {
            let model = resolve_model(tool, role, preference)?;
            Some(render_claude_code(role, model))
        }
        AiTool::OpenCode => {
            let model = resolve_model(tool, role, preference);
            Some(render_opencode(role, model))
        }
        AiTool::Codex => {
            let model = resolve_model(tool, role, preference)?;
            Some(render_codex(role, model))
        }
        AiTool::Copilot => Some(render_copilot(role)),
        AiTool::Grok
        | AiTool::Antigravity
        | AiTool::Amp
        | AiTool::GeminiCli
        | AiTool::Windsurf
        | AiTool::RooCode
        | AiTool::OpenClaw
        | AiTool::Generic => None,
    }
}

/// Generated agent files for a tool as (destination relative path, content).
///
/// Content is produced at call time from shared cores + the model preference,
/// so install/update never depend on pre-baked model IDs in scaffold trees.
pub fn agent_files(tool: AiTool, preference: ModelPreference) -> Vec<(String, String)> {
    let mut files = Vec::new();
    for role in AgentRole::ALL {
        let Some(path) = agent_dest(tool, role) else {
            continue;
        };
        let Some(content) = render_agent(tool, role, preference) else {
            continue;
        };
        files.push((path.to_string(), content));
    }
    files
}

/// Paths of agent files a tool would install (for presence checks).
pub fn agent_paths(tool: AiTool) -> Vec<&'static str> {
    AgentRole::ALL
        .iter()
        .filter_map(|role| agent_dest(tool, *role))
        .collect()
}

/// Post-install instruction shown on the summary screen for a tool.
pub fn post_install_note(tool: AiTool) -> Option<&'static str> {
    match tool {
        AiTool::ClaudeCode => {
            if cfg!(windows) {
                Some("Claude Code: run .\\.aps\\scripts\\install-hooks.ps1 to wire hooks")
            } else {
                Some("Claude Code: run ./.aps/scripts/install-hooks.sh to wire hooks")
            }
        }
        AiTool::Copilot => Some("Copilot: commit .github/agents so Copilot picks them up"),
        AiTool::Codex => Some("Codex: run codex skills install .agents/skills/aps-planning"),
        AiTool::OpenCode => Some("OpenCode: agents installed under .opencode/agent"),
        AiTool::Grok => {
            Some("Grok: Grok Build discovers AGENTS.md and installed skills automatically")
        }
        AiTool::Antigravity => {
            Some("Antigravity: reads AGENTS.md and auto-discovers .agents/skills/ automatically")
        }
        AiTool::Amp => {
            Some("Amp: reads AGENTS.md and auto-discovers .agents/skills/ automatically")
        }
        AiTool::GeminiCli => {
            Some("Gemini CLI: reads AGENTS.md and auto-discovers .agents/skills/ automatically")
        }
        AiTool::Windsurf => {
            Some("Windsurf: reads AGENTS.md and auto-discovers .agents/skills/ automatically")
        }
        AiTool::RooCode => {
            Some("Roo Code: enable AGENTS.md auto-load; discovers .agents/skills/ automatically")
        }
        AiTool::OpenClaw => Some(
            "OpenClaw: injects AGENTS.md and auto-discovers .agents/skills/; drive plans with `aps next/start/complete`",
        ),
        AiTool::Generic => None,
    }
}

// --- Selections -------------------------------------------------------------

/// Everything the scaffold needs, decoupled from wizard navigation state so
/// the non-interactive path (TUI-005) can construct it from flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selections {
    pub profile: Profile,
    pub shape: ProjectShape,
    pub tools: Vec<ToolConfig>,
    pub templates: Vec<Template>,
    pub custom_template: Option<String>,
    pub plans_dir: String,
    pub docs_dir: String,
    pub tooling_root: String,
    pub components: Vec<Component>,
    /// Semver of the toolchain this project pins (`.aps/config.yml` contract).
    /// `None` while building; serialization stamps the running binary's
    /// version when unset. See INSTALL-014 / D-035.
    pub cli_version: Option<String>,
}

/// Semver of the running `aps` binary, stamped into `.aps/config.yml` on init.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

impl Selections {
    fn plans_path(&self) -> PathBuf {
        PathBuf::from(normalize_dir(&self.plans_dir, "plans"))
    }

    fn docs_path(&self) -> PathBuf {
        PathBuf::from(normalize_dir(&self.docs_dir, "docs"))
    }

    fn tooling_path(&self) -> PathBuf {
        PathBuf::from(normalize_dir(&self.tooling_root, ".aps"))
    }

    fn has_component(&self, component: Component) -> bool {
        self.components.contains(&component)
    }

    fn has_template(&self, template: Template) -> bool {
        self.templates.contains(&template)
    }

    fn hooks_requested(&self) -> bool {
        self.tools
            .iter()
            .any(|config| config.hooks != HookVerbosity::None)
    }

    fn agents_requested(&self) -> bool {
        self.tools.iter().any(|config| config.install_agents)
    }
}

/// Normalize a target directory, falling back to the default when the value
/// is empty or unsafe. Absolute paths would replace the scaffold root in
/// `Path::join`, and `..` components would escape it — both are rejected
/// here as defense in depth behind the wizard's own validation (the
/// non-interactive path reaches this function without going through the
/// wizard).
fn normalize_dir(value: &str, default: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    let path = Path::new(trimmed);
    let unsafe_path = path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir));
    if trimmed.is_empty() || unsafe_path {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

// --- Plan -------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOp {
    Mkdir(PathBuf),
    /// Write embedded content; never overwrites an existing file.
    Write {
        path: PathBuf,
        content: &'static str,
    },
    /// Write generated content (e.g. config.yml).
    WriteOwned {
        path: PathBuf,
        content: String,
    },
    /// Copy a user-supplied file (custom template) into the scaffold.
    CopyUser {
        from: PathBuf,
        to: PathBuf,
    },
    /// Mark a previously written script executable (no-op on non-unix).
    MarkExecutable(PathBuf),
    /// Refresh generated content in place (upgrade flows only — never
    /// used for user-authored files).
    Overwrite {
        path: PathBuf,
        content: &'static str,
    },
    /// Refresh owned (generated-at-install) content in place.
    OverwriteOwned {
        path: PathBuf,
        content: String,
    },
    /// Remove a known obsolete generated file. Missing files are a no-op.
    RemoveGenerated(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldStep {
    pub label: String,
    pub ops: Vec<FileOp>,
}

/// Build the ordered scaffold step list for the given selections. Pure —
/// no filesystem access — so every selection combination is testable.
pub fn plan_steps(selections: &Selections) -> Vec<ScaffoldStep> {
    let plans = selections.plans_path();
    let tooling = selections.tooling_path();
    let mut steps = Vec::new();

    // Directories.
    let mut dirs = vec![
        FileOp::Mkdir(plans.clone()),
        FileOp::Mkdir(plans.join("modules")),
        FileOp::Mkdir(plans.join("execution")),
        FileOp::Mkdir(selections.docs_path()),
        FileOp::Mkdir(tooling.clone()),
    ];
    if selections.has_component(Component::DecisionsDir) {
        dirs.push(FileOp::Mkdir(plans.join("decisions")));
    }
    if selections.has_component(Component::DesignsDir) {
        dirs.push(FileOp::Mkdir(plans.join("designs")));
    }
    if selections.has_component(Component::ReleasesDir) {
        dirs.push(FileOp::Mkdir(plans.join("releases")));
    }
    steps.push(ScaffoldStep {
        label: "Create directories".to_string(),
        ops: dirs,
    });

    // Plan files + templates.
    let mut templates = Vec::new();
    // Federated nested layout (MONO-005): the root becomes a federation index
    // and starter child plans are created under packages/<pkg>/plans/.
    let nested = selections.has_template(Template::IndexNested);
    let index_content = if nested {
        INDEX_NESTED_TEMPLATE
    } else {
        // Project shape owns the root plan. Template selection controls the
        // additional template assets, but a stale plain-index choice must not
        // turn a reviewed Monorepo selection back into a single-project root.
        match selections.shape {
            ProjectShape::SingleProject => INDEX_APS,
            ProjectShape::Monorepo => INDEX_MONOREPO_TEMPLATE,
        }
    };
    templates.push(FileOp::Write {
        path: plans.join("index.aps.md"),
        content: index_content,
    });
    if nested {
        // Starter child plans, each a complete standalone plan with one module.
        // Distinct work-item prefixes (AUTH -> CORE / API) keep bare IDs unique
        // across trees so the federation lints W020-clean out of the box.
        for (pkg, prefix) in NESTED_CHILDREN {
            let child_plans = PathBuf::from(format!("packages/{pkg}/plans"));
            templates.push(FileOp::Mkdir(child_plans.join("modules")));
            templates.push(FileOp::Write {
                path: child_plans.join("index.aps.md"),
                content: INDEX_CHILD_TEMPLATE,
            });
            templates.push(FileOp::WriteOwned {
                path: child_plans.join("modules/module-name.aps.md"),
                content: CHILD_MODULE_TEMPLATE.replace("AUTH", prefix),
            });
        }
    }
    templates.push(FileOp::Write {
        path: plans.join("execution/.actions.template.md"),
        content: ACTIONS_TEMPLATE,
    });
    if selections.has_template(Template::Quickstart) {
        templates.push(FileOp::Write {
            path: plans.join("modules/.quickstart.template.md"),
            content: QUICKSTART_TEMPLATE,
        });
        templates.push(FileOp::Write {
            path: plans.join("modules/.simple.template.md"),
            content: SIMPLE_TEMPLATE,
        });
    }
    if selections.has_template(Template::Module) {
        templates.push(FileOp::Write {
            path: plans.join("modules/.module.template.md"),
            content: MODULE_TEMPLATE,
        });
    }
    if selections.has_template(Template::MonorepoIndex) {
        templates.push(FileOp::Write {
            path: plans.join("modules/.index-monorepo.template.md"),
            content: INDEX_MONOREPO_TEMPLATE,
        });
    }
    if selections.has_component(Component::DesignsDir) {
        templates.push(FileOp::Write {
            path: plans.join("designs/.design.template.md"),
            content: DESIGN_TEMPLATE,
        });
    }
    if selections.has_component(Component::ReleasesDir) {
        templates.push(FileOp::Write {
            path: plans.join("releases/README.md"),
            content: RELEASES_README,
        });
        templates.push(FileOp::Write {
            path: plans.join("releases/.release.template.md"),
            content: RELEASE_TEMPLATE,
        });
    }
    if let Some(custom) = &selections.custom_template {
        let from = PathBuf::from(custom);
        let name = from
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "custom.template.md".to_string());
        templates.push(FileOp::CopyUser {
            from,
            to: plans.join(format!("modules/.{}", name.trim_start_matches('.'))),
        });
    }
    steps.push(ScaffoldStep {
        label: "Install templates".to_string(),
        ops: templates,
    });

    // Optional components.
    let mut components = Vec::new();
    if selections.has_component(Component::ApsRules) {
        components.push(FileOp::Write {
            path: plans.join("aps-rules.md"),
            content: APS_RULES,
        });
    }
    if selections.has_component(Component::ProjectContext) {
        components.push(FileOp::Write {
            path: plans.join("project-context.md"),
            content: PROJECT_CONTEXT,
        });
    }
    if !components.is_empty() {
        steps.push(ScaffoldStep {
            label: "Install components".to_string(),
            ops: components,
        });
    }

    // Planning skill + per-tool agents.
    if !selections.tools.is_empty()
        && let Some(step) = skill_step(&selections.tools)
    {
        steps.push(step);
    }

    if selections.agents_requested()
        && let Some(step) = agents_step(&selections.tools)
    {
        steps.push(step);
    }

    // Hook scripts (actual hook wiring is tool-specific; see summary notes).
    if selections.hooks_requested() {
        steps.push(hooks_step());
    }

    // Replayable configuration (consumed by `aps init --from`, TUI-005).
    steps.push(ScaffoldStep {
        label: "Write config".to_string(),
        ops: vec![FileOp::WriteOwned {
            path: tooling.join("config.yml"),
            content: config_yaml(selections),
        }],
    });

    steps
}

/// Planning skill files under the v2 per-tool skill roots. Claude Code,
/// Copilot, and OpenCode all read `.claude/skills/`; Codex and Grok discover
/// `.agents/skills/`. With no tools given (the tool-agnostic `setup all`
/// flow), the shared `.claude/skills/` root is used. Generic-only selections
/// install no skill, matching the bash scaffold — hence `None`.
pub fn skill_step(tools: &[ToolConfig]) -> Option<ScaffoldStep> {
    let claude_root = tools.is_empty()
        || tools.iter().any(|c| {
            matches!(
                c.tool,
                AiTool::ClaudeCode | AiTool::Copilot | AiTool::OpenCode
            )
        });
    let agents_root = tools.iter().any(|c| {
        matches!(
            c.tool,
            AiTool::Codex
                | AiTool::Grok
                | AiTool::Antigravity
                | AiTool::Amp
                | AiTool::GeminiCli
                | AiTool::Windsurf
                | AiTool::RooCode
                | AiTool::OpenClaw
        )
    });

    let mut ops = Vec::new();
    let marker_json = managed::expected_skill_manifest().to_json();
    for (dir, wanted) in [
        (CLAUDE_SKILL_DIR, claude_root),
        (AGENTS_SKILL_DIR, agents_root),
    ] {
        if !wanted {
            continue;
        }
        for (name, content) in SKILL_FILES {
            ops.push(FileOp::Write {
                path: PathBuf::from(dir).join(name),
                content,
            });
        }
        // Managed inventory sidecar so `aps update` can refuse dirty trees.
        ops.push(FileOp::WriteOwned {
            path: PathBuf::from(dir).join(MANIFEST_NAME),
            content: marker_json.clone(),
        });
    }
    (!ops.is_empty()).then(|| ScaffoldStep {
        label: "Install planning skill".to_string(),
        ops,
    })
}

/// Agent files for every tool with agents enabled; None when nothing to do.
pub fn agents_step(tools: &[ToolConfig]) -> Option<ScaffoldStep> {
    let mut agents = Vec::new();
    for config in tools {
        if !config.install_agents {
            continue;
        }
        if config.tool == AiTool::Codex {
            agents.push(FileOp::RemoveGenerated(PathBuf::from(
                ".codex/agents/codex-config-snippet.toml",
            )));
            agents.push(FileOp::RemoveGenerated(PathBuf::from(
                ".codex/codex-config-snippet.toml",
            )));
        }
        for (path, content) in agent_files(config.tool, config.model) {
            let path = PathBuf::from(path);
            // Generated content is owned; Codex always refreshes so legacy
            // role files pick up model + core updates.
            if config.tool == AiTool::Codex {
                agents.push(FileOp::OverwriteOwned { path, content });
            } else {
                agents.push(FileOp::WriteOwned { path, content });
            }
        }
    }
    (!agents.is_empty()).then(|| ScaffoldStep {
        label: "Install agents".to_string(),
        ops: agents,
    })
}

/// Hook scripts under .aps/scripts/, marked executable.
pub fn hooks_step() -> ScaffoldStep {
    let mut hooks = Vec::new();
    for (path, content) in HOOK_SCRIPTS {
        hooks.push(FileOp::Write {
            path: PathBuf::from(path),
            content,
        });
        if path.ends_with(".sh") {
            hooks.push(FileOp::MarkExecutable(PathBuf::from(path)));
        }
    }
    ScaffoldStep {
        label: "Configure hooks".to_string(),
        ops: hooks,
    }
}

/// Serialize selections as the replayable `config.yml`. Hand-rolled — the
/// shape is small and stable, not worth a serde dependency.
pub fn config_yaml(selections: &Selections) -> String {
    let mut out = String::from("# APS init configuration (replay with `aps init --from`)\n");
    // Project contract (INSTALL-014 / D-035): the toolchain pin and runtime
    // path defaults the global `aps` binary discovers by walking up from cwd.
    let _ = writeln!(
        out,
        "cli_version: {}",
        selections.cli_version.as_deref().unwrap_or(CLI_VERSION)
    );
    let _ = writeln!(out, "profile: {}", profile_key(selections.profile));
    let _ = writeln!(out, "shape: {}", shape_key(selections.shape));
    let _ = writeln!(
        out,
        "plans_dir: {}",
        normalize_dir(&selections.plans_dir, "plans")
    );
    let _ = writeln!(
        out,
        "docs_dir: {}",
        normalize_dir(&selections.docs_dir, "docs")
    );
    let _ = writeln!(
        out,
        "tooling_root: {}",
        normalize_dir(&selections.tooling_root, ".aps")
    );
    out.push_str("templates:\n");
    for template in &selections.templates {
        let _ = writeln!(out, "  - {}", template_key(*template));
    }
    if let Some(custom) = &selections.custom_template {
        let _ = writeln!(out, "custom_template: {custom}");
    }
    out.push_str("components:\n");
    for component in &selections.components {
        let _ = writeln!(out, "  - {}", component_key(*component));
    }
    out.push_str("tools:\n");
    for config in &selections.tools {
        let _ = writeln!(out, "  - name: {}", tool_key(config.tool));
        let _ = writeln!(out, "    agents: {}", config.install_agents);
        let _ = writeln!(out, "    hooks: {}", hooks_key(config.hooks));
        let _ = writeln!(out, "    model: {}", model_key(config.model));
    }
    out
}

pub fn profile_key(profile: Profile) -> &'static str {
    match profile {
        Profile::Solo => "solo",
        Profile::Team => "team",
        Profile::AgentOperator => "agent-operator",
    }
}

pub fn shape_key(shape: ProjectShape) -> &'static str {
    match shape {
        ProjectShape::SingleProject => "single",
        ProjectShape::Monorepo => "monorepo",
    }
}

pub fn template_key(template: Template) -> &'static str {
    match template {
        Template::Quickstart => "quickstart",
        Template::Module => "module",
        Template::Index => "index",
        Template::MonorepoIndex => "monorepo-index",
        Template::IndexNested => "index-nested",
    }
}

pub fn component_key(component: Component) -> &'static str {
    match component {
        Component::LintRules => "lint-rules",
        Component::ApsRules => "aps-rules",
        Component::ProjectContext => "project-context",
        Component::DesignsDir => "designs-dir",
        Component::DecisionsDir => "decisions-dir",
        Component::ReleasesDir => "releases-dir",
    }
}

pub fn tool_key(tool: AiTool) -> &'static str {
    match tool {
        AiTool::ClaudeCode => "claude-code",
        AiTool::Copilot => "copilot",
        AiTool::Codex => "codex",
        AiTool::OpenCode => "opencode",
        AiTool::Grok => "grok",
        AiTool::Antigravity => "antigravity",
        AiTool::Amp => "amp",
        AiTool::GeminiCli => "gemini-cli",
        AiTool::Windsurf => "windsurf",
        AiTool::RooCode => "roo-code",
        AiTool::OpenClaw => "openclaw",
        AiTool::Generic => "generic",
    }
}

pub fn hooks_key(hooks: HookVerbosity) -> &'static str {
    match hooks {
        HookVerbosity::Full => "full",
        HookVerbosity::Minimal => "minimal",
        HookVerbosity::None => "none",
    }
}

pub fn model_key(model: ModelPreference) -> &'static str {
    match model {
        ModelPreference::Default => "default",
        ModelPreference::Opus => "opus",
        ModelPreference::Sonnet => "sonnet",
    }
}

// --- Execution ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Done,
    Failed(String),
}

#[derive(Debug)]
pub struct ScaffoldRun {
    root: PathBuf,
    steps: Vec<ScaffoldStep>,
    statuses: Vec<StepStatus>,
    next: usize,
    verify_lint: bool,
}

impl ScaffoldRun {
    pub fn new(root: PathBuf, selections: &Selections) -> Self {
        let mut steps = plan_steps(selections);
        let verify_lint = selections.has_component(Component::LintRules);
        if verify_lint {
            // Native rule engine lands with TUI-009; until then this step
            // verifies the scaffold structurally.
            steps.push(ScaffoldStep {
                label: "Run lint (structural checks)".to_string(),
                ops: Vec::new(),
            });
        }
        let statuses = vec![StepStatus::Pending; steps.len()];
        Self {
            root,
            steps,
            statuses,
            next: 0,
            verify_lint,
        }
    }

    /// Execute a custom step list (setup flows) instead of the full init
    /// plan. No implicit verify step is appended.
    pub fn from_steps(root: PathBuf, steps: Vec<ScaffoldStep>) -> Self {
        let statuses = vec![StepStatus::Pending; steps.len()];
        Self {
            root,
            steps,
            statuses,
            next: 0,
            verify_lint: false,
        }
    }

    pub fn steps(&self) -> impl Iterator<Item = (&str, &StepStatus)> {
        self.steps
            .iter()
            .zip(&self.statuses)
            .map(|(step, status)| (step.label.as_str(), status))
    }

    pub fn finished(&self) -> bool {
        self.next >= self.steps.len()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.next, self.steps.len())
    }

    pub fn failures(&self) -> Vec<(String, String)> {
        self.steps
            .iter()
            .zip(&self.statuses)
            .filter_map(|(step, status)| match status {
                StepStatus::Failed(message) => Some((step.label.clone(), message.clone())),
                _ => None,
            })
            .collect()
    }

    /// Execute the next pending step. Returns false when nothing was left
    /// to do. Failures are recorded inline and do not halt the run.
    pub fn run_next(&mut self) -> bool {
        if self.finished() {
            return false;
        }

        let index = self.next;
        let is_verify = self.verify_lint && index == self.steps.len() - 1;
        let result = if is_verify {
            self.verify()
        } else {
            let step = self.steps[index].clone();
            self.apply_step(&step)
        };
        self.statuses[index] = match result {
            Ok(()) => StepStatus::Done,
            Err(err) => StepStatus::Failed(err.to_string()),
        };
        self.next += 1;
        true
    }

    fn apply_step(&self, step: &ScaffoldStep) -> io::Result<()> {
        for op in &step.ops {
            self.apply_op(op)?;
        }
        Ok(())
    }

    fn apply_op(&self, op: &FileOp) -> io::Result<()> {
        match op {
            FileOp::Mkdir(path) => fs::create_dir_all(self.root.join(path)),
            FileOp::Write { path, content } => self.write_new(path, content),
            FileOp::WriteOwned { path, content } => self.write_new(path, content),
            FileOp::CopyUser { from, to } => {
                let dest = self.root.join(to);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(from, dest).map(|_| ())
            }
            FileOp::MarkExecutable(path) => mark_executable(&self.root.join(path)),
            FileOp::Overwrite { path, content } => self.overwrite(path, content),
            FileOp::OverwriteOwned { path, content } => self.overwrite(path, content),
            FileOp::RemoveGenerated(path) => {
                let dest = self.root.join(path);
                if dest.exists() {
                    fs::remove_file(dest)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn write_new(&self, path: &Path, content: &str) -> io::Result<()> {
        let dest = self.root.join(path);
        if dest.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} already exists; refusing to overwrite", dest.display()),
            ));
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, content)
    }

    fn overwrite(&self, path: &Path, content: &str) -> io::Result<()> {
        let dest = self.root.join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, content)
    }

    /// Structural verification standing in for native lint until TUI-009.
    fn verify(&self) -> io::Result<()> {
        let mut missing = Vec::new();
        for step in &self.steps {
            for op in &step.ops {
                let path = match op {
                    FileOp::Mkdir(path) => path,
                    FileOp::Write { path, .. } | FileOp::WriteOwned { path, .. } => path,
                    FileOp::CopyUser { to, .. } => to,
                    FileOp::MarkExecutable(_) => continue,
                    FileOp::Overwrite { path, .. } | FileOp::OverwriteOwned { path, .. } => path,
                    FileOp::RemoveGenerated(_) => continue,
                };
                if !self.root.join(path).exists() {
                    missing.push(path.display().to_string());
                }
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "missing after scaffold: {}",
                missing.join(", ")
            )))
        }
    }
}

#[cfg(unix)]
pub(crate) fn mark_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
pub(crate) fn mark_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// Refuse to scaffold over an existing plans directory, matching the bash
/// installer's behavior.
pub fn check_target(root: &Path, selections: &Selections) -> Result<(), String> {
    let plans = root.join(selections.plans_path());
    if plans.exists() {
        Err(format!(
            "{} already exists — use the update script or remove it first",
            plans.display()
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_dir_rejects_unsafe_paths() {
        assert_eq!(normalize_dir("../evil", "plans"), "plans");
        assert_eq!(normalize_dir("/etc/aps", "plans"), "plans");
        assert_eq!(normalize_dir("nested/../../evil", "plans"), "plans");
        assert_eq!(normalize_dir("docs/plans/", "plans"), "docs/plans");
    }

    fn base_selections() -> Selections {
        Selections {
            profile: Profile::Solo,
            shape: ProjectShape::SingleProject,
            tools: Vec::new(),
            templates: vec![Template::Quickstart, Template::Index],
            custom_template: None,
            plans_dir: "plans/".to_string(),
            docs_dir: "docs/".to_string(),
            tooling_root: ".aps/".to_string(),
            components: vec![
                Component::LintRules,
                Component::ApsRules,
                Component::ProjectContext,
                Component::DesignsDir,
                Component::DecisionsDir,
                Component::ReleasesDir,
            ],
            cli_version: None,
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("aps-scaffold-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn run_all(root: &Path, selections: &Selections) -> ScaffoldRun {
        let mut run = ScaffoldRun::new(root.to_path_buf(), selections);
        while run.run_next() {}
        run
    }

    #[test]
    fn nested_template_scaffolds_federated_child_plans() {
        // MONO-005: the index-nested template writes a federation root plus
        // starter child plans with distinct, W020-clean work-item prefixes.
        let mut selections = base_selections();
        selections.templates = vec![Template::IndexNested];
        let root = temp_root("nested");
        run_all(&root, &selections);

        // Federation root uses the nested template.
        let root_index = fs::read_to_string(root.join("plans/index.aps.md")).unwrap();
        assert!(root_index.contains("## Child Plans"));

        for (pkg, prefix) in NESTED_CHILDREN {
            let child_index = root.join(format!("packages/{pkg}/plans/index.aps.md"));
            let child_module =
                root.join(format!("packages/{pkg}/plans/modules/module-name.aps.md"));
            assert!(child_index.is_file(), "{pkg} child index written");
            let module = fs::read_to_string(&child_module).unwrap();
            // Work-item IDs carry the package prefix, not the template's AUTH.
            assert!(
                module.contains(&format!("### {prefix}-001")),
                "{pkg} prefixed IDs"
            );
            assert!(
                !module.contains("### AUTH-001"),
                "{pkg} AUTH prefix replaced"
            );
        }

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn plan_includes_optional_dirs_only_when_selected() {
        let mut selections = base_selections();
        let steps = plan_steps(&selections);
        let dirs = &steps[0].ops;
        assert!(dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/decisions"))));
        assert!(dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/designs"))));
        assert!(dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/releases"))));

        selections.components = vec![Component::ApsRules];
        let steps = plan_steps(&selections);
        let dirs = &steps[0].ops;
        assert!(!dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/decisions"))));
        assert!(!dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/designs"))));
        assert!(!dirs.contains(&FileOp::Mkdir(PathBuf::from("plans/releases"))));
    }

    #[test]
    fn releases_component_writes_readme_and_template() {
        let root = temp_root("releases");
        let mut selections = base_selections();
        selections.components = vec![Component::ReleasesDir];
        selections.tools = Vec::new();

        let run = run_all(&root, &selections);

        assert!(run.failures().is_empty(), "failures: {:?}", run.failures());
        assert!(root.join("plans/releases").is_dir());
        // The README points users at the local template, which must exist.
        let readme = fs::read_to_string(root.join("plans/releases/README.md")).unwrap();
        assert!(readme.contains(".release.template.md"));
        assert!(root.join("plans/releases/.release.template.md").exists());

        // Without the component nothing release-related is written.
        let bare = temp_root("releases-off");
        let mut off = base_selections();
        off.components = Vec::new();
        off.tools = Vec::new();
        run_all(&bare, &off);
        assert!(!bare.join("plans/releases").exists());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&bare).unwrap();
    }

    #[test]
    fn codex_agents_are_standalone_discoverable_roles() {
        let files = agent_files(AiTool::Codex, ModelPreference::Default);

        assert_eq!(files.len(), 3);
        // Roles are auto-discovered, but the skill still needs a one-time
        // `codex skills install` — surfaced as the post-install note.
        assert!(
            post_install_note(AiTool::Codex)
                .is_some_and(|note| note.contains(".agents/skills/aps-planning"))
        );
        for (path, content) in &files {
            assert!(path.starts_with(".codex/agents/"), "unexpected {path}");
            assert!(content.lines().any(|line| line.starts_with("name = \"")));
            assert!(
                content
                    .lines()
                    .any(|line| line.starts_with("description = \""))
            );
            assert!(content.contains("developer_instructions = \"\"\""));
            assert!(content.contains("model = \"gpt-5.6-"));
        }
        let planner = files
            .iter()
            .find(|(p, _)| p.ends_with("aps-planner.toml"))
            .map(|(_, c)| c.as_str())
            .unwrap();
        assert!(planner.contains("model = \"gpt-5.6-sol\""));
        let librarian = files
            .iter()
            .find(|(p, _)| p.ends_with("aps-librarian.toml"))
            .map(|(_, c)| c.as_str())
            .unwrap();
        assert!(librarian.contains("model = \"gpt-5.6-terra\""));
    }

    #[test]
    fn model_preference_rewrites_claude_and_opencode_envelopes() {
        let claude_default = agent_files(AiTool::ClaudeCode, ModelPreference::Default);
        let planner = claude_default
            .iter()
            .find(|(p, _)| p.ends_with("aps-planner.md"))
            .unwrap();
        let librarian = claude_default
            .iter()
            .find(|(p, _)| p.ends_with("aps-librarian.md"))
            .unwrap();
        assert!(planner.1.contains("model: opus"));
        assert!(librarian.1.contains("model: sonnet"));
        assert!(planner.1.contains("# APS Planner") || planner.1.contains("APS Planner"));

        let all_sonnet = agent_files(AiTool::ClaudeCode, ModelPreference::Sonnet);
        for (_, content) in &all_sonnet {
            assert!(content.contains("model: sonnet"));
        }

        // OpenCode default: no model pin (user's provider config wins).
        for (_, content) in agent_files(AiTool::OpenCode, ModelPreference::Default) {
            assert!(
                !content.lines().any(|l| l.starts_with("model:")),
                "OpenCode default should omit model field:\n{content}"
            );
        }
        let oc = agent_files(AiTool::OpenCode, ModelPreference::Opus);
        for (_, content) in &oc {
            assert!(content.contains("model: openai/gpt-5.6-sol"));
        }
        let oc_bal = agent_files(AiTool::OpenCode, ModelPreference::Sonnet);
        for (_, content) in &oc_bal {
            assert!(content.contains("model: openai/gpt-5.6-terra"));
        }

        // Copilot has no model field.
        for (_, content) in agent_files(AiTool::Copilot, ModelPreference::Opus) {
            assert!(!content.contains("model:"));
        }

        assert!(
            resolve_model(AiTool::Grok, AgentRole::Planner, ModelPreference::Default).is_none()
        );
        assert!(agent_files(AiTool::Grok, ModelPreference::Default).is_empty());
    }

    #[test]
    fn hook_bundle_contains_shell_and_powershell_variants() {
        let paths = HOOK_SCRIPTS
            .iter()
            .map(|(path, _)| *path)
            .collect::<Vec<_>>();
        assert!(paths.contains(&".aps/scripts/install-hooks.sh"));
        assert!(paths.contains(&".aps/scripts/install-hooks.ps1"));
        assert_eq!(paths.iter().filter(|path| path.ends_with(".sh")).count(), 6);
        assert_eq!(
            paths.iter().filter(|path| path.ends_with(".ps1")).count(),
            6
        );
    }

    #[test]
    fn custom_paths_flow_into_planned_ops() {
        let mut selections = base_selections();
        selections.plans_dir = "docs/plans/".to_string();
        selections.tooling_root = ".tooling".to_string();

        let steps = plan_steps(&selections);
        let all_ops: Vec<_> = steps.iter().flat_map(|step| step.ops.clone()).collect();

        assert!(all_ops.contains(&FileOp::Mkdir(PathBuf::from("docs/plans"))));
        assert!(
            all_ops
                .iter()
                .any(|op| matches!(op, FileOp::WriteOwned { path, .. }
                    if path == &PathBuf::from(".tooling/config.yml")))
        );
    }

    #[test]
    fn monorepo_shape_is_authoritative_over_stale_plain_index_selection() {
        let mut selections = base_selections();
        selections.shape = ProjectShape::Monorepo;
        selections.templates = vec![Template::Index];

        let steps = plan_steps(&selections);
        let index_op = steps
            .iter()
            .flat_map(|step| &step.ops)
            .find_map(|op| match op {
                FileOp::Write { path, content } if path == &PathBuf::from("plans/index.aps.md") => {
                    Some(*content)
                }
                _ => None,
            })
            .expect("index.aps.md planned");

        assert_eq!(index_op, INDEX_MONOREPO_TEMPLATE);
    }

    #[test]
    fn scaffold_produces_expected_structure() {
        let root = temp_root("structure");
        let mut selections = base_selections();
        selections.tools = vec![ToolConfig {
            tool: AiTool::ClaudeCode,
            install_agents: true,
            hooks: HookVerbosity::Minimal,
            model: ModelPreference::Opus,
        }];

        let run = run_all(&root, &selections);

        assert!(run.finished());
        assert!(run.failures().is_empty(), "failures: {:?}", run.failures());
        for path in [
            "plans/index.aps.md",
            "plans/aps-rules.md",
            "plans/project-context.md",
            "plans/execution/.actions.template.md",
            "plans/modules/.quickstart.template.md",
            "plans/designs/.design.template.md",
            "plans/releases/README.md",
            "plans/releases/.release.template.md",
            ".claude/skills/aps-planning/SKILL.md",
            ".claude/skills/aps-planning/.aps-managed.json",
            ".claude/agents/aps-conductor.md",
            ".aps/scripts/install-hooks.sh",
            ".aps/config.yml",
        ] {
            assert!(root.join(path).exists(), "missing {path}");
        }
        assert!(root.join("plans/decisions").is_dir());
        // The v1 footprint must not come back (root skill, Claude commands).
        assert!(!root.join("aps-planning").exists());
        assert!(!root.join(".claude/commands").exists());
        // Claude Code alone does not install the Codex/Grok skill root.
        assert!(!root.join(".agents").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn deselected_components_are_not_written() {
        let root = temp_root("minimal");
        let mut selections = base_selections();
        selections.components = Vec::new();
        selections.tools = Vec::new();

        let run = run_all(&root, &selections);

        assert!(run.failures().is_empty());
        assert!(!root.join("plans/aps-rules.md").exists());
        assert!(!root.join("plans/project-context.md").exists());
        assert!(!root.join("plans/designs").exists());
        assert!(!root.join("plans/releases").exists());
        assert!(!root.join("aps-planning").exists());
        assert!(!root.join(".claude").exists());
        assert!(root.join("plans/index.aps.md").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn skill_step_targets_v2_roots_per_tool() {
        fn roots(step: &ScaffoldStep) -> (bool, bool) {
            let has = |dir: &str| {
                step.ops
                    .iter()
                    .any(|op| matches!(op, FileOp::Write { path, .. } if path.starts_with(dir)))
            };
            (has(CLAUDE_SKILL_DIR), has(AGENTS_SKILL_DIR))
        }
        let cfg = ToolConfig::default_for;

        // Claude Code, Copilot, and OpenCode share .claude/skills/.
        for tool in [AiTool::ClaudeCode, AiTool::Copilot, AiTool::OpenCode] {
            let step = skill_step(&[cfg(tool)]).expect("skill installed");
            assert_eq!(roots(&step), (true, false), "{tool:?}");
        }
        // Codex + the D-045 native-discovery harnesses discover .agents/skills/.
        for tool in [
            AiTool::Codex,
            AiTool::Grok,
            AiTool::Antigravity,
            AiTool::Amp,
            AiTool::GeminiCli,
            AiTool::Windsurf,
            AiTool::RooCode,
            AiTool::OpenClaw,
        ] {
            let step = skill_step(&[cfg(tool)]).expect("skill installed");
            assert_eq!(roots(&step), (false, true), "{tool:?}");
        }
        // Mixed selection installs both roots.
        let step = skill_step(&[cfg(AiTool::OpenCode), cfg(AiTool::Codex)]).unwrap();
        assert_eq!(roots(&step), (true, true));
        // Generic-only means no tool integration — no skill.
        assert!(skill_step(&[cfg(AiTool::Generic)]).is_none());
        // Tool-agnostic (setup all) defaults to the shared .claude/skills root.
        let step = skill_step(&[]).unwrap();
        assert_eq!(roots(&step), (true, false));
        // hooks.md is v1-only; the v2 skill is three content files + managed marker.
        assert_eq!(step.ops.len(), SKILL_FILES.len() + 1);
        assert!(step.ops.iter().any(|op| {
            matches!(op, FileOp::WriteOwned { path, .. } if path.ends_with(MANIFEST_NAME))
        }));
    }

    #[test]
    fn custom_template_is_copied_with_dot_prefix() {
        let root = temp_root("custom");
        let source = root.join("my-template.md");
        fs::write(&source, "# custom\n").unwrap();

        let mut selections = base_selections();
        selections.custom_template = Some(source.display().to_string());

        let run = run_all(&root, &selections);

        assert!(run.failures().is_empty(), "failures: {:?}", run.failures());
        assert_eq!(
            fs::read_to_string(root.join("plans/modules/.my-template.md")).unwrap(),
            "# custom\n"
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_custom_template_records_failure_and_continues() {
        let root = temp_root("badcustom");
        let mut selections = base_selections();
        selections.custom_template = Some("/nonexistent/template.md".to_string());

        let run = run_all(&root, &selections);

        let failures = run.failures();
        assert_eq!(failures[0].0, "Install templates");
        // The verify step flags the file the failed copy never produced.
        assert_eq!(failures[1].0, "Run lint (structural checks)");
        assert!(failures[1].1.contains("missing after scaffold"));
        // Later steps still ran.
        assert!(root.join(".aps/config.yml").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn existing_plans_dir_is_rejected() {
        let root = temp_root("existing");
        fs::create_dir_all(root.join("plans")).unwrap();

        let selections = base_selections();
        assert!(check_target(&root, &selections).is_err());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn config_yaml_round_trips_keys() {
        let mut selections = base_selections();
        selections.tools = vec![ToolConfig {
            tool: AiTool::Codex,
            install_agents: false,
            hooks: HookVerbosity::Full,
            model: ModelPreference::Sonnet,
        }];
        selections.custom_template = Some("tpl.md".to_string());

        let yaml = config_yaml(&selections);

        for needle in [
            "profile: solo",
            "shape: single",
            "plans_dir: plans",
            "  - quickstart",
            "custom_template: tpl.md",
            "  - name: codex",
            "    agents: false",
            "    hooks: full",
            "    model: sonnet",
        ] {
            assert!(yaml.contains(needle), "missing {needle:?} in:\n{yaml}");
        }
    }
}

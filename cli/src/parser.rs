//! Shared `.aps.md` document model (TUI-009, D-028/D-031).
//!
//! One parser serving `aps lint`, `aps next`, and future orchestration
//! commands. Behavior deliberately mirrors the bash implementation in
//! `lib/rules/common.sh` and `lib/orchestrate.sh` — including its quirks —
//! because the parity contract is "same output on the same input".

use std::fs;
use std::io;
use std::path::Path;

/// File classification by path, mirroring `get_file_type` in lib/lint.sh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Template,
    Index,
    Archive,
    Issues,
    Design,
    Actions,
    Module,
    Simple,
    Release,
    Unknown,
}

impl FileType {
    pub fn key(self) -> &'static str {
        match self {
            Self::Template => "template",
            Self::Index => "index",
            Self::Archive => "archive",
            Self::Issues => "issues",
            Self::Design => "design",
            Self::Actions => "actions",
            Self::Module => "module",
            Self::Simple => "simple",
            Self::Release => "release",
            Self::Unknown => "unknown",
        }
    }
}

/// True when `path` sits anywhere under a `releases/` directory.
pub fn in_releases_dir(path: &str) -> bool {
    path.contains("/releases/") || path.starts_with("releases/")
}

pub fn file_type(path: &str) -> FileType {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let dirname = match path.rfind('/') {
        Some(index) => &path[..index],
        None => ".",
    };

    if basename.starts_with('.') {
        FileType::Template
    } else if basename == "index.aps.md" {
        FileType::Index
    } else if basename == "completed.aps.md" {
        FileType::Archive
    } else if basename == "issues.md" {
        FileType::Issues
    } else if basename.ends_with(".design.md")
        && (path.contains("/designs/") || path.starts_with("designs/"))
    {
        FileType::Design
    } else if path.contains("/execution/") && basename.ends_with(".actions.md") {
        FileType::Actions
    } else if in_releases_dir(path) && basename.ends_with(".md") && basename != "README.md" {
        // Release narratives live in releases/ as `v<version>.md`. README.md
        // is the directory guide, and `.`-prefixed templates were already
        // classified above. Anything else here is a (possibly misnamed)
        // release file — R001 flags the naming.
        FileType::Release
    } else if dirname.ends_with("/modules") || dirname.contains("/modules/") {
        FileType::Module
    } else if basename.ends_with(".aps.md") {
        FileType::Simple
    } else {
        FileType::Unknown
    }
}

/// Recursively find APS files (`*.aps.md`, `*.actions.md`, `*.design.md`,
/// `issues.md`), excluding dotfiles, sorted by path — mirrors
/// `find_aps_files` (`find … | sort`).
pub fn find_aps_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            let path_str = path.to_string_lossy();
            if name.ends_with(".aps.md")
                || name.ends_with(".actions.md")
                || name.ends_with(".design.md")
                || name == "issues.md"
                // Release narratives (`v<version>.md`) live under releases/.
                // README.md is the dir guide; dotfiles are already skipped.
                || (in_releases_dir(&path_str) && name.ends_with(".md") && name != "README.md")
            {
                files.push(path_str.into_owned());
            }
        }
    }
    files.sort();
    files
}

/// A loaded plan document. Line numbers are 1-based throughout, matching
/// grep/awk output in the bash implementation.
#[derive(Debug)]
pub struct PlanFile {
    pub path: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkItemHeader {
    /// 1-based line number of the `### ID: title` header.
    pub line: usize,
    pub header: String,
}

impl PlanFile {
    pub fn load(path: &str) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Ok(Self {
            path: path.to_string(),
            lines: text.lines().map(str::to_string).collect(),
        })
    }

    /// Test helper — build a document from inline text.
    #[cfg(test)]
    pub fn from_text(path: &str, text: &str) -> Self {
        Self {
            path: path.to_string(),
            lines: text.lines().map(str::to_string).collect(),
        }
    }

    /// Exact-line section check (`has_section`).
    pub fn has_section(&self, section: &str) -> bool {
        self.lines.iter().any(|line| line == section)
    }

    /// 1-based line number of the section header.
    pub fn section_line(&self, section: &str) -> Option<usize> {
        self.lines
            .iter()
            .position(|line| line == section)
            .map(|index| index + 1)
    }

    /// Lines between the section header and the next `## ` heading
    /// (`get_section_content`).
    pub fn section_content(&self, section: &str) -> Vec<&str> {
        let Some(start) = self.lines.iter().position(|line| line == section) else {
            return Vec::new();
        };
        self.lines[start + 1..]
            .iter()
            .take_while(|line| !line.starts_with("## "))
            .map(String::as_str)
            .collect()
    }

    /// Non-empty content ignoring blanks and HTML comments
    /// (`section_has_content`).
    pub fn section_has_content(&self, section: &str) -> bool {
        self.section_content(section).iter().any(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                return false;
            }
            // `^[[:space:]]*<!--.*-->$` and `^<!--` filters.
            if trimmed.starts_with("<!--") && line.trim_end().ends_with("-->") {
                return false;
            }
            if line.starts_with("<!--") {
                return false;
            }
            !line.is_empty()
        })
    }

    /// `| ID |` table header within the first 20 lines
    /// (`has_metadata_table`).
    pub fn has_metadata_table(&self) -> bool {
        self.lines
            .iter()
            .take(20)
            .any(|line| is_id_header_row(line))
    }

    /// First cell of the first data row under the `| ID |` header
    /// (`get_module_id`).
    pub fn module_id(&self) -> Option<String> {
        let mut found = false;
        for line in &self.lines {
            if !found {
                if is_id_header_row(line) {
                    found = true;
                }
                continue;
            }
            // Skip the separator row (only -,:,| and spaces).
            if line.starts_with('|') && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
                continue;
            }
            if line.starts_with('|') {
                let id = line.split('|').nth(1).unwrap_or("").trim().to_string();
                return Some(id).filter(|id| !id.is_empty());
            }
        }
        None
    }

    /// Value of the Status column in the metadata table (`get_status`).
    pub fn status(&self) -> Option<String> {
        let mut status_col = None;
        for line in &self.lines {
            match status_col {
                None => {
                    if is_id_header_row(line) {
                        let col = line.split('|').position(|cell| cell.trim() == "Status")?;
                        status_col = Some(col);
                    }
                }
                Some(col) => {
                    if !line.starts_with('|') || is_id_header_row(line) {
                        continue;
                    }
                    // Skip separator rows: nothing left after removing |:- and spaces.
                    if line.chars().all(|c| matches!(c, '|' | ':' | '-' | ' ')) {
                        continue;
                    }
                    let value = line.split('|').nth(col).unwrap_or("").trim().to_string();
                    return Some(value);
                }
            }
        }
        None
    }

    /// Value of the `Type` column in the metadata table, if present
    /// (`get_module_type`). Vertical modules omit the column and return None.
    pub fn module_type(&self) -> Option<String> {
        let mut type_col = None;
        for line in &self.lines {
            match type_col {
                None => {
                    if is_id_header_row(line) {
                        type_col = Some(line.split('|').position(|cell| cell.trim() == "Type")?);
                    }
                }
                Some(col) => {
                    if !line.starts_with('|') || is_id_header_row(line) {
                        continue;
                    }
                    if line.chars().all(|c| matches!(c, '|' | ':' | '-' | ' ')) {
                        continue;
                    }
                    let value = line.split('|').nth(col).unwrap_or("").trim().to_string();
                    return Some(value).filter(|v| !v.is_empty());
                }
            }
        }
        None
    }

    /// True when the metadata table marks this as a conductor (crosscutting)
    /// module (`Type: Conductor`, case-insensitive).
    pub fn is_conductor(&self) -> bool {
        self.module_type()
            .is_some_and(|t| t.eq_ignore_ascii_case("Conductor"))
    }

    /// All `### PREFIX-NNN:` work item headers (`get_work_items`).
    pub fn work_items(&self) -> Vec<WorkItemHeader> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| is_work_item_header(line))
            .map(|(index, line)| WorkItemHeader {
                line: index + 1,
                header: line.trim_start().to_string(),
            })
            .collect()
    }

    /// A `## <section>` heading and its body up to (excluding) the next
    /// `## ` heading, header line included (`orch_emit_section`). Empty when
    /// the section is absent.
    pub fn emit_section(&self, section: &str) -> Vec<&str> {
        let header = format!("## {section}");
        let Some(start) = self.lines.iter().position(|line| *line == header) else {
            return Vec::new();
        };
        let mut out = vec![self.lines[start].as_str()];
        out.extend(
            self.lines[start + 1..]
                .iter()
                .take_while(|line| !line.starts_with("## "))
                .map(String::as_str),
        );
        out
    }

    /// Lines after a work item header until the next `## `/`### ` heading
    /// (`orch_item_content` and the E005 extraction).
    pub fn item_content(&self, header_line: usize) -> Vec<&str> {
        self.lines[header_line..]
            .iter()
            .take_while(|line| !line.starts_with("## ") && !line.starts_with("### "))
            .map(String::as_str)
            .collect()
    }
}

fn is_id_header_row(line: &str) -> bool {
    // `^\| *ID *\|`
    let Some(rest) = line.strip_prefix('|') else {
        return false;
    };
    let rest = rest.trim_start_matches(' ');
    let Some(rest) = rest.strip_prefix("ID") else {
        return false;
    };
    rest.trim_start_matches(' ').starts_with('|')
}

/// `^### [A-Za-z]+-[0-9]+:` (any indentation is NOT allowed — anchored).
pub fn is_work_item_header(line: &str) -> bool {
    parse_work_item_id(line).is_some()
}

/// Extract the `PREFIX-NNN` ID from a `### PREFIX-NNN: …` header.
pub fn parse_work_item_id(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("### ")?;
    let colon = rest.find(':')?;
    let id = &rest[..colon];
    let dash = id.find('-')?;
    let (prefix, digits) = (&id[..dash], &id[dash + 1..]);
    if !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
        && !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
    {
        Some(id)
    } else {
        None
    }
}

/// Title from a work item header, stripping any trailing
/// `<punct> Complete …` suffix (mirrors the orchestrate sed expression
/// `s/[[:space:]]+[^[:alnum:][:space:]]+[[:space:]]+Complete.*$//`).
pub fn work_item_title(header: &str) -> String {
    let Some(rest) = header.strip_prefix("### ") else {
        return header.to_string();
    };
    let title = match rest.find(':') {
        Some(colon) => rest[colon + 1..].trim_start(),
        None => rest,
    };

    // Find ` <non-alnum-run> Complete` and cut there.
    let chars: Vec<char> = title.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            let ws_start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let punct_start = i;
            while i < chars.len() && !chars[i].is_alphanumeric() && !chars[i].is_whitespace() {
                i += 1;
            }
            if i > punct_start && i < chars.len() && chars[i].is_whitespace() {
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if chars[j..]
                    .iter()
                    .collect::<String>()
                    .starts_with("Complete")
                {
                    return chars[..ws_start].iter().collect();
                }
            }
            continue;
        }
        i += 1;
    }
    title.to_string()
}

/// Field value with indented continuation lines
/// (`orch_field_value`): `- **Field:** value` plus following indented
/// lines (leading whitespace and `- ` stripped), newline-joined.
pub fn field_value(content: &[&str], field: &str) -> String {
    let prefix = format!("- **{field}:**");
    let mut out: Vec<String> = Vec::new();
    let mut found = false;

    for line in content {
        if !found {
            if let Some(rest) = line.strip_prefix(&prefix) {
                let rest = rest.trim_start();
                if !rest.is_empty() {
                    out.push(rest.to_string());
                }
                found = true;
            }
            continue;
        }
        // Continuation: indented non-blank line.
        if line.starts_with(' ') || line.starts_with('\t') {
            let trimmed = line.trim_start();
            if !trimmed.is_empty() {
                out.push(trimmed.strip_prefix("- ").unwrap_or(trimmed).to_string());
                continue;
            }
        }
        break;
    }

    out.join("\n")
}

/// Normalize a status string (`orch_normalize_status`): strip leading
/// non-letters, then prefix-match the known states.
pub fn normalize_status(raw: &str, fallback: &str) -> String {
    if raw.is_empty() {
        return fallback.to_string();
    }
    let stripped: String = raw
        .chars()
        .skip_while(|c| !c.is_ascii_alphabetic())
        .collect();

    const ALIASES: &[(&str, &str)] = &[
        ("Complete", "Complete"),
        ("Done", "Complete"),
        ("In Progress", "In Progress"),
        ("Ready", "Ready"),
        ("Proposed", "Draft"),
        ("Draft", "Draft"),
        ("Blocked", "Blocked"),
    ];
    for (prefix, canonical) in ALIASES {
        if stripped.starts_with(prefix) {
            return canonical.to_string();
        }
    }
    "Unknown".to_string()
}

/// Extract dependency tokens (`grep -oE '[A-Z]+-[0-9]+|[A-Z]{2,}'`):
/// uppercase runs, optionally followed by `-digits` to form an item ID.
pub fn dep_tokens(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let run_end = i;
            // Try `-digits` suffix for an item ID.
            if i < chars.len() && chars[i] == '-' {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if j > i + 1 {
                    tokens.push(chars[start..j].iter().collect());
                    i = j;
                    continue;
                }
            }
            if run_end - start >= 2 {
                tokens.push(chars[start..run_end].iter().collect());
            }
            continue;
        }
        i += 1;
    }

    tokens
}

/// True when a dependency token names a work item (`^[A-Z]+-[0-9]+$`).
pub fn is_item_token(token: &str) -> bool {
    let Some(dash) = token.find('-') else {
        return false;
    };
    let (prefix, digits) = (&token[..dash], &token[dash + 1..]);
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
}

/// Module table rows from an index file (`orch_load_index_modules`):
/// lines `| [name](…) | … | status |` → (NAME uppercased, raw status).
pub fn index_modules(plan: &PlanFile) -> Vec<(String, String)> {
    let mut modules = Vec::new();
    for line in &plan.lines {
        // `^\| *\[`
        let Some(rest) = line.strip_prefix('|') else {
            continue;
        };
        if !rest.trim_start_matches(' ').starts_with('[') {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() < 4 {
            continue;
        }
        let module = cells[1];
        let module = match (module.find('['), module.rfind(']')) {
            (Some(open), Some(close)) if close > open => &module[open + 1..close],
            _ => continue,
        };
        let status = cells[3].trim();
        if !module.is_empty() && !status.is_empty() {
            modules.push((module.to_uppercase(), status.to_string()));
        }
    }
    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_file_types_by_path() {
        assert_eq!(file_type("plans/index.aps.md"), FileType::Index);
        assert_eq!(file_type("plans/completed.aps.md"), FileType::Archive);
        assert_eq!(file_type("plans/issues.md"), FileType::Issues);
        assert_eq!(
            file_type("plans/designs/2026-01-01-x.design.md"),
            FileType::Design
        );
        assert_eq!(
            file_type("plans/execution/TUI-001.actions.md"),
            FileType::Actions
        );
        assert_eq!(file_type("plans/modules/auth.aps.md"), FileType::Module);
        assert_eq!(file_type("plans/feature.aps.md"), FileType::Simple);
        assert_eq!(
            file_type("plans/modules/.module.template.md"),
            FileType::Template
        );
        assert_eq!(file_type("README.md"), FileType::Unknown);
    }

    #[test]
    fn in_releases_dir_matches_both_anchors() {
        assert!(in_releases_dir("plans/releases/v0.3.0.md")); // nested
        assert!(in_releases_dir("releases/v0.3.0.md")); // repo-relative root
        assert!(!in_releases_dir("plans/modules/auth.aps.md"));
        assert!(!in_releases_dir("releases.md")); // not a dir segment
    }

    #[test]
    fn classifies_release_files() {
        // Versioned narratives are releases; so are misnamed siblings (so
        // R001 can flag them).
        assert_eq!(file_type("plans/releases/v0.3.0.md"), FileType::Release);
        assert_eq!(file_type("plans/releases/draft.md"), FileType::Release);
        // The directory guide and dotfile template are not release files.
        assert_eq!(file_type("plans/releases/README.md"), FileType::Unknown);
        assert_eq!(
            file_type("plans/releases/.release.template.md"),
            FileType::Template
        );
    }

    #[test]
    fn find_aps_files_includes_releases_excludes_readme() {
        let root = std::env::temp_dir().join(format!("aps-find-rel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let releases = root.join("releases");
        fs::create_dir_all(&releases).unwrap();
        fs::write(releases.join("v0.3.0.md"), "# r\n").unwrap();
        fs::write(releases.join("README.md"), "# guide\n").unwrap();
        fs::write(releases.join(".release.template.md"), "# t\n").unwrap();

        let found = find_aps_files(&root);
        assert!(found.iter().any(|f| f.ends_with("releases/v0.3.0.md")));
        assert!(!found.iter().any(|f| f.ends_with("README.md")));
        assert!(!found.iter().any(|f| f.contains(".release.template.md")));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn extracts_metadata_table_values() {
        let plan = PlanFile::from_text(
            "x.aps.md",
            "# Title\n\n| ID  | Owner | Status |\n| --- | ----- | ------ |\n| TUI | @me   | In Progress |\n",
        );

        assert!(plan.has_metadata_table());
        assert_eq!(plan.module_id().as_deref(), Some("TUI"));
        assert_eq!(plan.status().as_deref(), Some("In Progress"));
        assert_eq!(plan.module_type(), None);
        assert!(!plan.is_conductor());
    }

    #[test]
    fn detects_conductor_type_column() {
        // Type column sits after ID; ID and Status are still read correctly.
        let plan = PlanFile::from_text(
            "x.aps.md",
            "# Title\n\n| ID  | Type      | Owner | Status   |\n| --- | --------- | ----- | -------- |\n| REL | Conductor | @me   | Complete |\n",
        );

        assert_eq!(plan.module_id().as_deref(), Some("REL"));
        assert_eq!(plan.status().as_deref(), Some("Complete"));
        assert_eq!(plan.module_type().as_deref(), Some("Conductor"));
        assert!(plan.is_conductor());
    }

    #[test]
    fn finds_work_items_and_content() {
        let plan = PlanFile::from_text(
            "x.aps.md",
            "## Work Items\n\n### AUTH-001: Login\n\n- **Intent:** allow login\n- **Status:** Ready\n\n### AUTH-002: Logout\n",
        );

        let items = plan.work_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line, 3);
        assert_eq!(items[0].header, "### AUTH-001: Login");

        let content = plan.item_content(items[0].line);
        assert!(content.contains(&"- **Intent:** allow login"));
        assert!(!content.iter().any(|line| line.contains("AUTH-002")));
    }

    #[test]
    fn field_value_handles_continuations() {
        let content = vec![
            "- **Dependencies:** AUTH-001,",
            "  AUTH-002",
            "- **Status:** Ready",
        ];
        assert_eq!(field_value(&content, "Dependencies"), "AUTH-001,\nAUTH-002");
        assert_eq!(field_value(&content, "Status"), "Ready");
        assert_eq!(field_value(&content, "Missing"), "");
    }

    #[test]
    fn status_normalization_matches_bash() {
        assert_eq!(normalize_status("Complete 2026-04-26", "Ready"), "Complete");
        assert_eq!(normalize_status("Done 2026-04-26", "Ready"), "Complete");
        assert_eq!(normalize_status("— Complete", "Ready"), "Complete");
        assert_eq!(normalize_status("— Done", "Ready"), "Complete");
        assert_eq!(normalize_status("Proposed", "Ready"), "Draft");
        assert_eq!(normalize_status("In Progress", "Ready"), "In Progress");
        assert_eq!(normalize_status("", "Draft"), "Draft");
        assert_eq!(normalize_status("Shipped", "Ready"), "Unknown");
    }

    #[test]
    fn work_item_title_strips_complete_suffix() {
        assert_eq!(
            work_item_title("### TUI-001: Project setup — Complete 2026-04-26"),
            "Project setup"
        );
        assert_eq!(work_item_title("### TUI-002: Plain title"), "Plain title");
    }

    #[test]
    fn dep_token_extraction_matches_grep() {
        assert_eq!(
            dep_tokens("AUTH-001, AUTH-002, CORE-001"),
            vec!["AUTH-001", "AUTH-002", "CORE-001"]
        );
        assert_eq!(dep_tokens("INSTALL (In Progress)"), vec!["INSTALL"]);
        assert_eq!(dep_tokens("D-026, D-027"), vec!["D-026", "D-027"]);
        assert_eq!(dep_tokens("none"), Vec::<String>::new());
    }

    #[test]
    fn index_module_rows_parse() {
        let plan = PlanFile::from_text(
            "plans/index.aps.md",
            "| Module | Description | Status |\n| [auth](./modules/auth.aps.md) | Login | Ready |\n| [tui](./modules/tui.aps.md) | Wizard | In Progress |\n",
        );
        assert_eq!(
            index_modules(&plan),
            vec![
                ("AUTH".to_string(), "Ready".to_string()),
                ("TUI".to_string(), "In Progress".to_string()),
            ]
        );
    }
}

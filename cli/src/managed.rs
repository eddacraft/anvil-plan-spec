//! Managed skill freshness — sidecar markers so `aps update` can distinguish
//! APS-owned content from user edits.
//!
//! Phase 1 covers the planning skill only. Agent inventory is Phase 3; keep
//! agent reconcile on the existing string-equality path until then.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::scaffold::{CLI_VERSION, SKILL_FILES};

/// Sidecar filename written next to managed skill content.
pub const MANIFEST_NAME: &str = ".aps-managed.json";

const SCHEMA_VERSION: u32 = 1;
const KIND_SKILL: &str = "skill";
const SKILL_NAME: &str = "aps-planning";

/// Inventory sidecar for a managed skill install (camelCase JSON on disk).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillManifest {
    pub schema_version: u32,
    pub kind: String,
    pub name: String,
    pub cli_version: String,
    pub bundle_digest: String,
    pub files: BTreeMap<String, String>,
}

/// Freshness of an on-disk skill directory relative to this binary's embeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    /// Directory missing, or empty of skill files and marker.
    Absent,
    /// Marker present, content matches marker, marker matches expected.
    Fresh,
    /// Marker present, content matches marker, marker is older than expected.
    Stale,
    /// Marker present, but on-disk content no longer matches the marker hashes.
    Dirty,
    /// Skill files present without a managed marker.
    Unmanaged,
    /// Marker present but unreadable / invalid.
    Broken,
}

/// Outcome of a managed skill reconcile (used by `aps update`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileResult {
    Added,
    Updated,
    Unchanged,
    DirtySkipped,
    Adopted,
    UnmanagedSkipped,
    BrokenSkipped,
}

/// SHA-256 hex digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Bundle digest over a sorted name→hash map (stable across serialisations).
pub fn bundle_digest(files: &BTreeMap<String, String>) -> String {
    let mut material = String::new();
    for (name, hash) in files {
        material.push_str(name);
        material.push('=');
        material.push_str(hash);
        material.push('\n');
    }
    sha256_hex(material.as_bytes())
}

/// Expected skill manifest for this binary's embedded `SKILL_FILES`.
pub fn expected_skill_manifest() -> SkillManifest {
    let mut files = BTreeMap::new();
    for (name, content) in SKILL_FILES {
        files.insert(name.to_string(), sha256_hex(content.as_bytes()));
    }
    let digest = bundle_digest(&files);
    SkillManifest {
        schema_version: SCHEMA_VERSION,
        kind: KIND_SKILL.to_string(),
        name: SKILL_NAME.to_string(),
        cli_version: CLI_VERSION.to_string(),
        bundle_digest: digest,
        files,
    }
}

impl SkillManifest {
    /// Serialise to the on-disk camelCase JSON shape (trailing newline).
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"schemaVersion\": {},\n", self.schema_version));
        out.push_str(&format!("  \"kind\": \"{}\",\n", escape_json(&self.kind)));
        out.push_str(&format!("  \"name\": \"{}\",\n", escape_json(&self.name)));
        out.push_str(&format!(
            "  \"cliVersion\": \"{}\",\n",
            escape_json(&self.cli_version)
        ));
        out.push_str(&format!(
            "  \"bundleDigest\": \"{}\",\n",
            escape_json(&self.bundle_digest)
        ));
        out.push_str("  \"files\": {\n");
        let entries: Vec<_> = self.files.iter().collect();
        for (i, (name, hash)) in entries.iter().enumerate() {
            let comma = if i + 1 < entries.len() { "," } else { "" };
            out.push_str(&format!(
                "    \"{}\": \"{}\"{comma}\n",
                escape_json(name),
                escape_json(hash)
            ));
        }
        out.push_str("  }\n}\n");
        out
    }

    /// Parse a sidecar written by [`SkillManifest::to_json`] (or compatible).
    pub fn from_json(text: &str) -> Result<Self, String> {
        let schema_version = json_u32(text, "schemaVersion")?;
        let kind = json_string(text, "kind")?;
        let name = json_string(text, "name")?;
        let cli_version = json_string(text, "cliVersion")?;
        let digest = json_string(text, "bundleDigest")?;
        let files = json_string_map(text, "files")?;
        if schema_version != SCHEMA_VERSION {
            return Err(format!("unsupported schemaVersion {schema_version}"));
        }
        if kind != KIND_SKILL {
            return Err(format!("unsupported kind '{kind}'"));
        }
        Ok(Self {
            schema_version,
            kind,
            name,
            cli_version,
            bundle_digest: digest,
            files,
        })
    }
}

/// Write `.aps-managed.json` into `skill_dir`.
pub fn write_skill_marker(skill_dir: &Path, manifest: &SkillManifest) -> io::Result<()> {
    fs::create_dir_all(skill_dir)?;
    fs::write(skill_dir.join(MANIFEST_NAME), manifest.to_json())
}

/// Classify a skill directory against the expected embed inventory.
pub fn evaluate_skill_dir(skill_dir: &Path, expected: &SkillManifest) -> SkillState {
    if !skill_dir.is_dir() {
        return SkillState::Absent;
    }

    let marker_path = skill_dir.join(MANIFEST_NAME);
    if !marker_path.is_file() {
        let any_skill_file = SKILL_FILES
            .iter()
            .any(|(name, _)| skill_dir.join(name).is_file());
        return if any_skill_file {
            SkillState::Unmanaged
        } else {
            // Empty dir (or only non-skill files) — safe to install.
            SkillState::Absent
        };
    }

    let text = match fs::read_to_string(&marker_path) {
        Ok(t) => t,
        Err(_) => return SkillState::Broken,
    };
    let marker = match SkillManifest::from_json(&text) {
        Ok(m) => m,
        Err(_) => return SkillState::Broken,
    };

    // Dirty when any tracked file is missing or no longer matches the marker.
    for (name, want_hash) in &marker.files {
        let path = skill_dir.join(name);
        let Ok(bytes) = fs::read(&path) else {
            return SkillState::Dirty;
        };
        if sha256_hex(&bytes) != *want_hash {
            return SkillState::Dirty;
        }
    }

    if manifests_equivalent(&marker, expected) {
        SkillState::Fresh
    } else {
        SkillState::Stale
    }
}

/// Reconcile skill files under `skill_dir` with managed-marker safety.
///
/// - Absent → write files + marker (`Added`)
/// - Fresh → no-op (`Unchanged`)
/// - Stale → refresh marker; rewrite files only when content differs from embeds (`Updated`)
/// - Dirty → refuse (`DirtySkipped`)
/// - Unmanaged matching embeds → write marker only (`Adopted`)
/// - Unmanaged differing → refuse (`UnmanagedSkipped`)
/// - Broken → refuse (`BrokenSkipped`)
pub fn reconcile_managed_skill(
    skill_dir: &Path,
    files: &[(&str, &str)],
    expected: &SkillManifest,
) -> Result<ReconcileResult, String> {
    match evaluate_skill_dir(skill_dir, expected) {
        SkillState::Absent => {
            write_skill_files(skill_dir, files)?;
            write_skill_marker(skill_dir, expected).map_err(|e| e.to_string())?;
            Ok(ReconcileResult::Added)
        }
        SkillState::Fresh => Ok(ReconcileResult::Unchanged),
        SkillState::Stale => {
            // Content may already match embeds (e.g. only cliVersion in the
            // marker drifted). Avoid needless file rewrites/mtime churn.
            if !skill_files_match(skill_dir, files) {
                write_skill_files(skill_dir, files)?;
            }
            write_skill_marker(skill_dir, expected).map_err(|e| e.to_string())?;
            Ok(ReconcileResult::Updated)
        }
        SkillState::Dirty => Ok(ReconcileResult::DirtySkipped),
        SkillState::Unmanaged => {
            if skill_files_match(skill_dir, files) {
                write_skill_marker(skill_dir, expected).map_err(|e| e.to_string())?;
                Ok(ReconcileResult::Adopted)
            } else {
                Ok(ReconcileResult::UnmanagedSkipped)
            }
        }
        SkillState::Broken => Ok(ReconcileResult::BrokenSkipped),
    }
}

fn write_skill_files(skill_dir: &Path, files: &[(&str, &str)]) -> Result<(), String> {
    fs::create_dir_all(skill_dir).map_err(|e| e.to_string())?;
    for (name, content) in files {
        fs::write(skill_dir.join(name), content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn skill_files_match(skill_dir: &Path, files: &[(&str, &str)]) -> bool {
    files.iter().all(|(name, content)| {
        fs::read_to_string(skill_dir.join(name)).is_ok_and(|current| current == *content)
    })
}

fn manifests_equivalent(a: &SkillManifest, b: &SkillManifest) -> bool {
    a.schema_version == b.schema_version
        && a.kind == b.kind
        && a.name == b.name
        && a.cli_version == b.cli_version
        && a.bundle_digest == b.bundle_digest
        && a.files == b.files
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn json_u32(text: &str, key: &str) -> Result<u32, String> {
    let after = after_key(text, key)?;
    let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err(format!("non-numeric value for '{key}'"));
    }
    digits
        .parse()
        .map_err(|_| format!("invalid u32 for '{key}'"))
}

fn json_string(text: &str, key: &str) -> Result<String, String> {
    let after = after_key(text, key)?;
    parse_json_string(after).map(|(s, _)| s)
}

fn json_string_map(text: &str, key: &str) -> Result<BTreeMap<String, String>, String> {
    let after = after_key(text, key)?;
    let body = after
        .strip_prefix('{')
        .ok_or_else(|| format!("'{key}' is not an object"))?;
    let end = find_matching_brace(body).ok_or_else(|| format!("unclosed object for '{key}'"))?;
    let inner = &body[..end];

    let mut map = BTreeMap::new();
    let mut cursor = inner.trim_start();
    while !cursor.is_empty() {
        if cursor.starts_with(',') {
            cursor = cursor[1..].trim_start();
            if cursor.is_empty() {
                break;
            }
        }
        let (k, after_key_str) =
            parse_json_string(cursor).map_err(|e| format!("{key} entry key: {e}"))?;
        let after_colon = after_key_str
            .trim_start()
            .strip_prefix(':')
            .ok_or_else(|| format!("expected ':' after key in '{key}'"))?
            .trim_start();
        let (v, after_val) =
            parse_json_string(after_colon).map_err(|e| format!("{key}.{k}: {e}"))?;
        map.insert(k, v);
        cursor = after_val.trim_start();
    }
    Ok(map)
}

fn after_key<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    let needle = format!("\"{key}\"");
    let rest = text
        .split_once(&needle)
        .map(|(_, r)| r)
        .ok_or_else(|| format!("missing key '{key}'"))?;
    rest.trim_start()
        .strip_prefix(':')
        .map(str::trim_start)
        .ok_or_else(|| format!("malformed key '{key}'"))
}

fn find_matching_brace(body: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in body.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a JSON string literal at the start of `s`.
/// Returns `(value, remainder_after_closing_quote)`.
fn parse_json_string(s: &str) -> Result<(String, &str), String> {
    let s = s.trim_start();
    let s = s
        .strip_prefix('"')
        .ok_or_else(|| "expected string".to_string())?;
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => return Ok((out, &s[i + 1..])),
            '\\' => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, '/')) => out.push('/'),
                Some((_, 'n')) => out.push('\n'),
                Some((_, 'r')) => out.push('\r'),
                Some((_, 't')) => out.push('\t'),
                Some((_, 'u')) => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        let Some((_, c)) = chars.next() else {
                            return Err("truncated unicode escape".to_string());
                        };
                        hex.push(c);
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| "bad unicode escape".to_string())?;
                    out.push(char::from_u32(code).ok_or_else(|| "invalid unicode".to_string())?);
                }
                Some((_, other)) => return Err(format!("unknown escape \\{other}")),
                None => return Err("truncated escape".to_string()),
            },
            c => out.push(c),
        }
    }
    Err("unterminated string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aps-managed-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn install_expected(dir: &Path) {
        let expected = expected_skill_manifest();
        for (name, content) in SKILL_FILES {
            fs::write(dir.join(name), content).unwrap();
        }
        write_skill_marker(dir, &expected).unwrap();
    }

    #[test]
    fn expected_files_plus_marker_is_fresh() {
        let dir = scratch("fresh");
        install_expected(&dir);
        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Fresh);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::Unchanged
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn stale_marker_with_matching_content_refreshes_marker_only() {
        let dir = scratch("stale");
        install_expected(&dir);
        // Rewrite marker to an older cliVersion / digest while keeping file hashes.
        let mut stale = expected_skill_manifest();
        stale.cli_version = "0.0.1".to_string();
        stale.bundle_digest = "0".repeat(64);
        write_skill_marker(&dir, &stale).unwrap();

        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Stale);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::Updated
        );
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Fresh);
        // On-disk skill content already matched embeds — still intact.
        let skill = fs::read_to_string(dir.join("SKILL.md")).unwrap();
        assert_eq!(skill, SKILL_FILES[0].1);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn stale_marker_with_outdated_content_rewrites_files() {
        let dir = scratch("stale-content");
        // Old content + marker that tracks it (so evaluate returns Stale, not Dirty).
        let old = [
            ("SKILL.md", "old skill\n"),
            ("reference.md", "old ref\n"),
            ("examples.md", "old examples\n"),
        ];
        let mut files = BTreeMap::new();
        for (name, content) in &old {
            fs::write(dir.join(name), content).unwrap();
            files.insert((*name).to_string(), sha256_hex(content.as_bytes()));
        }
        let stale = SkillManifest {
            schema_version: SCHEMA_VERSION,
            kind: KIND_SKILL.to_string(),
            name: SKILL_NAME.to_string(),
            cli_version: "0.0.1".to_string(),
            bundle_digest: bundle_digest(&files),
            files,
        };
        write_skill_marker(&dir, &stale).unwrap();

        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Stale);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::Updated
        );
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Fresh);
        assert_eq!(
            fs::read_to_string(dir.join("SKILL.md")).unwrap(),
            SKILL_FILES[0].1
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn user_edit_with_valid_marker_is_dirty_and_skipped() {
        let dir = scratch("dirty");
        install_expected(&dir);
        fs::write(dir.join("SKILL.md"), "# user edit\n").unwrap();

        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Dirty);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::DirtySkipped
        );
        // User content preserved.
        assert_eq!(
            fs::read_to_string(dir.join("SKILL.md")).unwrap(),
            "# user edit\n"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn matching_embeds_without_marker_are_adopted() {
        let dir = scratch("adopt");
        for (name, content) in SKILL_FILES {
            fs::write(dir.join(name), content).unwrap();
        }
        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Unmanaged);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::Adopted
        );
        assert!(dir.join(MANIFEST_NAME).is_file());
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Fresh);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn differing_content_without_marker_is_unmanaged_skip() {
        let dir = scratch("unmanaged");
        fs::write(dir.join("SKILL.md"), "custom skill\n").unwrap();
        fs::write(dir.join("reference.md"), "custom ref\n").unwrap();
        fs::write(dir.join("examples.md"), "custom examples\n").unwrap();

        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Unmanaged);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::UnmanagedSkipped
        );
        assert!(!dir.join(MANIFEST_NAME).exists());
        assert_eq!(
            fs::read_to_string(dir.join("SKILL.md")).unwrap(),
            "custom skill\n"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn absent_dir_is_added_with_marker() {
        let parent = scratch("absent-parent");
        let dir = parent.join("aps-planning");
        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Absent);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::Added
        );
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Fresh);
        fs::remove_dir_all(&parent).ok();
    }

    #[test]
    fn manifest_round_trips_json() {
        let expected = expected_skill_manifest();
        let json = expected.to_json();
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"kind\": \"skill\""));
        assert!(json.contains("\"name\": \"aps-planning\""));
        let parsed = SkillManifest::from_json(&json).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn broken_marker_is_skipped() {
        let dir = scratch("broken");
        for (name, content) in SKILL_FILES {
            fs::write(dir.join(name), content).unwrap();
        }
        fs::write(dir.join(MANIFEST_NAME), "{not json\n").unwrap();
        let expected = expected_skill_manifest();
        assert_eq!(evaluate_skill_dir(&dir, &expected), SkillState::Broken);
        assert_eq!(
            reconcile_managed_skill(&dir, &SKILL_FILES, &expected).unwrap(),
            ReconcileResult::BrokenSkipped
        );
        fs::remove_dir_all(&dir).ok();
    }
}

//! `aps export` — machine-readable snapshot of a plan tree (INTEGRATIONS-002).
//!
//! Compact JSON (schema `aps-export/v1`) on stdout: modules that define work
//! items in file order, each with its items in document order. Reuses the
//! `next`/`graph` loader so statuses, titles, children, and effective
//! `Packages:` tags match the orchestration commands exactly. Byte-identical
//! to the bash CLI's `aps export` (D-039); running it twice byte-matches.

use std::path::Path;

use crate::next::PlanGraph;
use crate::parser::PlanFile;

/// Escape a string for a JSON value: backslash, quote, and the control
/// characters a plan file can realistically contain (`export_json_escape`).
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// `"value"` when non-empty, `null` otherwise (`export_string_or_null`).
fn string_or_null(v: &str) -> String {
    if v.is_empty() {
        "null".to_string()
    } else {
        format!("\"{}\"", esc(v))
    }
}

/// A raw Dependencies field -> JSON array elements (`export_deps_elements`).
fn deps_elements(deps: &str) -> String {
    deps.replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|tok| !tok.is_empty())
        .map(|tok| format!("\"{}\"", esc(tok)))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn cmd_export(plan_root: &str) -> i32 {
    // Items-only load, matching the bash cmd_export (module statuses come
    // purely from each module file, like cmd_audit).
    let graph = match PlanGraph::load_items_only(Path::new(plan_root)) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };

    let mut json = format!(
        "{{\"schema\":\"aps-export/v1\",\"generated_by\":\"aps {}\",\"plans_dir\":\"{}\",\"modules\":[",
        env!("CARGO_PKG_VERSION"),
        esc(plan_root)
    );

    let mut cur_key: Option<String> = None;
    let mut first_module = true;
    let mut first_item = true;

    for item in &graph.items {
        let key = format!("{}|{}|{}", item.child, item.module, item.file);
        if cur_key.as_deref() != Some(key.as_str()) {
            if cur_key.is_some() {
                json.push_str("]}");
            }
            if !first_module {
                json.push(',');
            }
            first_module = false;
            cur_key = Some(key);
            first_item = true;

            let mstatus = graph.module_status(&item.module, &item.child).to_string();
            let plan = PlanFile::load(&item.file).ok();
            let mtype = plan
                .as_ref()
                .and_then(|p| p.module_type())
                .unwrap_or_default();
            let mpkgs = plan
                .as_ref()
                .and_then(|p| p.module_packages())
                .unwrap_or_default();

            json.push_str(&format!(
                "{{\"id\":\"{}\",\"child\":{},\"file\":\"{}\",\"status\":\"{}\",\"type\":{},\"packages\":{},\"work_items\":[",
                esc(&item.module),
                string_or_null(&item.child),
                esc(&item.file),
                esc(&mstatus),
                string_or_null(&mtype),
                string_or_null(&mpkgs),
            ));
        }

        if !first_item {
            json.push(',');
        }
        first_item = false;

        let pkgs = graph.item_packages(item);
        json.push_str(&format!(
            "{{\"id\":\"{}\",\"title\":\"{}\",\"status\":\"{}\",\"line\":{},\"dependencies\":[{}],\"packages\":{}}}",
            esc(&item.id),
            esc(&item.title),
            esc(&item.status),
            item.line,
            deps_elements(&item.deps),
            string_or_null(&pkgs),
        ));
    }

    if cur_key.is_some() {
        json.push_str("]}");
    }
    json.push_str("]}");

    println!("{json}");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_json_specials() {
        assert_eq!(esc(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(esc("x\ny"), "x\\ny");
    }

    #[test]
    fn deps_split_trims_and_drops_empties() {
        assert_eq!(
            deps_elements("AUTH-001, core:SSO-001,\n  D-040, "),
            "\"AUTH-001\",\"core:SSO-001\",\"D-040\""
        );
        assert_eq!(deps_elements(""), "");
    }

    #[test]
    fn string_or_null_maps_empty_to_null() {
        assert_eq!(string_or_null(""), "null");
        assert_eq!(string_or_null("core"), "\"core\"");
    }
}

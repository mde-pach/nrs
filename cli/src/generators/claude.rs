use crate::generators::Generator;
use crate::model::DirectoryContext;
use std::collections::BTreeMap;
use std::path::Path;

pub struct ClaudeGenerator;

const HEADER: &str = include_str!("../../templates/claude-header.txt");

const DENY_RULES: &[&str] = &[
    "Read(*.context.md)",
    "Edit(*.context.md)",
    "Write(*.context.md)",
];

impl Generator for ClaudeGenerator {
    fn name(&self) -> &str {
        "claude"
    }

    fn output_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn generate(&self, ctx: &DirectoryContext) -> String {
        let mut out = String::from(HEADER);
        out.push_str("\n\n");

        // Files are already sorted by layer priority in ContextSet.
        let sections: Vec<&str> = ctx
            .files
            .iter()
            .map(|f| f.content.trim())
            .filter(|s| !s.is_empty())
            .collect();

        out.push_str(&sections.join("\n\n"));
        out.push('\n');
        out
    }

    fn apply_permissions(&self, project_root: &Path) -> anyhow::Result<()> {
        let settings_dir = project_root.join(".claude");
        std::fs::create_dir_all(&settings_dir)?;

        let settings_path = settings_dir.join("settings.local.json");
        let mut settings: BTreeMap<String, serde_json::Value> = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            BTreeMap::new()
        };

        // Drop the legacy top-level ignorePatterns key (pre permissions.deny).
        settings.remove("ignorePatterns");

        let permissions = settings
            .entry("permissions".to_string())
            .or_insert_with(|| serde_json::json!({}));
        let permissions_obj = permissions
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("permissions must be an object"))?;

        // Drop legacy nested ignorePatterns — replaced by permissions.deny.
        permissions_obj.remove("ignorePatterns");

        let existing_deny = permissions_obj
            .get("deny")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut deny: Vec<String> = existing_deny
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        for rule in DENY_RULES {
            let s = rule.to_string();
            if !deny.contains(&s) {
                deny.push(s);
            }
        }

        permissions_obj.insert(
            "deny".to_string(),
            serde_json::Value::Array(deny.into_iter().map(serde_json::Value::String).collect()),
        );

        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&settings_path, content)?;

        let relative = settings_path
            .strip_prefix(project_root)
            .unwrap_or(&settings_path);
        println!("updated {}", relative.display());

        Ok(())
    }

    fn apply_hooks(&self, project_root: &Path) -> anyhow::Result<()> {
        let settings_dir = project_root.join(".claude");
        std::fs::create_dir_all(&settings_dir)?;

        let settings_path = settings_dir.join("settings.json");
        let mut settings: BTreeMap<String, serde_json::Value> = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            BTreeMap::new()
        };

        let hook_defs: Vec<(&str, serde_json::Value)> = vec![
            (
                "SubagentStop",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude observe --hook-mode"
                    }]
                }),
            ),
            (
                "TaskCompleted",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude notify --hook-mode"
                    }]
                }),
            ),
            (
                "PreToolUse",
                serde_json::json!({
                    "matcher": "Edit|Write",
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude guard --hook-mode"
                    }]
                }),
            ),
            (
                "FileChanged",
                serde_json::json!({
                    "matcher": "*.context.md",
                    "hooks": [{
                        "type": "command",
                        "command": "nrs generate claude && nrs validate"
                    }]
                }),
            ),
            (
                "SessionStart",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs gap summary && nrs validate"
                    }]
                }),
            ),
            (
                "SessionEnd",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude observe --hook-mode"
                    }]
                }),
            ),
            (
                "PreCompact",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude layers --hook-mode"
                    }]
                }),
            ),
            (
                "PostCompact",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude layers --hook-mode"
                    }]
                }),
            ),
            (
                "SubagentStart",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude layers --hook-mode"
                    }]
                }),
            ),
            (
                "Stop",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude observe --hook-mode"
                    }]
                }),
            ),
            (
                "StopFailure",
                serde_json::json!({
                    "hooks": [{
                        "type": "command",
                        "command": "nrs claude observe --hook-mode"
                    }]
                }),
            ),
        ];

        let hooks = settings
            .entry("hooks".to_string())
            .or_insert_with(|| serde_json::json!({}));
        let hooks_obj = hooks.as_object_mut().unwrap();

        let mut changed = false;
        for (event, entry) in &hook_defs {
            let command = entry["hooks"][0]["command"].as_str().unwrap();
            let arr = hooks_obj
                .entry(*event)
                .or_insert_with(|| serde_json::json!([]));
            let already_installed = arr
                .as_array()
                .unwrap()
                .iter()
                .any(|existing| {
                    existing
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|hooks| {
                            hooks.iter().any(|hook| {
                                hook.get("command")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c == command)
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                });
            if !already_installed {
                arr.as_array_mut().unwrap().push(entry.clone());
                changed = true;
            }
        }

        if changed {
            let content = serde_json::to_string_pretty(&settings)?;
            std::fs::write(&settings_path, content)?;

            let relative = settings_path
                .strip_prefix(project_root)
                .unwrap_or(&settings_path);
            println!("updated {}", relative.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::Generator;
    use crate::model::{ContextFile, ContextSet, DirectoryContext, Layer};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_ctx(files: Vec<(&str, &str)>) -> DirectoryContext {
        let mut ctx_files: Vec<ContextFile> = files
            .into_iter()
            .map(|(name, content)| ContextFile {
                relative_path: name.to_string(),
                filename: name.to_string(),
                layer: Layer::from_filename(name),
                content: content.to_string(),
            })
            .collect();
        ctx_files.sort_by_key(|f| f.layer.sort_priority());
        DirectoryContext {
            dir: PathBuf::from("/tmp/test"),
            relative_dir: String::new(),
            files: ctx_files,
        }
    }

    #[test]
    fn generate_includes_header() {
        let ctx = make_ctx(vec![("project.context.md", "# Project")]);
        let gen = ClaudeGenerator;
        let output = gen.generate(&ctx);
        assert!(output.starts_with("<!-- DO NOT EDIT — generated by NRS -->"));
    }

    #[test]
    fn generate_orders_nrs_first() {
        let ctx = make_ctx(vec![
            ("project.context.md", "# Project"),
            ("nrs.context.md", "# NRS"),
            ("corporate.context.md", "# Corp"),
        ]);
        let gen = ClaudeGenerator;
        let output = gen.generate(&ctx);

        let nrs_pos = output.find("# NRS").unwrap();
        let corp_pos = output.find("# Corp").unwrap();
        let project_pos = output.find("# Project").unwrap();

        assert!(nrs_pos < corp_pos);
        assert!(corp_pos < project_pos);
    }

    #[test]
    fn generate_orders_all_layers_correctly() {
        let ctx = make_ctx(vec![
            ("implementation.context.md", "# Impl"),
            ("domain.context.md", "# Domain"),
            ("team.context.md", "# Team"),
            ("corporate.context.md", "# Corp"),
            ("nrs.context.md", "# NRS"),
            ("project.context.md", "# Project"),
        ]);
        let gen = ClaudeGenerator;
        let output = gen.generate(&ctx);

        let positions: Vec<usize> = vec![
            output.find("# NRS").unwrap(),
            output.find("# Corp").unwrap(),
            output.find("# Team").unwrap(),
            output.find("# Project").unwrap(),
            output.find("# Domain").unwrap(),
            output.find("# Impl").unwrap(),
        ];

        for i in 1..positions.len() {
            assert!(
                positions[i - 1] < positions[i],
                "position {} should be before position {}",
                i - 1,
                i
            );
        }
    }

    #[test]
    fn generate_skips_empty_content() {
        let ctx = make_ctx(vec![
            ("project.context.md", "# Project"),
            ("corporate.context.md", ""),
        ]);
        let gen = ClaudeGenerator;
        let output = gen.generate(&ctx);
        assert!(!output.contains("\n\n\n\n"));
    }

    #[test]
    fn generate_empty_context() {
        let ctx = make_ctx(vec![]);
        let gen = ClaudeGenerator;
        let output = gen.generate(&ctx);
        assert!(output.starts_with("<!-- DO NOT EDIT"));
    }

    #[test]
    fn apply_permissions_creates_settings() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        gen.apply_permissions(tmp.path()).unwrap();

        let settings_path = tmp.path().join(".claude/settings.local.json");
        assert!(settings_path.exists());

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let deny = parsed["permissions"]["deny"].as_array().unwrap();
        assert!(deny.contains(&serde_json::Value::String("Read(*.context.md)".into())));
        assert!(deny.contains(&serde_json::Value::String("Edit(*.context.md)".into())));
        assert!(deny.contains(&serde_json::Value::String("Write(*.context.md)".into())));
    }

    #[test]
    fn apply_permissions_merges_with_existing_and_strips_legacy_ignores() {
        let tmp = TempDir::new().unwrap();
        let settings_dir = tmp.path().join(".claude");
        std::fs::create_dir_all(&settings_dir).unwrap();
        std::fs::write(
            settings_dir.join("settings.local.json"),
            r#"{"permissions":{"allow":["WebSearch"],"deny":["Bash(rm:*)"],"ignorePatterns":["old-nested"]},"ignorePatterns":["old-top"],"other":"preserved"}"#,
        )
        .unwrap();

        let gen = ClaudeGenerator;
        gen.apply_permissions(tmp.path()).unwrap();

        let content = std::fs::read_to_string(settings_dir.join("settings.local.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Existing allow preserved.
        let allow = parsed["permissions"]["allow"].as_array().unwrap();
        assert!(allow.contains(&serde_json::Value::String("WebSearch".into())));

        // Existing deny entry preserved + new rules appended.
        let deny = parsed["permissions"]["deny"].as_array().unwrap();
        assert!(deny.contains(&serde_json::Value::String("Bash(rm:*)".into())));
        assert!(deny.contains(&serde_json::Value::String("Read(*.context.md)".into())));
        assert!(deny.contains(&serde_json::Value::String("Edit(*.context.md)".into())));
        assert!(deny.contains(&serde_json::Value::String("Write(*.context.md)".into())));

        // Legacy ignorePatterns (both top-level and nested) removed.
        assert!(parsed.get("ignorePatterns").is_none());
        assert!(parsed["permissions"].get("ignorePatterns").is_none());

        // Unrelated top-level keys preserved.
        assert_eq!(parsed["other"], "preserved");
    }

    // ── Link rewriting via generate_and_rewrite ─────────────────────

    fn make_ctx_set(dirs: Vec<(&str, Vec<(&str, &str)>)>) -> ContextSet {
        let root = PathBuf::from("/tmp/test");
        let directories = dirs
            .into_iter()
            .map(|(rel_dir, files)| {
                let dir = if rel_dir.is_empty() {
                    root.clone()
                } else {
                    root.join(rel_dir)
                };
                let mut ctx_files: Vec<ContextFile> = files
                    .into_iter()
                    .map(|(name, content)| {
                        let relative_path = if rel_dir.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", rel_dir, name)
                        };
                        ContextFile {
                            relative_path,
                            filename: name.to_string(),
                            layer: Layer::from_filename(name),
                            content: content.to_string(),
                        }
                    })
                    .collect();
                ctx_files.sort_by_key(|f| f.layer.sort_priority());
                DirectoryContext {
                    dir,
                    relative_dir: rel_dir.to_string(),
                    files: ctx_files,
                }
            })
            .collect();
        ContextSet { root, directories }
    }

    #[test]
    fn generate_rewrites_same_dir_context_link() {
        let ctx_set = make_ctx_set(vec![(
            "",
            vec![
                ("domain.context.md", "# Domain Context\n\nSee [Impl](implementation.context.md)."),
                ("implementation.context.md", "# Implementation Context\n\nPatterns."),
            ],
        )]);
        let gen = ClaudeGenerator;
        let output = crate::generators::generate_and_rewrite(
            &gen,
            &ctx_set.directories[0],
            &ctx_set,
        );

        assert!(
            output.contains("[Impl](CLAUDE.md#implementation-context)"),
            "expected rewritten same-dir link, got: {output}"
        );
        assert!(
            !output.contains("implementation.context.md"),
            "original context link should be gone"
        );
    }

    #[test]
    fn generate_rewrites_cross_dir_context_link() {
        let ctx_set = make_ctx_set(vec![
            (
                "",
                vec![(
                    "project.context.md",
                    "# Project\n\n- [Orders](src/orders/domain.context.md)",
                )],
            ),
            (
                "src/orders",
                vec![("domain.context.md", "# Domain Context — Orders\n\nOrder rules.")],
            ),
        ]);
        let gen = ClaudeGenerator;
        let output = crate::generators::generate_and_rewrite(
            &gen,
            &ctx_set.directories[0],
            &ctx_set,
        );

        assert!(
            output.contains("[Orders](src/orders/CLAUDE.md#domain-context--orders)"),
            "expected rewritten cross-dir link, got: {output}"
        );
    }

    #[test]
    fn generate_leaves_docs_links_unchanged() {
        let ctx_set = make_ctx_set(vec![(
            "",
            vec![(
                "project.context.md",
                "# Project\n\n- [Testing](docs/testing.md)",
            )],
        )]);
        let gen = ClaudeGenerator;
        let output = crate::generators::generate_and_rewrite(
            &gen,
            &ctx_set.directories[0],
            &ctx_set,
        );

        assert!(
            output.contains("[Testing](docs/testing.md)"),
            "docs link should be unchanged"
        );
    }

    #[test]
    fn generate_leaves_external_urls_unchanged() {
        let ctx_set = make_ctx_set(vec![(
            "",
            vec![(
                "project.context.md",
                "# Project\n\n- [Ref](https://example.com/domain.context.md)",
            )],
        )]);
        let gen = ClaudeGenerator;
        let output = crate::generators::generate_and_rewrite(
            &gen,
            &ctx_set.directories[0],
            &ctx_set,
        );

        assert!(
            output.contains("https://example.com/domain.context.md"),
            "external URL should be unchanged"
        );
    }

    #[test]
    fn apply_hooks_creates_settings_with_all_hooks() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        gen.apply_hooks(tmp.path()).unwrap();

        let settings_path = tmp.path().join(".claude/settings.json");
        assert!(settings_path.exists());

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // SubagentStop → observe
        let subagent = parsed["hooks"]["SubagentStop"].as_array().unwrap();
        assert_eq!(subagent.len(), 1);
        assert_eq!(
            subagent[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude observe --hook-mode"
        );

        // TaskCompleted → notify
        let task = parsed["hooks"]["TaskCompleted"].as_array().unwrap();
        assert_eq!(task.len(), 1);
        assert_eq!(
            task[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude notify --hook-mode"
        );

        // PreToolUse → guard
        let pre = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1);
        assert_eq!(
            pre[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude guard --hook-mode"
        );
        assert_eq!(pre[0]["matcher"].as_str().unwrap(), "Edit|Write");

        // FileChanged → generate + validate
        let file_changed = parsed["hooks"]["FileChanged"].as_array().unwrap();
        assert_eq!(file_changed.len(), 1);
        assert_eq!(
            file_changed[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs generate claude && nrs validate"
        );
        assert_eq!(file_changed[0]["matcher"].as_str().unwrap(), "*.context.md");

        // SessionStart → gap summary + validate
        let session_start = parsed["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session_start.len(), 1);
        assert_eq!(
            session_start[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs gap summary && nrs validate"
        );

        // SessionEnd → observe
        let session_end = parsed["hooks"]["SessionEnd"].as_array().unwrap();
        assert_eq!(session_end.len(), 1);
        assert_eq!(
            session_end[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude observe --hook-mode"
        );

        // PreCompact → layers
        let pre_compact = parsed["hooks"]["PreCompact"].as_array().unwrap();
        assert_eq!(pre_compact.len(), 1);
        assert_eq!(
            pre_compact[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude layers --hook-mode"
        );

        // PostCompact → layers
        let post_compact = parsed["hooks"]["PostCompact"].as_array().unwrap();
        assert_eq!(post_compact.len(), 1);
        assert_eq!(
            post_compact[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude layers --hook-mode"
        );

        // SubagentStart → layers
        let subagent_start = parsed["hooks"]["SubagentStart"].as_array().unwrap();
        assert_eq!(subagent_start.len(), 1);
        assert_eq!(
            subagent_start[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude layers --hook-mode"
        );

        // Stop → observe
        let stop = parsed["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(
            stop[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude observe --hook-mode"
        );

        // StopFailure → observe
        let stop_failure = parsed["hooks"]["StopFailure"].as_array().unwrap();
        assert_eq!(stop_failure.len(), 1);
        assert_eq!(
            stop_failure[0]["hooks"][0]["command"].as_str().unwrap(),
            "nrs claude observe --hook-mode"
        );
    }

    #[test]
    fn apply_hooks_idempotent() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        gen.apply_hooks(tmp.path()).unwrap();
        gen.apply_hooks(tmp.path()).unwrap();

        let content = std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let subagent = parsed["hooks"]["SubagentStop"].as_array().unwrap();
        assert_eq!(subagent.len(), 1, "observe hook should not be duplicated");
        let task = parsed["hooks"]["TaskCompleted"].as_array().unwrap();
        assert_eq!(task.len(), 1, "notify hook should not be duplicated");
        let pre = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 1, "guard hook should not be duplicated");
        let file_changed = parsed["hooks"]["FileChanged"].as_array().unwrap();
        assert_eq!(file_changed.len(), 1, "FileChanged hook should not be duplicated");
        let session_start = parsed["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session_start.len(), 1, "SessionStart hook should not be duplicated");
        let session_end = parsed["hooks"]["SessionEnd"].as_array().unwrap();
        assert_eq!(session_end.len(), 1, "SessionEnd hook should not be duplicated");
        let pre_compact = parsed["hooks"]["PreCompact"].as_array().unwrap();
        assert_eq!(pre_compact.len(), 1, "PreCompact hook should not be duplicated");
        let post_compact = parsed["hooks"]["PostCompact"].as_array().unwrap();
        assert_eq!(post_compact.len(), 1, "PostCompact hook should not be duplicated");
        let subagent_start = parsed["hooks"]["SubagentStart"].as_array().unwrap();
        assert_eq!(subagent_start.len(), 1, "SubagentStart hook should not be duplicated");
        let stop = parsed["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1, "Stop hook should not be duplicated");
        let stop_failure = parsed["hooks"]["StopFailure"].as_array().unwrap();
        assert_eq!(stop_failure.len(), 1, "StopFailure hook should not be duplicated");
    }

    #[test]
    fn apply_hooks_merges_with_existing_hooks() {
        let tmp = TempDir::new().unwrap();
        let settings_dir = tmp.path().join(".claude");
        std::fs::create_dir_all(&settings_dir).unwrap();
        std::fs::write(
            settings_dir.join("settings.json"),
            r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"some-other-hook"}]}]}}"#,
        )
        .unwrap();

        let gen = ClaudeGenerator;
        gen.apply_hooks(tmp.path()).unwrap();

        let content = std::fs::read_to_string(settings_dir.join("settings.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        // Existing hook preserved + NRS guard hook added
        let pre = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre.len(), 2);
        assert_eq!(pre[0]["hooks"][0]["command"].as_str().unwrap(), "some-other-hook");
        assert_eq!(pre[1]["hooks"][0]["command"].as_str().unwrap(), "nrs claude guard --hook-mode");
        // Other hooks added
        assert_eq!(parsed["hooks"]["SubagentStop"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["hooks"]["TaskCompleted"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["hooks"]["FileChanged"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn apply_permissions_idempotent() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        gen.apply_permissions(tmp.path()).unwrap();
        gen.apply_permissions(tmp.path()).unwrap();

        let content =
            std::fs::read_to_string(tmp.path().join(".claude/settings.local.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let deny = parsed["permissions"]["deny"].as_array().unwrap();
        for rule in ["Read(*.context.md)", "Edit(*.context.md)", "Write(*.context.md)"] {
            let count = deny.iter().filter(|v| v.as_str() == Some(rule)).count();
            assert_eq!(count, 1, "{rule} should appear exactly once");
        }
    }
}

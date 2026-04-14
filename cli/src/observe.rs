use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::gaps::Gap;

// --- Transcript events (tool-agnostic abstraction) ---

#[derive(Debug)]
pub enum TranscriptEvent {
    FileRead { path: String },
    FileWrite { path: String },
    UserMessage { content: String },
    ToolFailure { tool: String },
}

// --- Transcript parsing ---

/// Parse a Claude Code transcript JSONL file into events.
pub fn parse_transcript(content: &str, project_root: &Path) -> Vec<TranscriptEvent> {
    let mut events = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        events.extend(extract_events(&value, project_root));
    }

    events
}

fn extract_events(value: &serde_json::Value, project_root: &Path) -> Vec<TranscriptEvent> {
    let mut events = Vec::new();

    // Handle assistant messages with tool_use content blocks
    if value.get("role").and_then(|v| v.as_str()) == Some("assistant") {
        if let Some(content) = value.get("content").and_then(|v| v.as_array()) {
            for block in content {
                if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                    let tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                    if let Some(event) = tool_use_to_event(tool_name, &input, project_root) {
                        events.push(event);
                    }
                }
            }
        }
    }

    // Handle user/human messages
    if value.get("role").and_then(|v| v.as_str()) == Some("user") {
        if let Some(text) = extract_user_text(value) {
            events.push(TranscriptEvent::UserMessage { content: text });
        }
    }

    // Handle tool_result with is_error
    if value.get("role").and_then(|v| v.as_str()) == Some("user") {
        if let Some(content) = value.get("content").and_then(|v| v.as_array()) {
            for block in content {
                if block.get("type").and_then(|v| v.as_str()) == Some("tool_result")
                    && block.get("is_error").and_then(|v| v.as_bool()) == Some(true)
                {
                    let tool_id = block
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    events.push(TranscriptEvent::ToolFailure {
                        tool: tool_id.to_string(),
                    });
                }
            }
        }
    }

    events
}

fn tool_use_to_event(
    tool_name: &str,
    input: &serde_json::Value,
    project_root: &Path,
) -> Option<TranscriptEvent> {
    match tool_name {
        "Read" | "View" => {
            let path = input.get("file_path").and_then(|v| v.as_str())?;
            Some(TranscriptEvent::FileRead {
                path: to_relative(path, project_root),
            })
        }
        "Write" => {
            let path = input.get("file_path").and_then(|v| v.as_str())?;
            Some(TranscriptEvent::FileWrite {
                path: to_relative(path, project_root),
            })
        }
        "Edit" => {
            let path = input.get("file_path").and_then(|v| v.as_str())?;
            Some(TranscriptEvent::FileWrite {
                path: to_relative(path, project_root),
            })
        }
        _ => None,
    }
}

fn extract_user_text(value: &serde_json::Value) -> Option<String> {
    // String content
    if let Some(text) = value.get("content").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    // Array content with text blocks
    if let Some(blocks) = value.get("content").and_then(|v| v.as_array()) {
        let texts: Vec<&str> = blocks
            .iter()
            .filter(|b| b.get("type").and_then(|v| v.as_str()) == Some("text"))
            .filter_map(|b| b.get("text").and_then(|v| v.as_str()))
            .collect();
        if !texts.is_empty() {
            return Some(texts.join(" "));
        }
    }
    None
}

fn to_relative(path: &str, project_root: &Path) -> String {
    let p = Path::new(path);
    if let Ok(rel) = p.strip_prefix(project_root) {
        rel.to_string_lossy().to_string()
    } else {
        path.to_string()
    }
}

fn parent_dir(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn is_context_file(path: &str) -> bool {
    path.ends_with(".context.md")
}

fn is_generated_file(path: &str) -> bool {
    let filename = Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    filename == "CLAUDE.md" || filename == "GEMINI.md"
}

fn is_source_file(path: &str) -> bool {
    !is_context_file(path) && !is_generated_file(path)
}

// --- Pattern detection ---

/// Detect all struggle patterns and return gaps.
pub fn detect_patterns(events: &[TranscriptEvent], project_root: &Path) -> Vec<Gap> {
    let mut gaps = Vec::new();
    gaps.extend(detect_excessive_reads(events, project_root));
    gaps.extend(detect_no_context(events, project_root));
    gaps.extend(detect_re_reads(events));
    gaps.extend(detect_backtracking(events));
    gaps.extend(detect_user_corrections(events));
    dedup_gaps(gaps)
}

/// excessive-reads: Agent reads 5+ distinct source files in a directory without writing any there.
fn detect_excessive_reads(events: &[TranscriptEvent], project_root: &Path) -> Vec<Gap> {
    let mut reads_by_dir: HashMap<String, HashSet<String>> = HashMap::new();
    let mut writes_by_dir: HashSet<String> = HashSet::new();

    for event in events {
        match event {
            TranscriptEvent::FileRead { path } if is_source_file(path) => {
                let dir = parent_dir(path);
                reads_by_dir.entry(dir).or_default().insert(path.clone());
            }
            TranscriptEvent::FileWrite { path } if is_source_file(path) => {
                writes_by_dir.insert(parent_dir(path));
            }
            _ => {}
        }
    }

    let mut gaps = Vec::new();
    for (dir, files) in &reads_by_dir {
        if writes_by_dir.contains(dir) {
            continue;
        }
        let count = files.len();
        if count < 5 {
            continue;
        }

        let has_context = dir_has_context(project_root, dir);
        let (gap_type, desc) = if has_context {
            (
                "missing-pattern",
                format!(
                    "agent read {} source files in {} without modifying any — context exists but may be insufficient",
                    count, dir
                ),
            )
        } else {
            (
                "missing-context",
                format!(
                    "agent read {} source files in {} without modifying any — no context file exists",
                    count, dir
                ),
            )
        };

        let confidence = if count >= 8 { "high" } else { "medium" };
        gaps.push(Gap::observed(gap_type, dir, &desc, "excessive-reads", confidence));
    }
    gaps
}

/// no-context: Agent performs 3+ file operations in a directory with no *.context.md.
fn detect_no_context(events: &[TranscriptEvent], project_root: &Path) -> Vec<Gap> {
    let mut ops_by_dir: HashMap<String, usize> = HashMap::new();

    for event in events {
        let path = match event {
            TranscriptEvent::FileRead { path } if is_source_file(path) => path,
            TranscriptEvent::FileWrite { path } if is_source_file(path) => path,
            _ => continue,
        };
        let dir = parent_dir(path);
        *ops_by_dir.entry(dir).or_default() += 1;
    }

    let mut gaps = Vec::new();
    for (dir, count) in &ops_by_dir {
        if *count < 3 {
            continue;
        }
        if dir_has_context(project_root, dir) {
            continue;
        }
        gaps.push(Gap::observed(
            "missing-context",
            dir,
            &format!("agent performed {} file operations in {} — no context file exists", count, dir),
            "no-context",
            "high",
        ));
    }
    gaps
}

/// re-reads: Same file read 3+ times in a session.
fn detect_re_reads(events: &[TranscriptEvent]) -> Vec<Gap> {
    let mut read_counts: HashMap<String, usize> = HashMap::new();

    for event in events {
        if let TranscriptEvent::FileRead { path } = event {
            if is_source_file(path) {
                *read_counts.entry(path.clone()).or_default() += 1;
            }
        }
    }

    let mut flagged_dirs: HashMap<String, Vec<String>> = HashMap::new();
    for (path, count) in &read_counts {
        if *count >= 3 {
            let dir = parent_dir(path);
            flagged_dirs
                .entry(dir)
                .or_default()
                .push(PathBuf::from(path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default());
        }
    }

    flagged_dirs
        .into_iter()
        .map(|(dir, files)| {
            Gap::observed(
                "missing-pattern",
                &dir,
                &format!(
                    "agent re-read files 3+ times ({}) — information may belong in context",
                    files.join(", ")
                ),
                "re-reads",
                "medium",
            )
        })
        .collect()
}

/// backtracking: Write → multiple reads in same dir → write same file again.
fn detect_backtracking(events: &[TranscriptEvent]) -> Vec<Gap> {
    let mut write_positions: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, event) in events.iter().enumerate() {
        if let TranscriptEvent::FileWrite { path } = event {
            if is_source_file(path) {
                write_positions.entry(path.clone()).or_default().push(i);
            }
        }
    }

    let mut flagged_dirs: HashSet<String> = HashSet::new();
    for (path, positions) in &write_positions {
        if positions.len() < 2 {
            continue;
        }
        let dir = parent_dir(path);

        // Check if there are reads between the first and last write
        for window in positions.windows(2) {
            let (first, last) = (window[0], window[1]);
            let reads_between = events[first + 1..last]
                .iter()
                .filter(|e| matches!(e, TranscriptEvent::FileRead { path: p } if parent_dir(p) == dir && is_source_file(p)))
                .count();
            if reads_between >= 2 {
                flagged_dirs.insert(dir.clone());
                break;
            }
        }
    }

    flagged_dirs
        .into_iter()
        .map(|dir| {
            Gap::observed(
                "missing-pattern",
                &dir,
                "agent wrote, then read multiple files, then rewrote — may have lacked sufficient context",
                "backtracking",
                "low",
            )
        })
        .collect()
}

/// user-correction: User correction markers followed by file writes.
fn detect_user_corrections(events: &[TranscriptEvent]) -> Vec<Gap> {
    const MARKERS: &[&str] = &[
        "no,", "no ", "wrong", "that's not", "thats not", "actually,", "actually ",
        "not what i", "not what I", "I said", "i said", "stop", "don't", "dont",
    ];

    let mut flagged_dirs: HashSet<String> = HashSet::new();

    for (i, event) in events.iter().enumerate() {
        if let TranscriptEvent::UserMessage { content } = event {
            let lower = content.to_lowercase();
            let is_correction = MARKERS.iter().any(|m| lower.contains(&m.to_lowercase()));
            if !is_correction {
                continue;
            }

            // Look at the next events for file writes (within 20 events)
            let lookahead = std::cmp::min(i + 20, events.len());
            for subsequent in &events[i + 1..lookahead] {
                if let TranscriptEvent::FileWrite { path } = subsequent {
                    if is_source_file(path) {
                        flagged_dirs.insert(parent_dir(path));
                    }
                }
                // Stop at next user message
                if matches!(subsequent, TranscriptEvent::UserMessage { .. }) {
                    break;
                }
            }
        }
    }

    flagged_dirs
        .into_iter()
        .map(|dir| {
            Gap::observed(
                "wrong",
                &dir,
                "user corrected agent behavior in this area — context may be inaccurate or missing",
                "user-correction",
                "low",
            )
        })
        .collect()
}

/// Check if a directory has any *.context.md files.
fn dir_has_context(project_root: &Path, relative_dir: &str) -> bool {
    let dir = project_root.join(relative_dir);
    if !dir.is_dir() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return false;
    };
    entries
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".context.md")
        })
}

/// Deduplicate gaps by (gap_type, target) — keep the first occurrence.
fn dedup_gaps(gaps: Vec<Gap>) -> Vec<Gap> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    gaps.into_iter()
        .filter(|g| seen.insert((g.gap_type.clone(), g.target.clone())))
        .collect()
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn make_transcript(entries: &[serde_json::Value]) -> String {
        entries
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn tool_use(name: &str, file_path: &str) -> serde_json::Value {
        serde_json::json!({
            "role": "assistant",
            "content": [{
                "type": "tool_use",
                "name": name,
                "input": { "file_path": file_path }
            }]
        })
    }

    fn user_msg(text: &str) -> serde_json::Value {
        serde_json::json!({
            "role": "user",
            "content": text
        })
    }

    #[test]
    fn parse_transcript_extracts_file_reads() {
        let transcript = make_transcript(&[
            tool_use("Read", "/project/src/foo.rs"),
            tool_use("Read", "/project/src/bar.rs"),
        ]);
        let events = parse_transcript(&transcript, Path::new("/project"));
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], TranscriptEvent::FileRead { path } if path == "src/foo.rs"));
        assert!(matches!(&events[1], TranscriptEvent::FileRead { path } if path == "src/bar.rs"));
    }

    #[test]
    fn parse_transcript_extracts_writes() {
        let transcript = make_transcript(&[
            tool_use("Write", "/project/src/new.rs"),
            tool_use("Edit", "/project/src/old.rs"),
        ]);
        let events = parse_transcript(&transcript, Path::new("/project"));
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], TranscriptEvent::FileWrite { path } if path == "src/new.rs"));
        assert!(matches!(&events[1], TranscriptEvent::FileWrite { path } if path == "src/old.rs"));
    }

    #[test]
    fn parse_transcript_extracts_user_messages() {
        let transcript = make_transcript(&[user_msg("fix the bug")]);
        let events = parse_transcript(&transcript, Path::new("/project"));
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], TranscriptEvent::UserMessage { content } if content == "fix the bug")
        );
    }

    #[test]
    fn parse_transcript_ignores_context_files_in_reads() {
        let transcript = make_transcript(&[
            tool_use("Read", "/project/src/domain.context.md"),
            tool_use("Read", "/project/src/foo.rs"),
        ]);
        let events = parse_transcript(&transcript, Path::new("/project"));
        // Both are parsed as events — filtering happens in pattern detection
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn excessive_reads_detects_5_reads_no_writes() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/billing");
        std::fs::create_dir_all(&src).unwrap();
        for i in 0..6 {
            std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
        }

        let events: Vec<TranscriptEvent> = (0..6)
            .map(|i| TranscriptEvent::FileRead {
                path: format!("src/billing/file{}.rs", i),
            })
            .collect();

        let gaps = detect_excessive_reads(&events, tmp.path());
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, "missing-context");
        assert_eq!(gaps[0].target, "src/billing");
        assert!(gaps[0].source.contains("excessive-reads"));
    }

    #[test]
    fn excessive_reads_skips_dirs_with_writes() {
        let events = vec![
            TranscriptEvent::FileRead { path: "src/billing/a.rs".into() },
            TranscriptEvent::FileRead { path: "src/billing/b.rs".into() },
            TranscriptEvent::FileRead { path: "src/billing/c.rs".into() },
            TranscriptEvent::FileRead { path: "src/billing/d.rs".into() },
            TranscriptEvent::FileRead { path: "src/billing/e.rs".into() },
            TranscriptEvent::FileWrite { path: "src/billing/a.rs".into() },
        ];

        let tmp = tempfile::TempDir::new().unwrap();
        let gaps = detect_excessive_reads(&events, tmp.path());
        assert!(gaps.is_empty(), "should skip directories where agent also wrote files");
    }

    #[test]
    fn excessive_reads_high_confidence_at_8() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/billing");
        std::fs::create_dir_all(&src).unwrap();
        for i in 0..9 {
            std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
        }

        let events: Vec<TranscriptEvent> = (0..9)
            .map(|i| TranscriptEvent::FileRead {
                path: format!("src/billing/file{}.rs", i),
            })
            .collect();

        let gaps = detect_excessive_reads(&events, tmp.path());
        assert_eq!(gaps[0].confidence, "high");
    }

    #[test]
    fn excessive_reads_returns_missing_pattern_when_context_exists() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/billing");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("domain.context.md"), "# Billing").unwrap();
        for i in 0..6 {
            std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
        }

        let events: Vec<TranscriptEvent> = (0..6)
            .map(|i| TranscriptEvent::FileRead {
                path: format!("src/billing/file{}.rs", i),
            })
            .collect();

        let gaps = detect_excessive_reads(&events, tmp.path());
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, "missing-pattern");
    }

    #[test]
    fn no_context_detects_3_ops_without_context() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/auth");
        std::fs::create_dir_all(&src).unwrap();

        let events = vec![
            TranscriptEvent::FileRead { path: "src/auth/login.rs".into() },
            TranscriptEvent::FileRead { path: "src/auth/session.rs".into() },
            TranscriptEvent::FileWrite { path: "src/auth/login.rs".into() },
        ];

        let gaps = detect_no_context(&events, tmp.path());
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, "missing-context");
        assert!(gaps[0].source.contains("no-context"));
    }

    #[test]
    fn no_context_skips_dirs_with_context() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/auth");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("domain.context.md"), "# Auth").unwrap();

        let events = vec![
            TranscriptEvent::FileRead { path: "src/auth/login.rs".into() },
            TranscriptEvent::FileRead { path: "src/auth/session.rs".into() },
            TranscriptEvent::FileWrite { path: "src/auth/login.rs".into() },
        ];

        let gaps = detect_no_context(&events, tmp.path());
        assert!(gaps.is_empty());
    }

    #[test]
    fn re_reads_detects_3_reads_of_same_file() {
        let events = vec![
            TranscriptEvent::FileRead { path: "src/core.rs".into() },
            TranscriptEvent::FileRead { path: "src/core.rs".into() },
            TranscriptEvent::FileRead { path: "src/core.rs".into() },
        ];

        let gaps = detect_re_reads(&events);
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].source.contains("re-reads"));
        assert_eq!(gaps[0].confidence, "medium");
    }

    #[test]
    fn re_reads_ignores_below_threshold() {
        let events = vec![
            TranscriptEvent::FileRead { path: "src/core.rs".into() },
            TranscriptEvent::FileRead { path: "src/core.rs".into() },
        ];

        let gaps = detect_re_reads(&events);
        assert!(gaps.is_empty());
    }

    #[test]
    fn backtracking_detects_write_reads_rewrite() {
        let events = vec![
            TranscriptEvent::FileWrite { path: "src/handler.rs".into() },
            TranscriptEvent::FileRead { path: "src/utils.rs".into() },
            TranscriptEvent::FileRead { path: "src/types.rs".into() },
            TranscriptEvent::FileWrite { path: "src/handler.rs".into() },
        ];

        let gaps = detect_backtracking(&events);
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].source.contains("backtracking"));
        assert_eq!(gaps[0].confidence, "low");
    }

    #[test]
    fn backtracking_ignores_single_write() {
        let events = vec![
            TranscriptEvent::FileWrite { path: "src/handler.rs".into() },
            TranscriptEvent::FileRead { path: "src/utils.rs".into() },
        ];

        let gaps = detect_backtracking(&events);
        assert!(gaps.is_empty());
    }

    #[test]
    fn user_correction_detects_no_followed_by_write() {
        let events = vec![
            TranscriptEvent::UserMessage { content: "no, that's not right".into() },
            TranscriptEvent::FileWrite { path: "src/billing/calc.rs".into() },
        ];

        let gaps = detect_user_corrections(&events);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, "wrong");
        assert!(gaps[0].source.contains("user-correction"));
    }

    #[test]
    fn user_correction_ignores_non_correction_messages() {
        let events = vec![
            TranscriptEvent::UserMessage { content: "looks good, now add tests".into() },
            TranscriptEvent::FileWrite { path: "src/billing/calc.rs".into() },
        ];

        let gaps = detect_user_corrections(&events);
        assert!(gaps.is_empty());
    }

    #[test]
    fn dedup_keeps_first_occurrence() {
        let gaps = vec![
            Gap::observed("missing-context", "src/billing", "first", "a", "high"),
            Gap::observed("missing-context", "src/billing", "second", "b", "low"),
        ];
        let result = dedup_gaps(gaps);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "first");
    }

    #[test]
    fn detect_patterns_combines_all_patterns() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src/billing");
        std::fs::create_dir_all(&src).unwrap();
        for i in 0..6 {
            std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
        }

        let mut events: Vec<TranscriptEvent> = (0..6)
            .map(|i| TranscriptEvent::FileRead {
                path: format!("src/billing/file{}.rs", i),
            })
            .collect();
        events.push(TranscriptEvent::UserMessage {
            content: "wrong approach".into(),
        });
        events.push(TranscriptEvent::FileWrite {
            path: "src/orders/fix.rs".into(),
        });

        let gaps = detect_patterns(&events, tmp.path());
        assert!(gaps.len() >= 2, "should detect multiple patterns");
    }
}

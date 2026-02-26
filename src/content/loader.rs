use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::Deserialize;

use crate::content::models::{Step, StepType};

const OBJECTIVE_ALIASES: &[&str] = &["objective", "mission"];
const RUN_ALIASES: &[&str] = &[
    "run",
    "deliverables",
    "required commands",
    "expected workflow",
    "drill rules",
    "pre-question checklist",
    "15-minute drill blocks",
    "required scenario mix",
];
const VERIFY_ALIASES: &[&str] = &[
    "verify",
    "verification",
    "validation",
    "success metric",
    "scoring",
    "passing rule in app",
];

#[derive(Debug, Deserialize, Default)]
struct StrictBlock {
    step_id: String,
    title: String,
    objective: String,
    #[serde(default)]
    ready_weight: Option<u32>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    success_check: Vec<String>,
    #[serde(default)]
    success_contains: Vec<String>,
    #[serde(default)]
    verify: Vec<String>,
    #[serde(default)]
    fallback_hint: Option<String>,
    #[serde(default)]
    what_changed: Vec<String>,
    #[serde(default)]
    optional: bool,
}

pub fn load_steps_from_root(root: &Path) -> Result<Vec<Step>> {
    let mut files = Vec::new();
    walk_markdown(root, &mut files)?;

    let mut steps = Vec::new();
    for path in files {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed reading {}", path.display()))?;
        steps.extend(parse_steps_from_doc(&path, &raw)?);
    }

    steps.sort_by(|a, b| a.id.cmp(&b.id));
    if steps.is_empty() {
        return Err(anyhow!("no runnable steps found under {}", root.display()));
    }
    Ok(steps)
}

fn walk_markdown(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_markdown(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

fn parse_steps_from_doc(path: &Path, raw: &str) -> Result<Vec<Step>> {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let title = lines
        .iter()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or("Untitled")
        .trim()
        .to_string();

    let (meta, _) = parse_metadata(&lines)?;
    let kind = match meta.get("type").map(|s| s.as_str()) {
        Some("project") => StepType::Project,
        Some("bug") => StepType::Bug,
        Some("exam") => StepType::Exam,
        _ => return Ok(Vec::new()),
    };

    let difficulty = meta
        .get("difficulty")
        .cloned()
        .unwrap_or_else(|| "n/a".to_string());
    let domains = meta
        .get("domains")
        .map(|d| {
            d.split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let timebox_min = meta
        .get("timebox_min")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let strict_blocks = parse_strict_blocks(raw)?;
    if !strict_blocks.is_empty() {
        let steps = strict_blocks
            .into_iter()
            .map(|b| Step {
                id: b.step_id,
                title: b.title,
                step_type: kind.clone(),
                domains: domains.clone(),
                difficulty: difficulty.clone(),
                timebox_min,
                objective: b.objective,
                run_items: b.commands.clone(),
                run_commands: b.commands,
                success_check_commands: b.success_check.clone(),
                success_contains: b.success_contains,
                verify_commands: if b.verify.is_empty() {
                    b.success_check
                } else {
                    b.verify
                },
                fallback_hint: b.fallback_hint,
                what_changed: b.what_changed,
                optional: b.optional,
                path: path.to_path_buf(),
                ready_weight: b.ready_weight.unwrap_or_else(|| weight_from_type(&meta)),
            })
            .collect::<Vec<_>>();
        return Ok(steps);
    }

    let id = meta
        .get("id")
        .cloned()
        .ok_or_else(|| anyhow!("missing id metadata in {}", path.display()))?;

    let sections = parse_sections(&lines);
    let objective =
        first_section_text(&sections, OBJECTIVE_ALIASES).unwrap_or_else(|| title.clone());
    let run_items = merged_section_items(&sections, RUN_ALIASES);
    let run_commands = extract_backtick_commands(&run_items.join("\n"));
    let verify_items = merged_section_items(&sections, VERIFY_ALIASES);
    let verify_commands = extract_backtick_commands(&verify_items.join("\n"));

    Ok(vec![Step {
        id,
        title,
        step_type: kind,
        domains,
        difficulty,
        timebox_min,
        objective,
        run_items,
        run_commands,
        success_check_commands: verify_commands.clone(),
        success_contains: Vec::new(),
        verify_commands,
        fallback_hint: None,
        what_changed: vec!["Verification checks passed".to_string()],
        optional: false,
        path: path.to_path_buf(),
        ready_weight: weight_from_type(&meta),
    }])
}

fn parse_strict_blocks(raw: &str) -> Result<Vec<StrictBlock>> {
    let re = Regex::new(r"(?s)```cka-step\n(.*?)\n```")?;
    let mut out = Vec::new();
    for caps in re.captures_iter(raw) {
        let body = caps
            .get(1)
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("invalid cka-step block"))?;
        let parsed: StrictBlock = serde_yaml::from_str(body)
            .with_context(|| "failed parsing cka-step yaml block".to_string())?;
        if !parsed.step_id.trim().is_empty() {
            out.push(parsed);
        }
    }
    Ok(out)
}

fn weight_from_type(meta: &HashMap<String, String>) -> u32 {
    if let Some(raw) = meta.get("ready_weight") {
        if let Ok(v) = raw.parse::<u32>() {
            return v.max(1);
        }
    }

    match meta.get("type").map(|s| s.as_str()) {
        Some("project") => 3,
        Some("bug") => 2,
        Some("exam") => 4,
        _ => 1,
    }
}

fn parse_metadata(lines: &[&str]) -> Result<(HashMap<String, String>, usize)> {
    let mut i = 0;
    while i < lines.len() && !lines[i].starts_with("# ") {
        i += 1;
    }
    if i < lines.len() {
        i += 1;
    }
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    let re = Regex::new(r"^([a-z_]+):\s*(.+)$")?;
    let mut map = HashMap::new();
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("##") || line.is_empty() {
            if line.starts_with("##") {
                break;
            }
            i += 1;
            continue;
        }

        if let Some(caps) = re.captures(line) {
            map.insert(caps[1].to_string(), caps[2].to_string());
        }
        i += 1;
    }
    Ok((map, i))
}

fn parse_sections(lines: &[&str]) -> HashMap<String, Vec<String>> {
    let mut sections: HashMap<String, Vec<String>> = HashMap::new();
    let mut current = String::new();

    for line in lines {
        let trimmed = line.trim();
        if let Some(header) = trimmed.strip_prefix("## ") {
            current = canonical_heading(header);
            sections.entry(current.clone()).or_default();
            continue;
        }

        if current.is_empty() {
            continue;
        }

        if let Some(item) = strip_list_prefix(trimmed) {
            sections
                .entry(current.clone())
                .or_default()
                .push(item.to_string());
        } else if !trimmed.is_empty() {
            sections
                .entry(current.clone())
                .or_default()
                .push(trimmed.to_string());
        }
    }

    sections
}

fn strip_list_prefix(line: &str) -> Option<&str> {
    if let Some(v) = line.strip_prefix("- ") {
        return Some(v.trim());
    }

    let mut digits = 0;
    for c in line.chars() {
        if c.is_ascii_digit() {
            digits += 1;
            continue;
        }
        break;
    }

    if digits > 0 {
        let rest = &line[digits..];
        if let Some(v) = rest.strip_prefix(") ") {
            return Some(v.trim());
        }
        if let Some(v) = rest.strip_prefix(". ") {
            return Some(v.trim());
        }
    }

    None
}

fn canonical_heading(h: &str) -> String {
    h.trim().to_ascii_lowercase()
}

fn first_section_text(sections: &HashMap<String, Vec<String>>, aliases: &[&str]) -> Option<String> {
    aliases
        .iter()
        .find_map(|key| sections.get(*key))
        .and_then(|items| items.first())
        .cloned()
}

fn merged_section_items(sections: &HashMap<String, Vec<String>>, aliases: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in aliases {
        if let Some(items) = sections.get(*key) {
            out.extend(items.iter().cloned());
        }
    }
    out
}

fn extract_backtick_commands(input: &str) -> Vec<String> {
    let Ok(re) = Regex::new(r"`([^`]+)`") else {
        return Vec::new();
    };

    re.captures_iter(input)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

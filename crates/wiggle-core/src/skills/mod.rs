//! SKILL.md skills engine: parse frontmatter-based skill packages,
//! match them to prompts semantically (keyword scoring), hot-reload,
//! and import from Claude Code / OpenCode / GitHub.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

pub mod importer;

/// Parsed frontmatter of a SKILL.md (agentskills.io spec).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub license: Option<String>,
    /// Compatible-host matrix per the spec.
    #[serde(default)]
    pub compatibility: Option<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// Absolute path of the SKILL.md file.
    pub path: PathBuf,
    /// Root directory of the skill package.
    pub root: PathBuf,
    /// Additional files bundled in the package (scripts, refs, ...).
    pub files: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Origin: bundled | user | project | imported
    pub origin: String,
}

fn default_true() -> bool {
    true
}

impl Skill {
    /// Full instructions body (everything below the frontmatter).
    pub fn instructions(&self) -> CoreResult<String> {
        let raw = std::fs::read_to_string(&self.path)?;
        Ok(split_frontmatter(&raw).1.trim().to_string())
    }
}

/// Split a SKILL.md into (frontmatter yaml, body).
pub fn split_frontmatter(raw: &str) -> (Option<String>, String) {
    let mut lines = raw.lines().peekable();
    if lines.peek().map(|l| l.trim()) == Some("---") {
        lines.next();
        let mut front = Vec::new();
        for line in lines.by_ref() {
            if line.trim() == "---" {
                return (Some(front.join("\n")), lines.collect::<Vec<_>>().join("\n"));
            }
            front.push(line);
        }
    }
    (None, raw.to_string())
}

/// Minimal YAML frontmatter parser (flat `key: value` + one nesting
/// level for metadata). Avoids a full YAML dependency for the two
/// fields the spec actually requires.
pub fn parse_frontmatter(raw: &str) -> CoreResult<SkillFrontmatter> {
    let (front, _) = split_frontmatter(raw);
    let front = front.ok_or_else(|| CoreError::Skill("missing YAML frontmatter".into()))?;

    let mut out = SkillFrontmatter::default();
    let mut current_list: Option<String> = None;
    for line in front.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim().trim_matches(['"', '\'']).to_string();
            match &current_list {
                Some(key) if key == "allowed-tools" || key == "allowed_tools" => {
                    out.allowed_tools.push(item);
                }
                Some(key) => {
                    out.metadata.entry(key.clone()).and_modify(|v| {
                        if !v.is_empty() {
                            v.push(' ');
                        }
                        v.push_str(&item);
                    });
                }
                None => {}
            }
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches(['"', '\'']);
            current_list = Some(key.to_string());
            match key {
                "name" => out.name = value.to_string(),
                "description" => out.description = value.to_string(),
                "license" => out.license = Some(value.to_string()),
                "compatibility" => out.compatibility = Some(value.to_string()),
                "allowed-tools" | "allowed_tools" => {
                    if !value.is_empty() {
                        out.allowed_tools =
                            value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
                other => {
                    out.metadata.insert(other.to_string(), value.to_string());
                }
            }
        }
    }
    if out.name.is_empty() {
        return Err(CoreError::Skill("frontmatter missing 'name'".into()));
    }
    if out.description.is_empty() {
        return Err(CoreError::Skill("frontmatter missing 'description'".into()));
    }
    Ok(out)
}

/// Load one skill package directory (must contain SKILL.md).
pub fn load_skill_dir(dir: &Path, origin: &str) -> CoreResult<Skill> {
    let path = dir.join("SKILL.md");
    if !path.exists() {
        return Err(CoreError::Skill(format!("{} has no SKILL.md", dir.display())));
    }
    let raw = std::fs::read_to_string(&path)?;
    let fm = parse_frontmatter(&raw)?;
    let mut files = Vec::new();
    collect_files(dir, &mut files, 0);
    files.retain(|f| f != "SKILL.md");
    Ok(Skill {
        name: fm.name,
        description: fm.description,
        path,
        root: dir.to_path_buf(),
        files,
        enabled: true,
        origin: origin.to_string(),
    })
}

fn collect_files(dir: &Path, out: &mut Vec<String>, depth: usize) {
    if depth > 6 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_files(&p, out, depth + 1);
        } else if let Ok(rel) = p.strip_prefix(dir) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// All installed skills across the three scopes + bundled library.
#[derive(Default)]
pub struct SkillStore {
    skills: Mutex<Vec<Skill>>,
}

impl SkillStore {
    pub fn load(
        home: &Path,
        workspace: &Path,
        bundled: Option<&Path>,
    ) -> CoreResult<SkillStore> {
        let mut skills = Vec::new();
        let scopes: Vec<(PathBuf, &str)> = bundled
            .map(|b| (b.to_path_buf(), "bundled"))
            .into_iter()
            .chain([(home.join("skills"), "user"), (workspace.join(".wiggle").join("skills"), "project")])
            .collect();
        for (root, origin) in scopes {
            for dir in skill_dirs(&root) {
                if let Ok(skill) = load_skill_dir(&dir, origin) {
                    skills.push(skill);
                }
            }
        }
        Ok(SkillStore { skills: Mutex::new(skills) })
    }

    pub fn all(&self) -> Vec<Skill> {
        self.skills.lock().expect("skills mutex").clone()
    }

    pub fn count(&self) -> usize {
        self.skills.lock().expect("skills mutex").len()
    }

    pub fn set_enabled(&self, name: &str, enabled: bool) -> CoreResult<()> {
        let mut skills = self.skills.lock().expect("skills mutex");
        let skill =
            skills.iter_mut().find(|s| s.name == name).ok_or_else(|| {
                CoreError::Skill(format!("skill '{name}' not installed"))
            })?;
        skill.enabled = enabled;
        Ok(())
    }

    pub fn install_from_dir(&self, dir: &Path) -> CoreResult<Skill> {
        let skill = load_skill_dir(dir, "imported")?;
        self.skills.lock().expect("skills mutex").push(skill.clone());
        Ok(skill)
    }

    /// Reload skills from disk (hot-reload after import or edit).
    pub fn reload(
        &mut self,
        home: &Path,
        workspace: &Path,
        bundled: Option<&Path>,
    ) -> CoreResult<()> {
        let disabled: Vec<String> =
            self.all().iter().filter(|s| !s.enabled).map(|s| s.name.clone()).collect();
        *self = Self::load(home, workspace, bundled)?;
        let mut skills = self.skills.lock().expect("skills mutex");
        for skill in skills.iter_mut() {
            if disabled.contains(&skill.name) {
                skill.enabled = false;
            }
        }
        Ok(())
    }

    /// Semantic match: score every enabled skill against the prompt
    /// using description/name keyword overlap and trigger phrases.
    pub fn match_prompts(&self, prompt: &str, limit: usize) -> Vec<Skill> {
        let needle = prompt.to_lowercase();
        let words: Vec<&str> =
            needle.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 2).collect();
        let mut scored: Vec<(i64, Skill)> = self
            .all()
            .into_iter()
            .filter(|s| s.enabled)
            .map(|s| {
                let mut score = 0i64;
                let hay = format!("{} {}", s.name.replace(['-', '_'], " "), s.description)
                    .to_lowercase();
                for word in &words {
                    if hay.contains(word) {
                        score += 1;
                    }
                }
                // Whole-phrase containment is a strong signal.
                if needle.contains(&s.name.replace(['-', '_'], " ")) {
                    score += 4;
                }
                (score, s)
            })
            .filter(|(score, _)| *score > 0)
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(limit);
        scored.into_iter().map(|(_, s)| s).collect()
    }
}

fn skill_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("SKILL.md").exists())
        .collect()
}

pub(crate) fn skill_dirs_pub(root: &Path) -> Vec<PathBuf> {
    skill_dirs(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_skill_frontmatter() {
        let raw = "---\nname: pdf\ndescription: Work with PDF files and forms.\nallowed-tools:\n  - Read\n  - Bash\nmetadata:\n  version: \"1.0\"\n---\n\n# PDF skill\n\nExtract text.";
        let fm = parse_frontmatter(raw).unwrap();
        assert_eq!(fm.name, "pdf");
        assert_eq!(fm.allowed_tools, vec!["Read", "Bash"]);
        assert_eq!(fm.metadata.get("version").map(String::as_str), Some("1.0"));
        let (front, body) = split_frontmatter(raw);
        assert!(front.is_some());
        assert!(body.contains("# PDF skill"));
    }

    #[test]
    fn rejects_missing_name() {
        assert!(parse_frontmatter("---\ndescription: only desc\n---\nbody").is_err());
    }

    #[test]
    fn semantic_matching_ranks_by_overlap() {
        let store = SkillStore::default();
        *store.skills.lock().unwrap() = vec![
            Skill {
                name: "pdf-processing".into(),
                description: "Extract text and tables from PDF documents".into(),
                path: Default::default(),
                root: Default::default(),
                files: vec![],
                enabled: true,
                origin: "bundled".into(),
            },
            Skill {
                name: "docker-deploy".into(),
                description: "Containerize and deploy services with docker compose".into(),
                path: Default::default(),
                root: Default::default(),
                files: vec![],
                enabled: true,
                origin: "bundled".into(),
            },
        ];
        let matches = store.match_prompts("please extract the pdf tables from report", 2);
        assert_eq!(matches.first().map(|s| s.name.clone()).as_deref(), Some("pdf-processing"));
        assert!(store.match_prompts("zzz qqq", 2).is_empty());
    }
}

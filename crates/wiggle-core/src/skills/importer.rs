//! 1-click skill importer: Claude Code (~/.claude/skills), OpenCode
//! (~/.config/opencode/skill) and GitHub repositories (anthropics/skills).

use std::path::{Path, PathBuf};

use crate::skills::{skill_dirs_pub, Skill};
use crate::{CoreError, CoreResult};

/// Known sources a user can import from with one click.
pub enum ImportSource {
    /// ~/.claude/skills (Claude Code)
    ClaudeCode,
    /// ~/.config/opencode/skill (OpenCode)
    OpenCode,
    /// Any GitHub repo containing <skill>/<SKILL.md> layout, e.g.
    /// anthropics/skills (fetched at the default branch, tarball).
    GitHub { owner: String, repo: String },
}

pub fn home_skills_dir(source: &ImportSource) -> Option<PathBuf> {
    let profile = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    match source {
        ImportSource::ClaudeCode => Some(profile.join(".claude").join("skills")),
        ImportSource::OpenCode => Some(
            profile
                .join(".config")
                .join("opencode")
                .join("skill"),
        ),
        ImportSource::GitHub { .. } => None,
    }
}

/// Import every skill package found in `from` into the Wiggle user
/// scope (`~/.wiggle/skills/`). Returns the imported skills.
pub fn import_dir(from: &Path) -> CoreResult<Vec<Skill>> {
    let target = crate::config::GlobalConfig::wiggle_home().join("skills");
    std::fs::create_dir_all(&target)?;
    let mut imported = Vec::new();
    for dir in skill_dirs_pub(from) {
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let dest = target.join(&name);
        if dest.exists() {
            continue; // never clobber
        }
        copy_dir(&dir, &dest)?;
        let skill = crate::skills::load_skill_dir(&dest, "imported")?;
        imported.push(skill);
    }
    Ok(imported)
}

/// Fetch and import a GitHub skill repo (tarball of the default branch).
pub async fn import_github(owner: &str, repo: &str) -> CoreResult<Vec<Skill>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/tarball");
    let response = reqwest::Client::new()
        .get(&url)
        .header("user-agent", "tervixa-wiggle")
        .header("accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| CoreError::Skill(format!("github fetch: {e}")))?
        .error_for_status()
        .map_err(|e| CoreError::Skill(format!("github fetch: {e}")))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| CoreError::Skill(format!("github read: {e}")))?;
    let gz = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(gz);

    let staging = std::env::temp_dir().join(format!("wiggle-import-{}-{}", owner, repo));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)?;
    archive
        .unpack(&staging)
        .map_err(|e| CoreError::Skill(format!("tar unpack: {e}")))?;

    // GitHub tarballs nest everything under <repo>-<sha>/.
    let root = std::fs::read_dir(&staging)?
        .flatten()
        .map(|e| e.path())
        .next()
        .ok_or_else(|| CoreError::Skill("empty tarball".into()))?;

    // If the repo root itself is one skill (SKILL.md at root), import as
    // single skill named after the repo; otherwise import each subdir.
    if root.join("SKILL.md").exists() {
        let target = crate::config::GlobalConfig::wiggle_home().join("skills").join(repo);
        if target.exists() {
            return Ok(Vec::new());
        }
        copy_dir(&root, &target)?;
        return Ok(vec![crate::skills::load_skill_dir(&target, "imported")?]);
    }

    let result = import_dir(&root)?;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(result)
}

fn copy_dir(from: &Path, to: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)?.flatten() {
        let p = entry.path();
        let dest = to.join(entry.file_name());
        if p.is_dir() {
            copy_dir(&p, &dest)?;
        } else {
            std::fs::copy(&p, &dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_dir_copies_skill_packages() {
        let tmp = std::env::temp_dir().join("wiggle-importer-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let skill_a = tmp.join("alpha");
        std::fs::create_dir_all(&skill_a).unwrap();
        std::fs::write(
            skill_a.join("SKILL.md"),
            "---\nname: alpha\ndescription: Test skill alpha.\n---\nBody.",
        )
        .unwrap();
        // home is shared with other runs; clear the target so the
        // never-clobber guard doesn't skip the copy
        let target = crate::config::GlobalConfig::wiggle_home().join("skills").join("alpha");
        let _ = std::fs::remove_dir_all(&target);
        let imported = import_dir(&tmp).unwrap();
        assert!(target.join("SKILL.md").exists(), "imported skill must exist on disk");
        assert_eq!(imported.len(), 1);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

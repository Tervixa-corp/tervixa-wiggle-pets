//! Pet registry: loads pet packages (pet.json + sprites) from assets/pets
//! and ~/.tervixa-wiggle-pets/pets, validates manifests, and downloads pets
//! from GitHub repositories / raw manifests.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    /// "gif" (animated gif file), "img" (static image) or "procedural"
    /// (canvas-drawn, params in `procedural`).
    pub kind: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub fps: Option<f32>,
    /// Procedural pets: named pose (cloud bobbing params, etc.)
    #[serde(default)]
    pub procedural: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetManifest {
    pub id: String,
    pub name: String,
    /// sprite | procedural
    pub kind: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    /// Logical size in px (at scale 1).
    #[serde(default)]
    pub size: Option<[f32; 2]>,
    /// Render scale multiplier.
    #[serde(default = "default_scale")]
    pub scale: f32,
    /// Where speech bubbles anchor (fraction of width/height).
    #[serde(default = "default_anchor")]
    pub bubble_anchor: [f32; 2],
    /// state name -> PetState
    pub states: std::collections::BTreeMap<String, PetState>,
    /// Default walking height offset from the bottom of the work area.
    #[serde(default = "default_ground")]
    pub ground_offset: f32,
}

fn default_scale() -> f32 {
    2.0
}
fn default_anchor() -> [f32; 2] {
    [0.5, 0.0]
}
fn default_ground() -> f32 {
    6.0
}

pub struct PetRegistry {
    pub dirs: Vec<PathBuf>,
    pub pets: Vec<PetManifest>,
}

pub fn builtin_assets_dir() -> Option<PathBuf> {
    // assets/pets lives next to the app; find it relative to the exe or cwd.
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets/pets"));
            candidates.push(dir.join("../../assets/pets"));
        }
    }
    candidates.push(PathBuf::from("assets/pets"));
    candidates.push(PathBuf::from("../assets/pets"));
    candidates.into_iter().find(|p| p.is_dir())
}

pub fn user_pets_dir() -> PathBuf {
    crate::app_dir().join("pets")
}

impl PetRegistry {
    pub fn load() -> Self {
        let mut dirs = Vec::new();
        if let Some(d) = builtin_assets_dir() {
            dirs.push(d);
        }
        let user = user_pets_dir();
        if user.is_dir() {
            dirs.push(user);
        }
        let mut pets = Vec::new();
        for dir in &dirs {
            pets.extend(load_dir(dir));
        }
        pets.sort_by(|a, b| a.id.cmp(&b.id));
        PetRegistry { dirs, pets }
    }

    pub fn get(&self, id: &str) -> Option<&PetManifest> {
        self.pets.iter().find(|p| p.id == id)
    }

    pub fn default_pet(&self) -> Option<&PetManifest> {
        self.get("cloud-nimbus").or_else(|| self.pets.first())
    }

    /// Resolve a state file to an absolute path.
    pub fn asset_path(&self, pet: &PetManifest, file: &str) -> Option<PathBuf> {
        for dir in &self.dirs {
            let p = dir.join(&pet.id).join(file);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
}

pub fn load_dir(dir: &Path) -> Vec<PetManifest> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let manifest = e.path().join("pet.json");
            if manifest.exists() {
                match std::fs::read_to_string(&manifest)
                    .map_err(|e| e.to_string())
                    .and_then(|s| serde_json::from_str::<PetManifest>(&s).map_err(|e| e.to_string()))
                {
                    Ok(p) => out.push(p),
                    Err(err) => eprintln!("[pet-registry] invalid {}: {err}", manifest.display()),
                }
            }
        }
    }
    out
}

/// Validate a manifest (used by tests and the importer).
pub fn validate(p: &PetManifest) -> Result<(), String> {
    if p.id.is_empty() || p.name.is_empty() {
        return Err("id and name are required".into());
    }
    if p.states.is_empty() {
        return Err("at least one state is required".into());
    }
    if !p.states.contains_key("idle") {
        return Err("an `idle` state is required".into());
    }
    for (name, st) in &p.states {
        match st.kind.as_str() {
            "gif" | "img" => {
                if st.file.is_none() {
                    return Err(format!("state {name}: file required for {kind}", kind = st.kind));
                }
            }
            "procedural" => {}
            other => return Err(format!("state {name}: unknown kind {other}")),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GitHub pet downloader
// ---------------------------------------------------------------------------

/// Fetch a pet package from a GitHub repo path (raw.githubusercontent).
/// `spec` formats: "user/repo/path/to/pet-folder" or a full raw URL.
pub fn download_pet(spec: &str) -> Result<PetManifest, String> {
    let (base, folder) = if let Some(rest) = spec.strip_prefix("https://") {
        // full URL: use as-is base
        (format!("https://{rest}"), None)
    } else if let Some((repo, sub)) = spec.split_once('/') {
        // user/repo/optional/sub/path
        let parts: Vec<&str> = spec.splitn(3, '/').collect();
        let sub = parts.get(2).copied().unwrap_or("pets");
        let branch = detect_default_branch(repo)?;
        (
            format!("https://raw.githubusercontent.com/{repo}/{branch}"),
            Some(sub.to_string()),
        )
    } else {
        return Err("spec must be user/repo[/subpath]".into());
    };

    // Try common layouts: <folder>/<name>/pet.json under given base.
    let candidates: Vec<String> = match (&base, folder) {
        (b, Some(f)) if b.contains("raw.githubusercontent.com") => vec![
            format!("{b}/{f}/pet.json"),
            format!("{b}/pets/{f}/pet.json"),
            format!("{b}/{f}/tervixa-pet/pet.json"),
        ],
        (b, _) => vec![format!("{b}/pet.json")],
    };
    for url in candidates {
        if let Ok(body) = http_get(&url) {
            let mut manifest: PetManifest = serde_json::from_str(&body).map_err(|e| e.to_string())?;
            validate(&manifest)?;
            manifest.source = Some(url.clone());
            // Fetch state assets alongside.
            let dir = user_pets_dir().join(&manifest.id);
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("pet.json"), serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            for st in manifest.states.values() {
                if let (Some(file), Some(base_url)) = (st.file.as_deref(), url.strip_suffix("/pet.json")) {
                    let asset_url = format!("{base_url}/{file}");
                    if let Ok(data) = http_get(&asset_url) {
                        let _ = std::fs::write(dir.join(file), data.as_bytes());
                    }
                }
            }
            return Ok(manifest);
        }
    }
    Err("pet.json not found in the repository (tried common layouts)".into())
}

fn detect_default_branch(repo: &str) -> Result<String, String> {
    match http_get(&format!("https://api.github.com/repos/{repo}")) {
        Ok(body) => {
            let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
            Ok(v["default_branch"].as_str().unwrap_or("main").to_string())
        }
        Err(_) => Ok("main".into()),
    }
}

fn http_get(url: &str) -> Result<String, String> {
    let mut req = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .set("User-Agent", "tervixa-wiggle-pets/1.0");
    req = req.set("Accept", "application/vnd.github+json");
    let resp = req.call().map_err(|e| e.to_string())?;
    resp.into_string().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_manifest() {
        let mut m = PetManifest {
            id: "test".into(),
            name: "Test".into(),
            kind: "sprite".into(),
            author: None,
            license: Some("MIT".into()),
            source: None,
            size: Some([32.0, 32.0]),
            scale: 2.0,
            bubble_anchor: [0.5, 0.0],
            states: Default::default(),
            ground_offset: 6.0,
        };
        assert!(validate(&m).is_err()); // no idle
        m.states.insert(
            "idle".into(),
            PetState { kind: "gif".into(), file: Some("idle.gif".into()), fps: Some(8.0), procedural: None },
        );
        assert!(validate(&m).is_ok());
    }
}

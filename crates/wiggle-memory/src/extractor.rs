//! Automatic per-turn user fact & preference extractor.
//!
//! Runs on every prompt + response pair. Uses deterministic pattern
//! rules first (zero cost, always available) and optionally a cheap
//! model call for the MemoryExtractor role when the user has bound one.

use regex::Regex;
use std::sync::OnceLock;

use crate::{MemoryResult, MemoryStore};

pub struct MemoryExtractor;

/// One candidate fact mined from a user message.
pub struct CandidateFact {
    pub kind: &'static str,
    pub content: String,
}

impl MemoryExtractor {
    /// Mine durable facts from one user prompt. Only clear, durable
    /// statements are kept — never one-off task chatter.
    pub fn extract_from_user_prompt(prompt: &str) -> Vec<CandidateFact> {
        let mut facts = Vec::new();
        let text = prompt.trim();
        if text.len() < 12 || text.len() > 2000 {
            return facts;
        }

        // "I prefer / I like / I use / my favorite / remember that ..."
        let preference = preference_re();
        for m in preference.captures_iter(text) {
            if let Some(stmt) = m.get(1) {
                facts.push(CandidateFact {
                    kind: "preference",
                    content: format!("User preference: {}", stmt.as_str().trim()),
                });
            }
        }

        // "remember that ..." / "note that ..." — explicit memory writes.
        let remember = remember_re();
        for m in remember.captures_iter(text) {
            if let Some(stmt) = m.get(1) {
                facts.push(CandidateFact {
                    kind: "user",
                    content: format!("User asked to remember: {}", stmt.as_str().trim()),
                });
            }
        }

        // Project declarations: "we use X" / "the project uses X".
        let project = project_re();
        for m in project.captures_iter(text) {
            if let Some(stmt) = m.get(1) {
                facts.push(CandidateFact {
                    kind: "project",
                    content: format!("Project fact: {}", stmt.as_str().trim()),
                });
            }
        }

        facts.truncate(4); // never flood the vault from one message
        facts
    }

    /// Mine durable outcomes from one assistant response (past fixes,
    /// architecture decisions worth remembering).
    pub fn extract_from_assistant_response(response: &str) -> Vec<CandidateFact> {
        let mut facts = Vec::new();
        for needle_marker in ["root cause was", "the fix was", "we decided", "the architecture is"] {
            for sentence in sentences(response) {
                let lower = sentence.to_lowercase();
                if lower.contains(needle_marker) && sentence.len() < 300 {
                    facts.push(CandidateFact {
                        kind: "conversation_summary",
                        content: sentence.trim().to_string(),
                    });
                    break;
                }
            }
            if facts.len() >= 2 {
                break;
            }
        }
        facts
    }

    /// Persist mined facts, deduplicating against the vault.
    pub fn learn(&self, store: &mut MemoryStore, user_prompt: &str, assistant: &str) -> MemoryResult<usize> {
        let mut stored = 0;
        let mut candidates = Self::extract_from_user_prompt(user_prompt);
        candidates.extend(Self::extract_from_assistant_response(assistant));
        for fact in candidates {
            // Skip near-duplicates already in the vault.
            let existing = store.query_relevant_context(&fact.content, 3)?;
            let duplicate = existing
                .iter()
                .any(|e| e.score > 1.6 && jaccard(&e.content, &fact.content) > 0.6);
            if !duplicate {
                store.remember(fact.kind, &fact.content, false)?;
                stored += 1;
            }
        }
        Ok(stored)
    }
}

fn sentences(text: &str) -> Vec<&str> {
    text.split(['.', '!', '?', '\n']).filter(|s| s.len() > 8).collect()
}

fn jaccard(a: &str, b: &str) -> f64 {
    let set_a: std::collections::HashSet<&str> =
        a.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).collect();
    let set_b: std::collections::HashSet<&str> =
        b.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).collect();
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

fn preference_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:i (?:prefer|like|always use|usually use|use)|my favorite(?: [a-z]+)? is|remember that|i am)\s*[:\-]?\s*(.{8,240}?)(?:[.;!\n]|$)",
        )
        .unwrap()
    })
}

fn remember_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:remember|note) that\s*[:\-]?\s*(.{8,240}?)(?:[.;!\n]|$)").unwrap()
    })
}

fn project_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:this project|the project|we|our team)\s+(?:uses?|built? with|runs? on)\s*[:\-]?\s*(.{8,240}?)(?:[.;!\n]|$)",
        )
        .unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mines_preferences() {
        let facts = MemoryExtractor::extract_from_user_prompt(
            "I prefer tabs over spaces in every project. Also remember that my CI is GitHub Actions.",
        );
        assert!(facts.iter().any(|f| f.kind == "preference"));
        assert!(facts.iter().any(|f| f.kind == "user"));
    }

    #[test]
    fn ignores_chatter() {
        assert!(MemoryExtractor::extract_from_user_prompt("fix the build please").is_empty());
        assert!(MemoryExtractor::extract_from_user_prompt("hi").is_empty());
    }

    #[test]
    fn mines_project_statements() {
        let facts = MemoryExtractor::extract_from_user_prompt(
            "The project uses Tauri v2 for the desktop shell",
        );
        assert!(facts.iter().any(|f| f.kind == "project" && f.content.contains("Tauri")));
    }

    #[tokio::test]
    async fn learn_dedupes_against_vault() {
        let dir = std::env::temp_dir().join(format!("wiggle-ext-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = crate::MemoryStore::open(&dir.join("wiggle.db")).unwrap();
        let extractor = MemoryExtractor;
        let n1 = extractor
            .learn(&mut store, "Remember that I prefer Rust for backend work", "ok")
            .unwrap();
        let n2 = extractor
            .learn(&mut store, "Remember that I prefer Rust for backend work", "ok")
            .unwrap();
        assert!(n1 >= 1);
        assert_eq!(n2, 0, "duplicate fact must not be stored twice");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

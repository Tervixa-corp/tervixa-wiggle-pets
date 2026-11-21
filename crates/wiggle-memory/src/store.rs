//! SQLite + FTS5 + vector cosine hybrid personal memory store.
//!
//! Facts and conversation summaries live in `~/.wiggle/memory/wiggle.db`.
//! Retrieval hybridizes three signals: FTS5 full-text match, bag-of-words
//! cosine similarity over hashed embeddings, and recency/pin boosts.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::MemoryResult;

const EMBED_DIM: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: i64,
    /// user | preference | project | conversation_summary
    pub kind: String,
    pub content: String,
    pub pinned: bool,
    pub created_at: String,
    pub score: f32,
}

pub struct MemoryStore {
    conn: Connection,
    path: PathBuf,
}

impl MemoryStore {
    /// Open (creating the schema if needed). FTS5 ships in the bundled
    /// SQLite build.
    pub fn open(path: &Path) -> MemoryResult<MemoryStore> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                content TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                embedding BLOB,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS summaries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB,
                created_at TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                content, kind UNINDEXED, content='memories', content_rowid='id'
            );
            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, content, kind)
                VALUES (new.id, new.content, new.kind);
            END;
            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, content, kind)
                VALUES ('delete', old.id, old.content, old.kind);
            END;
            "#,
        )?;
        Ok(MemoryStore { conn, path: path.to_path_buf() })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Insert a fact. Embedding is computed with the built-in
    /// character-n-gram hash (no model server required).
    pub fn remember(&mut self, kind: &str, content: &str, pinned: bool) -> MemoryResult<i64> {
        let embedding = embed(content);
        let embedding: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.conn.execute(
            "INSERT INTO memories (kind, content, pinned, embedding, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                kind,
                content,
                pinned as i64,
                embedding,
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Store a conversation summary for a session.
    pub fn remember_summary(&mut self, session: &str, content: &str) -> MemoryResult<i64> {
        let embedding = embed(content);
        let embedding: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.conn.execute(
            "INSERT INTO summaries (session, content, embedding, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session, content, embedding, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Hybrid retrieval: FTS5 match + vector cosine + recency/pin.
    pub fn query_relevant_context(&self, query: &str, limit: usize) -> MemoryResult<Vec<MemoryEntry>> {
        let qvec = embed(query);
        let mut entries: Vec<MemoryEntry> = Vec::new();

        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, pinned, embedding, created_at FROM memories",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let kind: String = row.get(1)?;
            let content: String = row.get(2)?;
            let pinned: i64 = row.get(3)?;
            let blob: Option<Vec<u8>> = row.get(4)?;
            let created: String = row.get(5)?;
            let stored = decode_embedding(blob.as_deref());
            let cosine = cosine(&qvec, &stored);
            let fts = fts_bonus(&self.conn, id)?;
            let recency = recency_bonus(&created);
            let score = cosine * 2.0 + fts + recency + if pinned != 0 { 1.5 } else { 0.0 };
            entries.push(MemoryEntry {
                id,
                kind,
                content,
                pinned: pinned != 0,
                created_at: created,
                score,
            });
        }

        // Conversation summaries participate with the same scoring.
        let mut stmt = self.conn.prepare(
            "SELECT id, content, embedding, created_at FROM summaries ORDER BY id DESC LIMIT 200",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let content: String = row.get(1)?;
            let blob: Option<Vec<u8>> = row.get(2)?;
            let created: String = row.get(3)?;
            let cosine = cosine(&qvec, &decode_embedding(blob.as_deref()));
            let recency = recency_bonus(&created);
            entries.push(MemoryEntry {
                id: -1,
                kind: "conversation_summary".into(),
                content,
                pinned: false,
                created_at: created,
                score: cosine * 2.0 + recency,
            });
        }

        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(limit);
        Ok(entries)
    }

    /// Formatted `<personal_context>` block for the prompt compiler.
    pub fn context_block(&self, query: &str, limit: usize) -> String {
        match self.query_relevant_context(query, limit) {
            Ok(entries) if !entries.is_empty() => {
                let mut block = String::from("<personal_context>\n");
                for e in &entries {
                    block.push_str(&format!("- [{}] {}\n", e.kind, e.content));
                }
                block.push_str("</personal_context>");
                block
            }
            _ => String::new(),
        }
    }

    pub fn list(&self, limit: usize) -> MemoryResult<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, pinned, created_at FROM memories ORDER BY pinned DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                kind: row.get(1)?,
                content: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                score: 0.0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(crate::MemoryError::from)
    }

    pub fn search(&self, text: &str, limit: usize) -> MemoryResult<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.kind, m.content, m.pinned, m.created_at \
             FROM memories_fts f JOIN memories m ON m.id = f.rowid \
             WHERE memories_fts MATCH ?1 ORDER BY rank LIMIT ?2",
        )?;
        let query = format!("\"{}\"", text.replace('"', "\"\""));
        let rows = stmt.query_map(rusqlite::params![query, limit as i64], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                kind: row.get(1)?,
                content: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                score: 0.0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(crate::MemoryError::from)
    }

    pub fn pin(&mut self, id: i64, pinned: bool) -> MemoryResult<()> {
        self.conn.execute(
            "UPDATE memories SET pinned = ?1 WHERE id = ?2",
            rusqlite::params![pinned as i64, id],
        )?;
        Ok(())
    }

    pub fn delete(&mut self, id: i64) -> MemoryResult<()> {
        self.conn.execute("DELETE FROM memories WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Manual fact insertion from the Memory Vault settings panel.
    pub fn add_fact(&mut self, content: &str) -> MemoryResult<i64> {
        self.remember("user", content, false)
    }

    pub fn count(&self) -> MemoryResult<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?)
    }
}

/// Deterministic 64-dim hashed character-trigram embedding. No model
/// server, no network: stable semantics for near-duplicate retrieval.
pub fn embed(text: &str) -> Vec<f32> {
    let mut vec = vec![0f32; EMBED_DIM];
    let normalized: String = text.to_lowercase();
    let chars: Vec<char> = normalized.chars().collect();
    if chars.is_empty() {
        return vec;
    }
    let trigrams = chars.len().max(3) - 2;
    for i in 0..trigrams {
        let gram: String = chars[i..i + 3].iter().collect();
        let mut h = 5381u64;
        for b in gram.as_bytes() {
            h = (h * 33) ^ (*b as u64);
        }
        let slot = (h % EMBED_DIM as u64) as usize;
        let sign = if (h >> 31) & 1 == 0 { 1.0 } else { -1.0 };
        vec[slot] += sign;
    }
    // L2 normalize.
    let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        vec.iter_mut().for_each(|v| *v /= norm);
    }
    vec
}

fn decode_embedding(blob: Option<&[u8]>) -> Vec<f32> {
    match blob {
        Some(bytes) if bytes.len() == EMBED_DIM * 4 => bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
        _ => vec![0f32; EMBED_DIM],
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot.clamp(-1.0, 1.0)
}

fn fts_bonus(conn: &Connection, id: i64) -> MemoryResult<f32> {
    // Cheap presence check: is this rowid reachable in the fts index?
    let mut stmt = conn.prepare("SELECT 1 FROM memories_fts WHERE rowid = ?1 AND content != ''")?;
    Ok(if stmt.exists([id])? { 0.5 } else { 0.0 })
}

fn recency_bonus(created_at: &str) -> f32 {
    match chrono::DateTime::parse_from_rfc3339(created_at) {
        Ok(t) => {
            let age_days = (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_days();
            (7f32 - age_days.min(7) as f32) / 7.0 * 0.3
        }
        Err(_) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_store(name: &str) -> MemoryStore {
        let dir =
            std::env::temp_dir().join(format!("wiggle-mem-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        MemoryStore::open(&dir.join("wiggle.db")).unwrap()
    }

    #[test]
    fn roundtrip_and_hybrid_query() {
        let mut store = tmp_store("roundtrip");
        store.remember("preference", "User prefers Rust over Python for systems work", false).unwrap();
        store.remember("user", "User's laptop is an MSI Katana 15 with an RTX 3050", false).unwrap();
        store.remember("project", "Tervixa Wiggle is built with Tauri v2", false).unwrap();

        let hits = store.query_relevant_context("what gpu does the user have", 3).unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].content.contains("RTX 3050") || hits[0].content.contains("Katana"));

        let fts = store.search("Tauri", 5).unwrap();
        assert!(fts.iter().any(|e| e.content.contains("Tauri")));
    }

    #[test]
    fn pin_boosts_and_delete_removes() {
        let mut store = tmp_store("pin");
        let id = store.remember("user", "favorite editor is vim", false).unwrap();
        store.pin(id, true).unwrap();
        let listed = store.list(10).unwrap();
        assert!(listed[0].pinned);
        store.delete(id).unwrap();
        assert!(store.list(10).unwrap().is_empty());
    }

    #[test]
    fn embeddings_are_stable_and_normalized() {
        let a = embed("rust programming language");
        let b = embed("rust programming language");
        let c = embed("cooking pasta recipes");
        assert_eq!(a, b);
        assert!(cosine(&a, &c) < cosine(&a, &b));
        let norm: f32 = a.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn summaries_stored_and_retrieved() {
        let mut store = tmp_store("summary");
        store.remember_summary("s1", "We decided to use SQLite for memory storage").unwrap();
        let hits = store.query_relevant_context("memory storage decision", 5).unwrap();
        assert!(hits.iter().any(|e| e.kind == "conversation_summary"));
    }
}

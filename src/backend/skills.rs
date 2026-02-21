use crate::shared::models::VgaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    /// Relative path from repo root (e.g. "skills/gui/foo.md").
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillIndex {
    version: u32,
    skills: Vec<SkillEntry>,
}

#[derive(Debug, Clone)]
pub struct SkillRepository {
    root: PathBuf,
    index: Vec<SkillEntry>,
    by_id: HashMap<String, SkillEntry>,
}

impl SkillRepository {
    pub fn load_default() -> Result<Self, VgaError> {
        let root = std::env::current_dir()
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to get current dir: {e}")))?;
        Self::load_from_root(root)
    }

    pub fn load_from_root(root: impl Into<PathBuf>) -> Result<Self, VgaError> {
        let root = root.into();
        let index_path = root.join("skills").join("index.json");
        let raw = fs::read_to_string(&index_path).map_err(|e| {
            VgaError::ResourceLimit(format!(
                "Failed to read skills index {}: {e}",
                index_path.display()
            ))
        })?;

        let parsed: SkillIndex = serde_json::from_str(&raw)
            .map_err(|e| VgaError::ResourceLimit(format!("Invalid skills index JSON: {e}")))?;
        let _ = parsed.version;

        let mut by_id = HashMap::new();
        for entry in &parsed.skills {
            by_id.insert(entry.id.clone(), entry.clone());
        }

        Ok(Self {
            root,
            index: parsed.skills,
            by_id,
        })
    }

    pub fn list(&self) -> &[SkillEntry] {
        &self.index
    }

    pub fn get_entry(&self, id: &str) -> Option<&SkillEntry> {
        self.by_id.get(id)
    }

    pub fn load_text(&self, id: &str) -> Result<String, VgaError> {
        let entry = self
            .get_entry(id)
            .ok_or_else(|| VgaError::ResourceLimit(format!("Skill not found: {id}")))?;
        self.load_path_text(&entry.path)
    }

    pub fn load_path_text(&self, relative_path: &str) -> Result<String, VgaError> {
        let path = self.root.join(relative_path);
        let canonical_root = canonicalize_fallback(&self.root);
        let canonical_path = canonicalize_fallback(&path);

        if !canonical_path.starts_with(&canonical_root) {
            return Err(VgaError::ResourceLimit("Skill path escapes repository root".into()));
        }

        fs::read_to_string(&path).map_err(|e| {
            VgaError::ResourceLimit(format!("Failed to read skill {}: {e}", path.display()))
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn canonicalize_fallback(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

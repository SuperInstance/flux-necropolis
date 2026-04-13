/// Artifact preservation: save important artifacts from dead agents.

use std::collections::HashMap;

/// Category of a preserved artifact.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactCategory {
    SourceCode,
    Configuration,
    KnowledgeBase,
    ModelWeights,
    TestSuite,
    Documentation,
    ToolDefinition,
    CommunicationLog,
}

/// A preserved artifact from a dead agent.
#[derive(Clone, Debug)]
pub struct PreservedArtifact {
    pub id: u32,
    pub name: String,
    pub category: ArtifactCategory,
    pub source_agent: u16,
    pub size_bytes: usize,
    pub checksum: u64,
    pub preserved_at_tick: u64,
    pub importance_score: f64,     // 0.0 to 1.0
    pub access_count: u32,
    pub content_hash: String,
    pub metadata: HashMap<String, String>,
}

/// Result of an artifact search.
#[derive(Clone, Debug)]
pub struct ArtifactSearchResult {
    pub artifact: u32,
    pub relevance_score: f64,
}

/// The artifact preservation system.
pub struct ArtifactVault {
    artifacts: HashMap<u32, PreservedArtifact>,
    next_id: u32,
    max_artifacts: usize,
}

impl ArtifactVault {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            next_id: 1,
            max_artifacts: 256,
        }
    }

    /// Preserve an artifact.
    pub fn preserve(
        &mut self,
        name: &str,
        category: ArtifactCategory,
        source_agent: u16,
        size_bytes: usize,
        checksum: u64,
        tick: u64,
        importance: f64,
        content_hash: &str,
    ) -> Result<u32, String> {
        if self.artifacts.len() >= self.max_artifacts {
            return Err("artifact vault is full".to_string());
        }

        let id = self.next_id;
        self.next_id += 1;
        self.artifacts.insert(id, PreservedArtifact {
            id,
            name: name.to_string(),
            category,
            source_agent,
            size_bytes,
            checksum,
            preserved_at_tick: tick,
            importance_score: importance.clamp(0.0, 1.0),
            access_count: 0,
            content_hash: content_hash.to_string(),
            metadata: HashMap::new(),
        });
        Ok(id)
    }

    /// Add metadata to a preserved artifact.
    pub fn add_metadata(&mut self, id: u32, key: &str, value: &str) -> bool {
        if let Some(artifact) = self.artifacts.get_mut(&id) {
            artifact.metadata.insert(key.to_string(), value.to_string());
            true
        } else {
            false
        }
    }

    /// Access an artifact (increments access count).
    pub fn access(&mut self, id: u32) -> Option<&PreservedArtifact> {
        if let Some(artifact) = self.artifacts.get_mut(&id) {
            artifact.access_count += 1;
            return Some(artifact);
        }
        None
    }

    /// Get an artifact without incrementing access.
    pub fn get(&self, id: u32) -> Option<&PreservedArtifact> {
        self.artifacts.get(&id)
    }

    /// Find artifacts by source agent.
    pub fn by_agent(&self, agent_id: u16) -> Vec<&PreservedArtifact> {
        self.artifacts.values().filter(|a| a.source_agent == agent_id).collect()
    }

    /// Find artifacts by category.
    pub fn by_category(&self, category: &ArtifactCategory) -> Vec<&PreservedArtifact> {
        self.artifacts.values().filter(|a| a.category == *category).collect()
    }

    /// Search artifacts by name substring.
    pub fn search(&self, query: &str) -> Vec<&PreservedArtifact> {
        let q = query.to_lowercase();
        self.artifacts.values()
            .filter(|a| a.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Get most important artifacts.
    pub fn most_important(&self, n: usize) -> Vec<&PreservedArtifact> {
        let mut sorted: Vec<&PreservedArtifact> = self.artifacts.values().collect();
        sorted.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Get most accessed artifacts.
    pub fn most_accessed(&self, n: usize) -> Vec<&PreservedArtifact> {
        let mut sorted: Vec<&PreservedArtifact> = self.artifacts.values().collect();
        sorted.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        sorted.into_iter().take(n).collect()
    }

    /// Get total preserved size in bytes.
    pub fn total_size(&self) -> usize {
        self.artifacts.values().map(|a| a.size_bytes).sum()
    }

    /// Total artifacts count.
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserve_artifact() {
        let mut vault = ArtifactVault::new();
        let id = vault.preserve("model_v2", ArtifactCategory::ModelWeights, 1, 1024, 42, 0, 0.9, "abc123").unwrap();
        assert_eq!(id, 1);
        let a = vault.get(id).unwrap();
        assert_eq!(a.name, "model_v2");
        assert_eq!(a.importance_score, 0.9);
    }

    #[test]
    fn test_add_metadata() {
        let mut vault = ArtifactVault::new();
        let id = vault.preserve("config", ArtifactCategory::Configuration, 1, 64, 0, 0, 0.5, "hash").unwrap();
        vault.add_metadata(id, "format", "toml");
        vault.add_metadata(id, "version", "2");
        assert_eq!(vault.get(id).unwrap().metadata.len(), 2);
    }

    #[test]
    fn test_access_increments() {
        let mut vault = ArtifactVault::new();
        let id = vault.preserve("kb", ArtifactCategory::KnowledgeBase, 1, 512, 0, 0, 0.7, "h").unwrap();
        vault.access(id);
        vault.access(id);
        assert_eq!(vault.get(id).unwrap().access_count, 2);
    }

    #[test]
    fn test_by_agent() {
        let mut vault = ArtifactVault::new();
        vault.preserve("a1", ArtifactCategory::SourceCode, 10, 100, 0, 0, 0.5, "h").unwrap();
        vault.preserve("a2", ArtifactCategory::SourceCode, 10, 200, 0, 0, 0.6, "h").unwrap();
        vault.preserve("b1", ArtifactCategory::SourceCode, 20, 300, 0, 0, 0.7, "h").unwrap();
        assert_eq!(vault.by_agent(10).len(), 2);
    }

    #[test]
    fn test_by_category() {
        let mut vault = ArtifactVault::new();
        vault.preserve("src", ArtifactCategory::SourceCode, 1, 100, 0, 0, 0.5, "h").unwrap();
        vault.preserve("cfg", ArtifactCategory::Configuration, 1, 50, 0, 0, 0.5, "h").unwrap();
        vault.preserve("src2", ArtifactCategory::SourceCode, 2, 200, 0, 0, 0.5, "h").unwrap();
        assert_eq!(vault.by_category(&ArtifactCategory::SourceCode).len(), 2);
    }

    #[test]
    fn test_search() {
        let mut vault = ArtifactVault::new();
        vault.preserve("important_model", ArtifactCategory::ModelWeights, 1, 1024, 0, 0, 0.9, "h").unwrap();
        vault.preserve("other_file", ArtifactCategory::SourceCode, 1, 100, 0, 0, 0.5, "h").unwrap();
        let results = vault.search("model");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_most_important() {
        let mut vault = ArtifactVault::new();
        vault.preserve("low", ArtifactCategory::Documentation, 1, 10, 0, 0, 0.2, "h").unwrap();
        vault.preserve("high", ArtifactCategory::ModelWeights, 1, 100, 0, 0, 0.9, "h").unwrap();
        vault.preserve("mid", ArtifactCategory::KnowledgeBase, 1, 50, 0, 0, 0.5, "h").unwrap();
        let top = vault.most_important(1);
        assert_eq!(top[0].name, "high");
    }

    #[test]
    fn test_importance_clamp() {
        let mut vault = ArtifactVault::new();
        let id = vault.preserve("x", ArtifactCategory::SourceCode, 1, 10, 0, 0, 1.5, "h").unwrap();
        assert_eq!(vault.get(id).unwrap().importance_score, 1.0);
    }
}

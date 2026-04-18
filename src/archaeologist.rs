/// Archaeological analysis: analyze patterns in dead code for insights.

use crate::cemetery::CodeCemetery;
use crate::memory_tombstone::MemoryTombstoneSystem;
use crate::process_graveyard::ProcessGraveyard;
use crate::artifact_vault::ArtifactVault;
use std::collections::HashMap;

/// A pattern discovered through archaeological analysis.
#[derive(Clone, Debug)]
pub struct DiscoveredPattern {
    pub name: String,
    pub description: String,
    pub confidence: f64,       // 0.0 to 1.0
    pub occurrence_count: usize,
    pub affected_entities: Vec<String>,  // names of affected entities
    pub severity: PatternSeverity,
}

/// Severity level of a discovered pattern.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternSeverity {
    Info,
    Warning,
    Critical,
}

/// Comprehensive archaeology report.
#[derive(Clone, Debug, Default)]
pub struct ArchaeologyReport {
    pub patterns: Vec<DiscoveredPattern>,
    pub dead_code_by_module: HashMap<String, usize>,
    pub memory_leak_candidates: Vec<String>,
    pub common_failure_modes: HashMap<String, usize>,
    pub total_dead_code_entries: usize,
    pub total_freed_memory_bytes: usize,
    pub total_terminated_processes: usize,
    pub preserved_artifact_count: usize,
    pub insights: Vec<String>,
}

/// The archaeological analysis engine.
pub struct Archaeologist {
    reports: Vec<ArchaeologyReport>,
}

impl Archaeologist {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    /// Perform a comprehensive analysis across all subsystems.
    pub fn analyze(
        &mut self,
        cemetery: &CodeCemetery,
        memory_ts: &MemoryTombstoneSystem,
        process_gy: &ProcessGraveyard,
        artifact_vault: &ArtifactVault,
        current_tick: u64,
    ) -> ArchaeologyReport {
        let mut report = ArchaeologyReport::default();

        // 1. Analyze dead code by module
        let buried = cemetery.entries_by_status(&crate::cemetery::CodeStatus::Buried);
        report.total_dead_code_entries = buried.len();
        for entry in &buried {
            *report.dead_code_by_module.entry(entry.module.clone()).or_insert(0) += 1;
        }

        // 2. Analyze memory tombstones for leak candidates
        let mem_stats = memory_ts.statistics(current_tick);
        report.total_freed_memory_bytes = mem_stats.total_freed_bytes;
        if mem_stats.total_freed_bytes > 0 {
            report.insights.push(format!(
                "Total freed memory: {} bytes across {} regions",
                mem_stats.total_freed_bytes, mem_stats.total_tombstones
            ));
        }

        // Find potential leak candidates (same owner, many freed regions)
        let mut owner_counts: HashMap<String, usize> = HashMap::new();
        for ts in &memory_ts.tombstones {
            *owner_counts.entry(ts.owner.clone()).or_insert(0) += 1;
        }
        for (owner, count) in &owner_counts {
            if *count >= 3 {
                report.memory_leak_candidates.push(format!("{} ({} freed regions)", owner, count));
            }
        }
        if !report.memory_leak_candidates.is_empty() {
            report.patterns.push(DiscoveredPattern {
                name: "Memory Leak Pattern".to_string(),
                description: format!("{} entities have 3+ freed memory regions", report.memory_leak_candidates.len()),
                confidence: 0.6,
                occurrence_count: report.memory_leak_candidates.len(),
                affected_entities: report.memory_leak_candidates.clone(),
                severity: PatternSeverity::Warning,
            });
        }

        // 3. Analyze process termination patterns
        use crate::process_graveyard::ProcessStatus;
        let terminated: Vec<_> = process_gy.records.values()
            .filter(|r| r.status == ProcessStatus::Terminated || r.status == ProcessStatus::Archived)
            .collect();
        report.total_terminated_processes = terminated.len();

        for record in &terminated {
            if let Some(ref reason) = record.termination_reason {
                let reason_str = format!("{:?}", reason);
                let key = reason_str.split('(').next().unwrap_or(&reason_str).to_string();
                *report.common_failure_modes.entry(key).or_insert(0) += 1;
            }
        }

        // Find crash patterns
        let crashed = process_gy.crashed();
        if !crashed.is_empty() {
            report.patterns.push(DiscoveredPattern {
                name: "Crash Pattern".to_string(),
                description: format!("{} processes crashed during execution", crashed.len()),
                confidence: 0.9,
                occurrence_count: crashed.len(),
                affected_entities: crashed.iter().map(|c| c.name.clone()).collect(),
                severity: PatternSeverity::Critical,
            });
        }

        // 4. Analyze artifacts
        report.preserved_artifact_count = artifact_vault.len();
        if artifact_vault.len() > 0 {
            report.insights.push(format!(
                "{} artifacts preserved, total size: {} bytes",
                artifact_vault.len(), artifact_vault.total_size()
            ));
        }

        // 5. Generate module-level dead code pattern
        for (module, count) in &report.dead_code_by_module {
            if *count >= 2 {
                report.patterns.push(DiscoveredPattern {
                    name: format!("Dead Code Cluster in {}", module),
                    description: format!("{} dead code entries in module '{}'", count, module),
                    confidence: 0.7,
                    occurrence_count: *count,
                    affected_entities: buried.iter()
                        .filter(|e| e.module == *module)
                        .map(|e| e.name.clone())
                        .collect(),
                    severity: if *count >= 5 { PatternSeverity::Critical } else { PatternSeverity::Warning },
                });
            }
        }

        self.reports.push(report.clone());
        report
    }

    /// Get historical reports.
    pub fn reports(&self) -> &[ArchaeologyReport] {
        &self.reports
    }

    /// Compare latest two reports for changes.
    pub fn delta_analysis(&self) -> Option<String> {
        if self.reports.len() < 2 {
            return None;
        }
        let prev = &self.reports[self.reports.len() - 2];
        let curr = &self.reports[self.reports.len() - 1];

        let mut delta = String::new();
        let dead_delta = curr.total_dead_code_entries as i64 - prev.total_dead_code_entries as i64;
        if dead_delta != 0 {
            delta.push_str(&format!("Dead code: {:+} entries\n", dead_delta));
        }
        let mem_delta = curr.total_freed_memory_bytes as i64 - prev.total_freed_memory_bytes as i64;
        if mem_delta != 0 {
            delta.push_str(&format!("Freed memory: {:+} bytes\n", mem_delta));
        }
        let proc_delta = curr.total_terminated_processes as i64 - prev.total_terminated_processes as i64;
        if proc_delta != 0 {
            delta.push_str(&format!("Terminated processes: {:+}\n", proc_delta));
        }

        if delta.is_empty() {
            delta = "No significant changes detected.".to_string();
        }
        Some(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty() {
        let mut arch = Archaeologist::new();
        let cem = CodeCemetery::new();
        let mts = MemoryTombstoneSystem::new();
        let pg = ProcessGraveyard::new();
        let vault = ArtifactVault::new();
        let report = arch.analyze(&cem, &mts, &pg, &vault, 100);
        assert_eq!(report.total_dead_code_entries, 0);
        assert_eq!(report.total_terminated_processes, 0);
    }

    #[test]
    fn test_analyze_with_data() {
        let mut arch = Archaeologist::new();
        let mut cem = CodeCemetery::new();
        let id1 = cem.register("dead1", "core", 0, 100, 0);
        let id2 = cem.register("dead2", "core", 0, 200, 0);
        cem.bury(id1, 10, "unused").unwrap();
        cem.bury(id2, 10, "unused").unwrap();

        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 64, crate::memory_tombstone::MemoryDataType::HeapObject, "leaky_process", 0, 10, 0, vec![], "x").unwrap();
        mts.mark_freed(0x2000, 128, crate::memory_tombstone::MemoryDataType::HeapObject, "leaky_process", 0, 20, 0, vec![], "x").unwrap();
        mts.mark_freed(0x3000, 64, crate::memory_tombstone::MemoryDataType::HeapObject, "leaky_process", 0, 30, 0, vec![], "x").unwrap();

        let mut pg = ProcessGraveyard::new();
        let pid = pg.register("crasher", 1, 0);
        pg.terminate(pid, 100, crate::process_graveyard::TerminationReason::Crash("segfault".to_string()), 1000, 512, 5000).unwrap();

        let vault = ArtifactVault::new();

        let report = arch.analyze(&cem, &mts, &pg, &vault, 200);
        assert_eq!(report.total_dead_code_entries, 2);
        assert_eq!(report.total_freed_memory_bytes, 256);
        assert_eq!(report.total_terminated_processes, 1);
        assert!(!report.patterns.is_empty());
        assert!(!report.memory_leak_candidates.is_empty());
    }

    #[test]
    fn test_delta_analysis() {
        let mut arch = Archaeologist::new();
        let cem = CodeCemetery::new();
        let mts = MemoryTombstoneSystem::new();
        let pg = ProcessGraveyard::new();
        let vault = ArtifactVault::new();

        arch.analyze(&cem, &mts, &pg, &vault, 100);
        assert!(arch.delta_analysis().is_none()); // need at least 2 reports
    }

    #[test]
    fn test_dead_code_module_clustering() {
        let mut arch = Archaeologist::new();
        let mut cem = CodeCemetery::new();
        for i in 0..3 {
            let id = cem.register(&format!("func_{}", i), "clustered_module", 0, 100, 0);
            cem.bury(id, 10, "dead").unwrap();
        }

        let report = arch.analyze(
            &cem, &MemoryTombstoneSystem::new(), &ProcessGraveyard::new(), &ArtifactVault::new(), 100
        );
        let module_patterns: Vec<_> = report.patterns.iter()
            .filter(|p| p.name.contains("Dead Code Cluster"))
            .collect();
        assert_eq!(module_patterns.len(), 1);
    }
}

/// Resurrection protocol: selectively revive useful dead code/agents.

use crate::cemetery::{CodeCemetery, CodeStatus};
use crate::process_graveyard::{ProcessGraveyard, TerminationReason};

/// Status of a resurrection attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResurrectionStatus {
    Pending,
    InProgress,
    Succeeded,
    Failed(String),
    Rejected,  // deemed not worth reviving
}

/// Criteria for evaluating resurrection candidates.
#[derive(Clone, Debug)]
pub struct ResurrectionCriteria {
    pub min_importance: f64,
    pub max_age_ticks: u64,
    pub allow_crashed: bool,
    pub min_references: u32,
}

impl Default for ResurrectionCriteria {
    fn default() -> Self {
        Self {
            min_importance: 0.3,
            max_age_ticks: 10000,
            allow_crashed: true,
            min_references: 1,
        }
    }
}

/// A resurrection candidate with evaluation score.
#[derive(Clone, Debug)]
pub struct ResurrectionCandidate {
    pub entity_id: u32,
    pub entity_type: String,  // "code", "process", "memory"
    pub name: String,
    pub score: f64,
    pub reason: String,
}

/// A record of a completed resurrection.
#[derive(Clone, Debug)]
pub struct ResurrectionRecord {
    pub candidate_id: u32,
    pub entity_type: String,
    pub entity_name: String,
    pub status: ResurrectionStatus,
    pub ticks_taken: u64,
    pub artifacts_recovered: u32,
    pub notes: String,
}

/// The resurrection protocol manager.
pub struct ResurrectionProtocol {
    pub criteria: ResurrectionCriteria,
    pub candidates: Vec<ResurrectionCandidate>,
    pub history: Vec<ResurrectionRecord>,
    pub pending_count: usize,
    pub succeeded_count: usize,
    pub rejected_count: usize,
}

impl ResurrectionProtocol {
    pub fn new() -> Self {
        Self {
            criteria: ResurrectionCriteria::default(),
            candidates: Vec::new(),
            history: Vec::new(),
            pending_count: 0,
            succeeded_count: 0,
            rejected_count: 0,
        }
    }

    /// Evaluate code cemetery entries as resurrection candidates.
    pub fn evaluate_code_cemetery(&mut self, cemetery: &CodeCemetery, current_tick: u64) {
        for entry in cemetery.entries_by_status(&CodeStatus::Buried) {
            let age = current_tick.saturating_sub(entry.burial_tick.unwrap_or(0));
            if age > self.criteria.max_age_ticks {
                continue;
            }
            let score = if entry.reference_count >= 3 {
                0.8 + (entry.exhumation_count as f64 * 0.05).min(0.2)
            } else if entry.reference_count >= 1 {
                0.4 + (entry.exhumation_count as f64 * 0.05).min(0.2)
            } else {
                entry.exhumation_count as f64 * 0.1
            };

            if score >= self.criteria.min_importance {
                self.candidates.push(ResurrectionCandidate {
                    entity_id: entry.id,
                    entity_type: "code".to_string(),
                    name: entry.name.clone(),
                    score,
                    reason: format!("buried code, {} references, {} prior exhumations", 
                        entry.reference_count, entry.exhumation_count),
                });
            }
        }
    }

    /// Evaluate process graveyard entries as resurrection candidates.
    pub fn evaluate_process_graveyard(&mut self, graveyard: &ProcessGraveyard, current_tick: u64) {
        use crate::process_graveyard::ProcessStatus;

        for record in graveyard.records.values() {
            if record.status != ProcessStatus::Terminated && record.status != ProcessStatus::Archived {
                continue;
            }

            let terminated_tick = record.terminated_tick.unwrap_or(0);
            let age = current_tick.saturating_sub(terminated_tick);
            if age > self.criteria.max_age_ticks {
                continue;
            }

            let terminated_reason = &record.termination_reason;
            if !self.criteria.allow_crashed {
                if let Some(TerminationReason::Crash(_)) = terminated_reason {
                    continue;
                }
            }

            // Score based on execution metrics
            let score = if matches!(terminated_reason, Some(TerminationReason::Completed)) {
                0.9 // completed processes are prime candidates
            } else if matches!(terminated_reason, Some(TerminationReason::TimedOut)) {
                0.5 // timed out but may be salvageable
            } else if matches!(terminated_reason, Some(TerminationReason::Crash(_))) {
                0.3 // crashed, risky
            } else {
                0.4
            };

            if score >= self.criteria.min_importance {
                self.candidates.push(ResurrectionCandidate {
                    entity_id: record.id,
                    entity_type: "process".to_string(),
                    name: record.name.clone(),
                    score,
                    reason: format!("{:?}, used {} CPU ticks", terminated_reason, record.cpu_ticks_used),
                });
            }
        }
    }

    /// Accept a resurrection candidate.
    pub fn accept(&mut self, candidate_index: usize) -> Result<ResurrectionRecord, String> {
        if candidate_index >= self.candidates.len() {
            return Err("invalid candidate index".to_string());
        }
        let candidate = self.candidates.remove(candidate_index);
        self.pending_count += 1;
        Ok(ResurrectionRecord {
            candidate_id: candidate.entity_id,
            entity_type: candidate.entity_type.clone(),
            entity_name: candidate.name,
            status: ResurrectionStatus::Pending,
            ticks_taken: 0,
            artifacts_recovered: 0,
            notes: candidate.reason,
        })
    }

    /// Reject a resurrection candidate.
    pub fn reject(&mut self, candidate_index: usize) -> Result<(), String> {
        if candidate_index >= self.candidates.len() {
            return Err("invalid candidate index".to_string());
        }
        self.candidates.remove(candidate_index);
        self.rejected_count += 1;
        Ok(())
    }

    /// Complete a resurrection (mark as succeeded).
    pub fn complete_resurrection(&mut self, record: &mut ResurrectionRecord, ticks: u64, artifacts: u32) {
        record.status = ResurrectionStatus::Succeeded;
        record.ticks_taken = ticks;
        record.artifacts_recovered = artifacts;
        self.pending_count = self.pending_count.saturating_sub(1);
        self.succeeded_count += 1;
        self.history.push(record.clone());
    }

    /// Fail a resurrection.
    pub fn fail_resurrection(&mut self, record: &mut ResurrectionRecord, reason: &str) {
        record.status = ResurrectionStatus::Failed(reason.to_string());
        self.pending_count = self.pending_count.saturating_sub(1);
        self.history.push(record.clone());
    }

    /// Get candidates sorted by score (highest first).
    pub fn ranked_candidates(&self) -> Vec<&ResurrectionCandidate> {
        let mut sorted: Vec<&ResurrectionCandidate> = self.candidates.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    /// Count pending candidates.
    pub fn pending_candidate_count(&self) -> usize {
        self.candidates.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_protocol() {
        let rp = ResurrectionProtocol::new();
        assert_eq!(rp.pending_candidate_count(), 0);
        assert_eq!(rp.succeeded_count, 0);
    }

    #[test]
    fn test_evaluate_code_cemetery() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("useful_func", "core", 42, 256, 0);
        for i in 0..5 { cem.touch(id, i); }
        cem.bury(id, 100, "unused").unwrap();

        let mut rp = ResurrectionProtocol::new();
        rp.evaluate_code_cemetery(&cem, 200);
        let ranked = rp.ranked_candidates();
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].entity_type, "code");
    }

    #[test]
    fn test_evaluate_process_graveyard() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("good_worker", 10, 0);
        pg.terminate(id, 100, TerminationReason::Completed, 5000, 1024, 10000).unwrap();

        let mut rp = ResurrectionProtocol::new();
        rp.evaluate_process_graveyard(&pg, 200);
        let ranked = rp.ranked_candidates();
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].entity_type, "process");
        assert!(ranked[0].score >= 0.8);
    }

    #[test]
    fn test_accept_candidate() {
        let mut rp = ResurrectionProtocol::new();
        rp.candidates.push(ResurrectionCandidate {
            entity_id: 1, entity_type: "code".to_string(), name: "test".to_string(),
            score: 0.8, reason: "test".to_string(),
        });
        let record = rp.accept(0).unwrap();
        assert_eq!(record.status, ResurrectionStatus::Pending);
        assert_eq!(rp.pending_candidate_count(), 0);
    }

    #[test]
    fn test_reject_candidate() {
        let mut rp = ResurrectionProtocol::new();
        rp.candidates.push(ResurrectionCandidate {
            entity_id: 1, entity_type: "code".to_string(), name: "test".to_string(),
            score: 0.1, reason: "test".to_string(),
        });
        rp.reject(0).unwrap();
        assert_eq!(rp.rejected_count, 1);
        assert_eq!(rp.pending_candidate_count(), 0);
    }

    #[test]
    fn test_complete_resurrection() {
        let mut rp = ResurrectionProtocol::new();
        let mut record = ResurrectionRecord {
            candidate_id: 1, entity_type: "code".to_string(), entity_name: "func".to_string(),
            status: ResurrectionStatus::Pending, ticks_taken: 0, artifacts_recovered: 0,
            notes: "test".to_string(),
        };
        rp.pending_count = 1;
        rp.complete_resurrection(&mut record, 50, 3);
        assert_eq!(rp.succeeded_count, 1);
        assert_eq!(rp.pending_count, 0);
        assert_eq!(record.artifacts_recovered, 3);
    }

    #[test]
    fn test_fail_resurrection() {
        let mut rp = ResurrectionProtocol::new();
        let mut record = ResurrectionRecord {
            candidate_id: 1, entity_type: "process".to_string(), entity_name: "proc".to_string(),
            status: ResurrectionStatus::Pending, ticks_taken: 0, artifacts_recovered: 0,
            notes: "test".to_string(),
        };
        rp.pending_count = 1;
        rp.fail_resurrection(&mut record, "code no longer compatible");
        assert_eq!(record.status, ResurrectionStatus::Failed("code no longer compatible".to_string()));
        assert_eq!(rp.pending_count, 0);
    }

    #[test]
    fn test_ranked_candidates_ordering() {
        let mut rp = ResurrectionProtocol::new();
        rp.candidates.push(ResurrectionCandidate {
            entity_id: 1, entity_type: "code".to_string(), name: "low".to_string(),
            score: 0.2, reason: "test".to_string(),
        });
        rp.candidates.push(ResurrectionCandidate {
            entity_id: 2, entity_type: "code".to_string(), name: "high".to_string(),
            score: 0.9, reason: "test".to_string(),
        });
        let ranked = rp.ranked_candidates();
        assert_eq!(ranked[0].name, "high");
        assert_eq!(ranked[1].name, "low");
    }
}

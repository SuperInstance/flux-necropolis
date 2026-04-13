/// Process graveyard: keep records of terminated agent processes.

use std::collections::HashMap;

/// Reason for process termination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminationReason {
    Completed,
    Failed,
    TimedOut,
    Killed,
    Crash(String),
    OutOfMemory,
    Deadlock,
    ErrorBoundary,
}

/// Status of a process record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Terminated,
    Archived,
}

/// A record of a terminated agent process.
#[derive(Clone, Debug)]
pub struct ProcessRecord {
    pub id: u32,
    pub name: String,
    pub agent_id: u16,
    pub status: ProcessStatus,
    pub termination_reason: Option<TerminationReason>,
    pub started_tick: u64,
    pub terminated_tick: Option<u64>,
    pub cpu_ticks_used: u64,
    pub memory_peak_bytes: usize,
    pub instructions_executed: u64,
    pub output_summary: String,
    pub error_log: Vec<String>,
}

/// The process graveyard.
pub struct ProcessGraveyard {
    pub records: HashMap<u32, ProcessRecord>,
    next_id: u32,
}

impl ProcessGraveyard {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new process.
    pub fn register(&mut self, name: &str, agent_id: u16, tick: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.records.insert(id, ProcessRecord {
            id,
            name: name.to_string(),
            agent_id,
            status: ProcessStatus::Running,
            termination_reason: None,
            started_tick: tick,
            terminated_tick: None,
            cpu_ticks_used: 0,
            memory_peak_bytes: 0,
            instructions_executed: 0,
            output_summary: String::new(),
            error_log: Vec::new(),
        });
        id
    }

    /// Terminate a process with a reason.
    pub fn terminate(&mut self, id: u32, tick: u64, reason: TerminationReason, cpu_ticks: u64, mem_peak: usize, instructions: u64) -> Result<(), String> {
        let record = self.records.get_mut(&id).ok_or("process not found")?;
        record.status = ProcessStatus::Terminated;
        record.termination_reason = Some(reason);
        record.terminated_tick = Some(tick);
        record.cpu_ticks_used = cpu_ticks;
        record.memory_peak_bytes = mem_peak;
        record.instructions_executed = instructions;
        Ok(())
    }

    /// Archive a terminated process.
    pub fn archive(&mut self, id: u32) -> Result<(), String> {
        let record = self.records.get_mut(&id).ok_or("process not found")?;
        if record.status != ProcessStatus::Terminated {
            return Err("process is not terminated".to_string());
        }
        record.status = ProcessStatus::Archived;
        Ok(())
    }

    /// Add an error log entry.
    pub fn add_error(&mut self, id: u32, error: &str) -> bool {
        if let Some(record) = self.records.get_mut(&id) {
            record.error_log.push(error.to_string());
            true
        } else {
            false
        }
    }

    /// Set output summary.
    pub fn set_output(&mut self, id: u32, summary: &str) -> bool {
        if let Some(record) = self.records.get_mut(&id) {
            record.output_summary = summary.to_string();
            true
        } else {
            false
        }
    }

    /// Get a process record.
    pub fn get(&self, id: u32) -> Option<&ProcessRecord> {
        self.records.get(&id)
    }

    /// Get processes by agent ID.
    pub fn by_agent(&self, agent_id: u16) -> Vec<&ProcessRecord> {
        self.records.values().filter(|r| r.agent_id == agent_id).collect()
    }

    /// Get processes by termination reason.
    pub fn by_termination_reason(&self, reason: &TerminationReason) -> Vec<&ProcessRecord> {
        self.records.values()
            .filter(|r| r.termination_reason.as_ref() == Some(reason))
            .collect()
    }

    /// Get crashed processes.
    pub fn crashed(&self) -> Vec<&ProcessRecord> {
        self.records.values()
            .filter(|r| matches!(r.termination_reason, Some(TerminationReason::Crash(_))))
            .collect()
    }

    /// Get total CPU ticks used by terminated processes.
    pub fn total_cpu_ticks(&self) -> u64 {
        self.records.values().map(|r| r.cpu_ticks_used).sum()
    }

    /// Get process count by status.
    pub fn count_by_status(&self, status: &ProcessStatus) -> usize {
        self.records.values().filter(|r| r.status == *status).count()
    }

    /// Total records.
    pub fn len(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_process() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("worker_1", 10, 0);
        assert_eq!(id, 1);
        assert_eq!(pg.get(id).unwrap().status, ProcessStatus::Running);
    }

    #[test]
    fn test_terminate_process() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("worker_1", 10, 0);
        pg.terminate(id, 100, TerminationReason::Completed, 5000, 1024, 10000).unwrap();
        let r = pg.get(id).unwrap();
        assert_eq!(r.status, ProcessStatus::Terminated);
        assert_eq!(r.cpu_ticks_used, 5000);
    }

    #[test]
    fn test_archive_process() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("worker_1", 10, 0);
        pg.terminate(id, 100, TerminationReason::Failed, 1000, 512, 5000).unwrap();
        pg.archive(id).unwrap();
        assert_eq!(pg.get(id).unwrap().status, ProcessStatus::Archived);
    }

    #[test]
    fn test_archive_running_fails() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("worker_1", 10, 0);
        assert!(pg.archive(id).is_err());
    }

    #[test]
    fn test_add_error() {
        let mut pg = ProcessGraveyard::new();
        let id = pg.register("worker_1", 10, 0);
        pg.add_error(id, "segmentation fault");
        pg.add_error(id, "core dumped");
        assert_eq!(pg.get(id).unwrap().error_log.len(), 2);
    }

    #[test]
    fn test_by_agent() {
        let mut pg = ProcessGraveyard::new();
        pg.register("w1", 10, 0);
        pg.register("w2", 10, 0);
        pg.register("w3", 20, 0);
        assert_eq!(pg.by_agent(10).len(), 2);
    }

    #[test]
    fn test_crashed() {
        let mut pg = ProcessGraveyard::new();
        let id1 = pg.register("w1", 10, 0);
        let id2 = pg.register("w2", 10, 0);
        pg.terminate(id1, 100, TerminationReason::Crash("segfault".to_string()), 100, 512, 1000).unwrap();
        pg.terminate(id2, 100, TerminationReason::Completed, 100, 512, 1000).unwrap();
        assert_eq!(pg.crashed().len(), 1);
    }

    #[test]
    fn test_total_cpu_ticks() {
        let mut pg = ProcessGraveyard::new();
        let id1 = pg.register("w1", 10, 0);
        let id2 = pg.register("w2", 10, 0);
        pg.terminate(id1, 100, TerminationReason::Completed, 1000, 512, 5000).unwrap();
        pg.terminate(id2, 100, TerminationReason::Completed, 2000, 512, 5000).unwrap();
        assert_eq!(pg.total_cpu_ticks(), 3000);
    }
}

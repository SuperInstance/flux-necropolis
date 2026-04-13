/// Dead code cemetery: track and archive unused bytecode/instructions.

use std::collections::HashMap;

/// Status of a dead code entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeStatus {
    Active,
    Suspect,      // suspected unused
    Dead,         // confirmed dead
    Buried,       // archived in cemetery
    Exhumed,      // temporarily removed for analysis
}

/// A tracked piece of code/instruction.
#[derive(Clone, Debug)]
pub struct DeadCodeEntry {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub bytecode_hash: u64,
    pub size_bytes: usize,
    pub last_referenced_tick: u64,
    pub reference_count: u32,
    pub status: CodeStatus,
    pub burial_tick: Option<u64>,
    pub burial_reason: String,
    pub exhumation_count: u32,
}

/// The cemetery that tracks dead code.
pub struct CodeCemetery {
    entries: HashMap<u32, DeadCodeEntry>,
    next_id: u32,
    suspect_threshold_ticks: u64,
    max_capacity: usize,
}

impl CodeCemetery {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 1,
            suspect_threshold_ticks: 100,
            max_capacity: 256,
        }
    }

    /// Register a new code entry.
    pub fn register(&mut self, name: &str, module: &str, bytecode_hash: u64, size_bytes: usize, current_tick: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(id, DeadCodeEntry {
            id,
            name: name.to_string(),
            module: module.to_string(),
            bytecode_hash,
            size_bytes,
            last_referenced_tick: current_tick,
            reference_count: 1,
            status: CodeStatus::Active,
            burial_tick: None,
            burial_reason: String::new(),
            exhumation_count: 0,
        });
        id
    }

    /// Record a reference to a code entry.
    pub fn touch(&mut self, id: u32, tick: u64) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.last_referenced_tick = tick;
            entry.reference_count += 1;
            if entry.status == CodeStatus::Suspect {
                entry.status = CodeStatus::Active;
            }
            true
        } else {
            false
        }
    }

    /// Mark an entry as suspect (potentially dead).
    pub fn mark_suspect(&mut self, id: u32) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            if entry.status == CodeStatus::Active {
                entry.status = CodeStatus::Suspect;
                return true;
            }
        }
        false
    }

    /// Bury a dead code entry (confirm it as dead and archive it).
    pub fn bury(&mut self, id: u32, tick: u64, reason: &str) -> Result<(), String> {
        let current_buried = self.count_by_status(&CodeStatus::Buried);
        if current_buried >= self.max_capacity {
            return Err("cemetery is full".to_string());
        }
        let entry = self.entries.get_mut(&id).ok_or("entry not found")?;
        entry.status = CodeStatus::Buried;
        entry.burial_tick = Some(tick);
        entry.burial_reason = reason.to_string();
        Ok(())
    }

    /// Exhume a buried entry for analysis.
    pub fn exhume(&mut self, id: u32) -> Result<&DeadCodeEntry, String> {
        let entry = self.entries.get_mut(&id).ok_or("entry not found")?;
        if entry.status != CodeStatus::Buried {
            return Err("entry is not buried".to_string());
        }
        entry.status = CodeStatus::Exhumed;
        entry.exhumation_count += 1;
        Ok(self.entries.get(&id).unwrap())
    }

    /// Get an entry by ID.
    pub fn get(&self, id: u32) -> Option<&DeadCodeEntry> {
        self.entries.get(&id)
    }

    /// Count entries by status.
    pub fn count_by_status(&self, status: &CodeStatus) -> usize {
        self.entries.values().filter(|e| e.status == *status).count()
    }

    /// Get all entries of a given status.
    pub fn entries_by_status(&self, status: &CodeStatus) -> Vec<&DeadCodeEntry> {
        self.entries.values().filter(|e| e.status == *status).collect()
    }

    /// Get entries from a specific module.
    pub fn entries_by_module(&self, module: &str) -> Vec<&DeadCodeEntry> {
        self.entries.values().filter(|e| e.module == module).collect()
    }

    /// Total size of buried code in bytes.
    pub fn total_buried_size(&self) -> usize {
        self.entries.values()
            .filter(|e| e.status == CodeStatus::Buried)
            .map(|e| e.size_bytes)
            .sum()
    }

    /// Get most frequently exhumed entries.
    pub fn most_exhumed(&self, n: usize) -> Vec<&DeadCodeEntry> {
        let mut entries: Vec<&DeadCodeEntry> = self.entries.values()
            .filter(|e| e.exhumation_count > 0)
            .collect();
        entries.sort_by(|a, b| b.exhumation_count.cmp(&a.exhumation_count));
        entries.into_iter().take(n).collect()
    }

    /// Get total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_entry() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("main_loop", "core", 12345, 256, 0);
        assert_eq!(id, 1);
        let entry = cem.get(id).unwrap();
        assert_eq!(entry.name, "main_loop");
        assert_eq!(entry.status, CodeStatus::Active);
    }

    #[test]
    fn test_touch_entry() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("func", "mod", 0, 100, 0);
        cem.mark_suspect(id);
        assert_eq!(cem.get(id).unwrap().status, CodeStatus::Suspect);
        cem.touch(id, 50);
        assert_eq!(cem.get(id).unwrap().status, CodeStatus::Active);
        assert_eq!(cem.get(id).unwrap().reference_count, 2);
    }

    #[test]
    fn test_mark_suspect() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("func", "mod", 0, 100, 0);
        assert!(cem.mark_suspect(id));
        assert_eq!(cem.get(id).unwrap().status, CodeStatus::Suspect);
    }

    #[test]
    fn test_bury_entry() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("dead_func", "mod", 0, 100, 0);
        cem.bury(id, 100, "unused for 1000 ticks").unwrap();
        assert_eq!(cem.get(id).unwrap().status, CodeStatus::Buried);
        assert_eq!(cem.get(id).unwrap().burial_tick, Some(100));
    }

    #[test]
    fn test_exhume_entry() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("dead_func", "mod", 0, 100, 0);
        cem.bury(id, 100, "reason").unwrap();
        cem.exhume(id).unwrap();
        assert_eq!(cem.get(id).unwrap().status, CodeStatus::Exhumed);
        assert_eq!(cem.get(id).unwrap().exhumation_count, 1);
    }

    #[test]
    fn test_exhume_non_buried_fails() {
        let mut cem = CodeCemetery::new();
        let id = cem.register("func", "mod", 0, 100, 0);
        assert!(cem.exhume(id).is_err());
    }

    #[test]
    fn test_entries_by_module() {
        let mut cem = CodeCemetery::new();
        cem.register("f1", "module_a", 0, 50, 0);
        cem.register("f2", "module_a", 0, 60, 0);
        cem.register("f3", "module_b", 0, 70, 0);
        assert_eq!(cem.entries_by_module("module_a").len(), 2);
    }

    #[test]
    fn test_total_buried_size() {
        let mut cem = CodeCemetery::new();
        let id1 = cem.register("f1", "mod", 0, 100, 0);
        let id2 = cem.register("f2", "mod", 0, 200, 0);
        cem.bury(id1, 10, "dead").unwrap();
        cem.bury(id2, 10, "dead").unwrap();
        assert_eq!(cem.total_buried_size(), 300);
    }
}

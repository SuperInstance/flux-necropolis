/// Memory tombstone system: mark freed memory with metadata about what was there.

/// The type of data that occupied a memory region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryDataType {
    Instruction,
    StackFrame,
    HeapObject,
    Buffer,
    Metadata,
    Unknown,
}

/// A tombstone placed at a freed memory location.
#[derive(Clone, Debug)]
pub struct MemoryTombstone {
    pub address: u64,
    pub size: usize,
    pub data_type: MemoryDataType,
    pub owner: String,
    pub allocation_tick: u64,
    pub deallocation_tick: u64,
    pub data_hash: u64,
    pub preserved_snippet: Vec<u8>,  // first N bytes preserved for archaeology
    pub cause: String,
}

/// Statistics about the memory tombstone system.
#[derive(Clone, Debug, Default)]
pub struct MemoryTombstoneStats {
    pub total_tombstones: usize,
    pub total_freed_bytes: usize,
    pub instruction_regions: usize,
    pub heap_regions: usize,
    pub oldest_tombstone_age: u64,
}

/// The memory tombstone system that marks freed memory.
pub struct MemoryTombstoneSystem {
    pub tombstones: Vec<MemoryTombstone>,
    next_id: u32,
    max_tombstones: usize,
    snippet_preserve_size: usize,
}

impl MemoryTombstoneSystem {
    pub fn new() -> Self {
        Self {
            tombstones: Vec::new(),
            next_id: 1,
            max_tombstones: 512,
            snippet_preserve_size: 32,
        }
    }

    /// Create a tombstone for a freed memory region.
    pub fn mark_freed(
        &mut self,
        address: u64,
        size: usize,
        data_type: MemoryDataType,
        owner: &str,
        allocation_tick: u64,
        deallocation_tick: u64,
        data_hash: u64,
        snippet: Vec<u8>,
        cause: &str,
    ) -> Result<u32, String> {
        if self.tombstones.len() >= self.max_tombstones {
            return Err("tombstone system is full".to_string());
        }

        let id = self.next_id;
        self.next_id += 1;

        let preserved = snippet.into_iter()
            .take(self.snippet_preserve_size)
            .collect();

        self.tombstones.push(MemoryTombstone {
            address,
            size,
            data_type,
            owner: owner.to_string(),
            allocation_tick,
            deallocation_tick,
            data_hash,
            preserved_snippet: preserved,
            cause: cause.to_string(),
        });

        Ok(id)
    }

    /// Find tombstones by address (exact match).
    pub fn find_at_address(&self, address: u64) -> Vec<&MemoryTombstone> {
        self.tombstones.iter()
            .filter(|t| t.address == address)
            .collect()
    }

    /// Find tombstones in an address range.
    pub fn find_in_range(&self, start: u64, end: u64) -> Vec<&MemoryTombstone> {
        self.tombstones.iter()
            .filter(|t| t.address >= start && (t.address + t.size as u64) <= end)
            .collect()
    }

    /// Find tombstones by owner.
    pub fn find_by_owner(&self, owner: &str) -> Vec<&MemoryTombstone> {
        self.tombstones.iter()
            .filter(|t| t.owner == owner)
            .collect()
    }

    /// Find tombstones by data type.
    pub fn find_by_type(&self, data_type: &MemoryDataType) -> Vec<&MemoryTombstone> {
        self.tombstones.iter()
            .filter(|t| t.data_type == *data_type)
            .collect()
    }

    /// Check if an address was previously occupied.
    pub fn was_occupied(&self, address: u64) -> bool {
        self.tombstones.iter().any(|t| t.address == address)
    }

    /// Get statistics.
    pub fn statistics(&self, current_tick: u64) -> MemoryTombstoneStats {
        let total_freed_bytes: usize = self.tombstones.iter().map(|t| t.size).sum();
        let instruction_regions = self.tombstones.iter()
            .filter(|t| t.data_type == MemoryDataType::Instruction)
            .count();
        let heap_regions = self.tombstones.iter()
            .filter(|t| t.data_type == MemoryDataType::HeapObject)
            .count();
        let oldest_age = self.tombstones.iter()
            .map(|t| current_tick.saturating_sub(t.deallocation_tick))
            .max()
            .unwrap_or(0);

        MemoryTombstoneStats {
            total_tombstones: self.tombstones.len(),
            total_freed_bytes,
            instruction_regions,
            heap_regions,
            oldest_tombstone_age: oldest_age,
        }
    }

    /// Total number of tombstones.
    pub fn len(&self) -> usize {
        self.tombstones.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_freed() {
        let mut mts = MemoryTombstoneSystem::new();
        let id = mts.mark_freed(
            0x1000, 128, MemoryDataType::Instruction,
            "process_a", 10, 100, 42,
            vec![0x01, 0x02, 0x03], "process terminated"
        ).unwrap();
        assert_eq!(id, 1);
        assert_eq!(mts.len(), 1);
    }

    #[test]
    fn test_find_at_address() {
        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 64, MemoryDataType::HeapObject, "p1", 0, 10, 0, vec![], "freed").unwrap();
        mts.mark_freed(0x2000, 32, MemoryDataType::StackFrame, "p1", 0, 20, 0, vec![], "freed").unwrap();
        let found = mts.find_at_address(0x1000);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].data_type, MemoryDataType::HeapObject);
    }

    #[test]
    fn test_find_in_range() {
        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 64, MemoryDataType::Buffer, "p1", 0, 10, 0, vec![], "x").unwrap();
        mts.mark_freed(0x2000, 32, MemoryDataType::Buffer, "p1", 0, 20, 0, vec![], "x").unwrap();
        mts.mark_freed(0x3000, 128, MemoryDataType::Buffer, "p1", 0, 30, 0, vec![], "x").unwrap();
        let in_range = mts.find_in_range(0x1000, 0x2200);
        assert_eq!(in_range.len(), 2);
    }

    #[test]
    fn test_find_by_owner() {
        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 64, MemoryDataType::HeapObject, "agent_a", 0, 10, 0, vec![], "x").unwrap();
        mts.mark_freed(0x2000, 32, MemoryDataType::HeapObject, "agent_b", 0, 20, 0, vec![], "x").unwrap();
        assert_eq!(mts.find_by_owner("agent_a").len(), 1);
    }

    #[test]
    fn test_was_occupied() {
        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 64, MemoryDataType::HeapObject, "a", 0, 10, 0, vec![], "x").unwrap();
        assert!(mts.was_occupied(0x1000));
        assert!(!mts.was_occupied(0x2000));
    }

    #[test]
    fn test_statistics() {
        let mut mts = MemoryTombstoneSystem::new();
        mts.mark_freed(0x1000, 100, MemoryDataType::Instruction, "a", 0, 10, 0, vec![], "x").unwrap();
        mts.mark_freed(0x2000, 200, MemoryDataType::HeapObject, "b", 0, 20, 0, vec![], "x").unwrap();
        let stats = mts.statistics(100);
        assert_eq!(stats.total_tombstones, 2);
        assert_eq!(stats.total_freed_bytes, 300);
        assert_eq!(stats.instruction_regions, 1);
        assert_eq!(stats.heap_regions, 1);
    }

    #[test]
    fn test_snippet_truncation() {
        let mut mts = MemoryTombstoneSystem::new();
        let long_snippet: Vec<u8> = (0..100).collect();
        mts.mark_freed(0x1000, 64, MemoryDataType::Buffer, "a", 0, 10, 0, long_snippet, "x").unwrap();
        assert!(mts.find_at_address(0x1000)[0].preserved_snippet.len() <= 32);
    }
}

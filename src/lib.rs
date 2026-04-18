pub mod afterlife;
pub mod archaeologist;
pub mod artifact_vault;
pub mod cemetery;
pub mod graveyard;
pub mod memorial;
pub mod memory_tombstone;
pub mod necropolis;
pub mod process_graveyard;
pub mod resurrection;
pub mod tombstone;

pub use afterlife::{Afterlife, KnowledgeFragment};
pub use archaeologist::{Archaeologist, ArchaeologyReport, DiscoveredPattern, PatternSeverity};
pub use artifact_vault::{ArtifactVault, ArtifactCategory};
pub use cemetery::{CodeCemetery, CodeStatus, DeadCodeEntry};
pub use graveyard::Graveyard;
pub use memorial::{MemorialLog, MemorialVisit};
pub use memory_tombstone::{MemoryTombstoneSystem, MemoryDataType, MemoryTombstone};
pub use necropolis::{Necropolis, NecropolisStats};
pub use process_graveyard::{ProcessGraveyard, TerminationReason, ProcessStatus, ProcessRecord};
pub use resurrection::{ResurrectionProtocol, ResurrectionCriteria, ResurrectionStatus, ResurrectionCandidate, ResurrectionRecord};
pub use tombstone::{Tombstone, VesselState};

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Original tests ───

    // 1. tombstone new defaults
    #[test]
    fn tombstone_new_defaults() {
        let t = Tombstone::new(1, "TestVessel");
        assert_eq!(t.vessel_id, 1);
        assert_eq!(t.name, "TestVessel");
        assert_eq!(t.state, VesselState::Alive);
        assert!(t.cause.is_empty());
        assert!(t.lesson.is_empty());
        assert_eq!(t.commits_made, 0);
        assert_eq!(t.peak_trust, 0.0);
    }

    // 2. tombstone mark_dead
    #[test]
    fn tombstone_mark_dead() {
        let mut t = Tombstone::new(2, "Vessel2");
        t.birth_time = 100;
        t.mark_dead(200);
        assert_eq!(t.state, VesselState::Dead);
        assert_eq!(t.death_time, 200);
        assert_eq!(t.cycles_lived, 100);
        assert!((t.lifetime_days() - 100.0 / 86400.0).abs() < f64::EPSILON);
    }

    // 3. graveyard bury and find
    #[test]
    fn graveyard_bury_and_find() {
        let mut gy = Graveyard::new();
        let t = Tombstone::new(10, "Vessel10");
        let idx = gy.bury(t).unwrap();
        assert_eq!(idx, 0);
        assert!(gy.find(10).is_some());
        assert!(gy.find(99).is_none());
    }

    // 4. graveyard find_name
    #[test]
    fn graveyard_find_name() {
        let mut gy = Graveyard::new();
        gy.bury(Tombstone::new(1, "Alpha")).unwrap();
        gy.bury(Tombstone::new(2, "Beta")).unwrap();
        assert!(gy.find_name("Alpha").is_some());
        assert!(gy.find_name("Gamma").is_none());
    }

    // 5. graveyard counts
    #[test]
    fn graveyard_counts() {
        let mut gy = Graveyard::new();
        let mut t1 = Tombstone::new(1, "A");
        t1.state = VesselState::Dead;
        let mut t2 = Tombstone::new(2, "B");
        t2.state = VesselState::Memorialized;
        let mut t3 = Tombstone::new(3, "C");
        t3.state = VesselState::Harvested;
        gy.bury(t1).unwrap();
        gy.bury(t2).unwrap();
        gy.bury(t3).unwrap();
        assert_eq!(gy.count_dead(), 1);
        assert_eq!(gy.count_memorialized(), 1);
        assert_eq!(gy.count_harvested(), 1);
    }

    // 6. graveyard recent_deaths
    #[test]
    fn graveyard_recent_deaths() {
        let mut gy = Graveyard::new();
        let mut t1 = Tombstone::new(1, "A");
        t1.death_time = 100;
        let mut t2 = Tombstone::new(2, "B");
        t2.death_time = 200;
        gy.bury(t1).unwrap();
        gy.bury(t2).unwrap();
        let recent = gy.recent_deaths(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].vessel_id, 2);
    }

    // 7. graveyard lessons
    #[test]
    fn graveyard_lessons() {
        let mut gy = Graveyard::new();
        let mut t = Tombstone::new(1, "A");
        t.state = VesselState::Dead;
        t.lesson = "always check types".to_string();
        gy.bury(t).unwrap();
        let lessons = gy.lessons();
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0], "always check types");
    }

    // 8. graveyard vessels_by_trust
    #[test]
    fn graveyard_vessels_by_trust() {
        let mut gy = Graveyard::new();
        let mut t1 = Tombstone::new(1, "A");
        t1.peak_trust = 0.5;
        let mut t2 = Tombstone::new(2, "B");
        t2.peak_trust = 0.9;
        gy.bury(t1).unwrap();
        gy.bury(t2).unwrap();
        let sorted = gy.vessels_by_trust();
        assert_eq!(sorted[0].vessel_id, 2);
        assert_eq!(sorted[1].vessel_id, 1);
    }

    // 9. afterlife store and find
    #[test]
    fn afterlife_store_and_find() {
        let mut al = Afterlife::new();
        al.store("key1", "value1", 1).unwrap();
        assert!(al.find("key1").is_some());
        assert_eq!(al.find("key1").unwrap().value, "value1");
        assert!(al.find("missing").is_none());
    }

    // 10. afterlife search prefix
    #[test]
    fn afterlife_search_prefix() {
        let mut al = Afterlife::new();
        al.store("rust:tip1", "use iterators", 1).unwrap();
        al.store("rust:tip2", "clippy is good", 1).unwrap();
        al.store("go:tip1", "handle errors", 2).unwrap();
        let results = al.search("rust:", 10);
        assert_eq!(results.len(), 2);
    }

    // 11. afterlife reuse increment
    #[test]
    fn afterlife_reuse_increment() {
        let mut al = Afterlife::new();
        al.store("key1", "val1", 1).unwrap();
        al.increment_reuse("key1");
        al.increment_reuse("key1");
        assert_eq!(al.find("key1").unwrap().reused_count, 2);
    }

    // 12. afterlife most_reused
    #[test]
    fn afterlife_most_reused() {
        let mut al = Afterlife::new();
        al.store("a", "1", 1).unwrap();
        al.store("b", "2", 1).unwrap();
        al.increment_reuse("b");
        al.increment_reuse("b");
        al.increment_reuse("b");
        let top = al.most_reused(1);
        assert_eq!(top[0].key, "b");
    }

    // 13. afterlife harvest from tombstone
    #[test]
    fn afterlife_harvest_from_tombstone() {
        let mut al = Afterlife::new();
        let mut t = Tombstone::new(5, "Harvested");
        t.lesson = "don't panic".to_string();
        al.harvest(&t);
        assert!(al.find("v:5:lesson").is_some());
        assert_eq!(al.find("v:5:lesson").unwrap().value, "don't panic");
    }

    // 14. memorial record and counts
    #[test]
    fn memorial_record_and_counts() {
        let mut ml = MemorialLog::new();
        ml.record(1, 0, 2);
        ml.record(1, 0, 1);
        ml.record(2, 0, 1);
        assert_eq!(ml.visits_to(0), 3);
        assert_eq!(ml.visits_by(1), 2);
        let top = ml.most_visited(1);
        assert_eq!(top[0], (0, 3));
    }

    // 15. necropolis bury auto-harvests
    #[test]
    fn necropolis_bury_auto_harvests() {
        let mut nec = Necropolis::new();
        let mut t = Tombstone::new(1, "Auto");
        t.mark_dead(100);
        t.set_lesson("automated wisdom");
        nec.bury(t).unwrap();
        assert!(nec.afterlife.find("v:1:lesson").is_some());
    }

    // 16. necropolis visit
    #[test]
    fn necropolis_visit() {
        let mut nec = Necropolis::new();
        let mut t = Tombstone::new(1, "Visited");
        t.mark_dead(50);
        nec.bury(t).unwrap();
        assert!(nec.visit(10, 1, 3));
        assert!(!nec.visit(10, 99, 1)); // non-existent
    }

    // 17. necropolis statistics
    #[test]
    fn necropolis_statistics() {
        let mut nec = Necropolis::new();
        let mut t = Tombstone::new(1, "Stats");
        t.mark_dead(100);
        t.set_lesson("stats lesson");
        nec.bury(t).unwrap();
        nec.visit(5, 1, 1);
        let stats = nec.statistics();
        assert_eq!(stats.total_buried, 1);
        assert_eq!(stats.total_knowledge, 1);
        assert_eq!(stats.total_memorial_visits, 1);
    }

    // 18. graveyard full error
    #[test]
    fn graveyard_full_error() {
        let mut gy = Graveyard::new();
        for i in 0..64 {
            gy.bury(Tombstone::new(i as u16, &format!("v{}", i))).unwrap();
        }
        let result = gy.bury(Tombstone::new(99, "overflow"));
        assert!(result.is_err());
    }

    // ─── Integration tests ───

    #[test]
    fn test_full_necropolis_workflow() {
        let mut arch = Archaeologist::new();
        let mut cem = CodeCemetery::new();
        let mut mts = MemoryTombstoneSystem::new();
        let mut pg = ProcessGraveyard::new();
        let mut vault = ArtifactVault::new();

        // Register and bury code in same module (triggers clustering pattern)
        let code_id = cem.register("critical_fn", "engine", 42, 512, 0);
        for i in 0..10 { cem.touch(code_id, i); }
        cem.mark_suspect(code_id);
        cem.bury(code_id, 100, "optimization removed").unwrap();
        let code_id2 = cem.register("helper_fn", "engine", 99, 128, 0);
        cem.bury(code_id2, 100, "dead code").unwrap();

        // Mark freed memory from same owner (triggers leak pattern)
        mts.mark_freed(0x1000, 256, MemoryDataType::Instruction, "process_1", 0, 50, 12345,
            vec![0xDE, 0xAD, 0xBE, 0xEF], "process killed").unwrap();
        mts.mark_freed(0x2000, 128, MemoryDataType::HeapObject, "process_1", 0, 60, 54321,
            vec![], "leaked").unwrap();
        mts.mark_freed(0x3000, 64, MemoryDataType::Buffer, "process_1", 0, 70, 99999,
            vec![], "leaked").unwrap();

        // Register and terminate processes including a crash (triggers crash pattern)
        let proc_id = pg.register("worker_1", 5, 0);
        pg.add_error(proc_id, "resource exhausted");
        pg.terminate(proc_id, 80, TerminationReason::TimedOut, 10000, 2048, 50000).unwrap();
        pg.set_output(proc_id, "partial results: 42 items processed");

        let crash_id = pg.register("crasher", 6, 0);
        pg.terminate(crash_id, 90, TerminationReason::Crash("segfault".to_string()), 5000, 1024, 30000).unwrap();

        // Preserve artifacts
        vault.preserve("partial_results", ArtifactCategory::KnowledgeBase, 5, 1024, 99, 80, 0.8, "sha256").unwrap();
        vault.add_metadata(1, "items_processed", "42");

        // Evaluate resurrection candidates
        let mut rp = ResurrectionProtocol::new();
        rp.evaluate_code_cemetery(&cem, 200);
        rp.evaluate_process_graveyard(&pg, 200);
        let ranked = rp.ranked_candidates();
        assert!(!ranked.is_empty());

        // Archaeological analysis
        let report = arch.analyze(&cem, &mts, &pg, &vault, 200);
        assert_eq!(report.total_dead_code_entries, 2);
        assert!(report.total_freed_memory_bytes > 0);
        assert!(!report.patterns.is_empty());
    }

    #[test]
    fn test_resurrection_with_archaeology() {
        let mut arch = Archaeologist::new();
        let mut cem = CodeCemetery::new();
        let mut pg = ProcessGraveyard::new();
        let vault = ArtifactVault::new();

        // Bury some useful code
        for i in 0..5 {
            let id = cem.register(&format!("fn_{}", i), "utils", i as u64, 64, 0);
            cem.bury(id, 100, "dead code cleanup").unwrap();
        }

        // Terminate a completed process
        let proc_id = pg.register("batch_worker", 10, 0);
        pg.terminate(proc_id, 500, TerminationReason::Completed, 50000, 4096, 100000).unwrap();

        // Analyze
        arch.analyze(&cem, &MemoryTombstoneSystem::new(), &pg, &vault, 1000);

        // Evaluate for resurrection
        let mut rp = ResurrectionProtocol::new();
        rp.evaluate_code_cemetery(&cem, 1000);
        rp.evaluate_process_graveyard(&pg, 1000);

        // Accept a candidate and complete resurrection
        if !rp.ranked_candidates().is_empty() {
            let mut record = rp.accept(0).unwrap();
            rp.complete_resurrection(&mut record, 25, 2);
            assert_eq!(rp.succeeded_count, 1);
        }
    }
}

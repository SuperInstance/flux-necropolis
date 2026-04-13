# flux-necropolis

> *Fleet graveyard — vessels die, knowledge persists.*

A Rust crate for tracking the lifecycle of autonomous fleet vessels: their birth, service, death, and the wisdom extracted from their existence.

## What This Is

`flux-necropolis` provides the **afterlife infrastructure** for the FLUX fleet — when an agent vessel reaches end-of-life, its tombstone records peak trust, commits made, lessons learned, and cause of death. Knowledge is automatically harvested into an afterlife knowledge store where living agents can search and reuse it.

## Role in the FLUX Ecosystem

The necropolis embodies a core FLUX principle: nothing is wasted.

- **`flux-trust`** records peak trust scores that survive on tombstones
- **`flux-grimoire`** builds curricula from necropolis-harvested lessons
- **`flux-evolve`** uses posthumous lessons to avoid repeating failed strategies
- **`flux-social`** adjusts relationships when a vessel is memorialized
- **`flux-simulator`** models vessel lifecycle including death and knowledge extraction
- **`flux-memory`** provides the ephemeral store; necropolis provides the permanent one

## Modules

| Module | Purpose |
|--------|---------|
| `tombstone` | Individual vessel lifecycle record (birth, death, lessons, trust) |
| `graveyard` | Collection of tombstones (max 64) with search and ranking |
| `afterlife` | Harvested knowledge fragments (max 128) with reuse tracking |
| `memorial` | Visitor interaction log (who visited which grave) |
| `necropolis` | Unified graveyard + afterlife + memorial with auto-harvest |

## Lifecycle

```
Alive → Dying → Dead → Memorialized
                  ↘ Harvested
```

When a vessel is buried in the Necropolis, its lesson is automatically harvested into the Afterlife.

## Quick Start

```rust
use flux_necropolis::{Necropolis, Tombstone};

let mut necropolis = Necropolis::new();

let mut vessel = Tombstone::new(1, "Explorer-7");
vessel.birth_time = 1700000000;
vessel.peak_trust = 0.92;
vessel.set_lesson("Always verify before commit");

vessel.mark_dead(1700086400);
let idx = necropolis.bury(vessel).unwrap();

// Knowledge auto-harvested
let wisdom = necropolis.search_wisdom("v:1:");

// Visit the grave
necropolis.visit(2, 1, 3);

// Statistics
let stats = necropolis.statistics();
println!("Buried: {}, Knowledge: {}", stats.total_buried, stats.total_knowledge);
```

## Key Features

| Feature | Description |
|---------|-------------|
| **Auto-Harvest** | Lessons automatically extracted on burial |
| **Trust Ranking** | `vessels_by_trust()` ranks tombstones by peak trust |
| **Reuse Tracking** | `increment_reuse()` tracks how often knowledge is reused |
| **Prefix Search** | Search harvested knowledge by key prefix |
| **Visit Logging** | Track which living agents visit which graves |
| **Capacity Limits** | Graveyard (64), Afterlife (128) with error on overflow |

## Building & Testing

```bash
cargo build
cargo test
```

## Related Fleet Repos

- [`flux-grimoire`](https://github.com/SuperInstance/flux-grimoire) — Curriculum built from harvested lessons
- [`flux-trust`](https://github.com/SuperInstance/flux-trust) — Trust scoring (peak values recorded on tombstones)
- [`flux-evolve`](https://github.com/SuperInstance/flux-evolve) — Behavioral evolution informed by posthumous lessons
- [`flux-social`](https://github.com/SuperInstance/flux-social) — Social graph adjustments on vessel death
- [`flux-memory`](https://github.com/SuperInstance/flux-memory) — Ephemeral in-memory store (necropolis is permanent)

## License

Part of the [SuperInstance](https://github.com/SuperInstance) FLUX fleet.

//! Pure, UI-free, process-free logic. Everything here is deterministic and unit-tested;
//! the runtime (`crate::media`) only spawns what these functions describe.

pub mod encode;
pub mod models;
pub mod naming;
pub mod planner;
pub mod presets;
pub mod preview;
pub mod probe;
pub mod progress;

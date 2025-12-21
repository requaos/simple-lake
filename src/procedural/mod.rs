pub mod library;
pub mod generator;
pub mod text_assembly;
pub mod stat_calculator;
pub mod risk_calculator;

pub use library::{SituationLibrary, EventDomain};
pub use generator::generate_procedural_event;
pub use risk_calculator::PlayerStats;

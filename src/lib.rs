/// Core agent loop and session tree
pub mod core;

/// Tool interface — all agent capabilities implement Tool
pub mod tools;

/// Memory store and recall (LanceDB — ADR-0003)
pub mod memory;

/// Messaging surface abstraction (ADR-0003)
pub mod messaging;

/// Cortex supervisor — process health, stuck detection, session routing
pub mod cortex;

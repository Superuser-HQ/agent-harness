/// Cortex supervisor — v1 scope (PRD §4.9, ADR decisions)
///
/// v1 Cortex does ONLY:
///   1. Process supervision — stuck worker detection, retries, kill policies
///   2. Health signals — queue depth, failure rate, memory sync lag, liveness
///
/// Explicitly NOT in v1 Cortex:
///   - Pattern mining
///   - Memory bulletins
///   - Admin chat interface
///   - Cross-session learning
///   - Predictive routing
///
/// SLOs enforced by Cortex (from PRD §7):
///   - Stuck worker detection: <= 60s median, <= 5m max cleanup
///   - Process crash-free: >= 99.5% daily uptime

use std::time::Duration;

/// Cortex supervisor handle
pub struct Cortex {
    // TODO: Phase 1 — hold session registry, health state
    stuck_timeout: Duration,
}

impl Cortex {
    pub fn new() -> Self {
        Self {
            // PRD SLO: ≤60s median detection — conservative default
            stuck_timeout: Duration::from_secs(60),
        }
    }

    /// Start the supervision loop
    /// Runs as a background task alongside the main agent loop
    pub async fn run(&self) {
        // TODO: Phase 1
        // - Poll session registry for stuck workers
        // - Emit health signals on configurable interval
        // - Trigger cleanup/retry per kill policy
        tracing::info!("Cortex supervisor starting (stuck_timeout={:?})", self.stuck_timeout);
        todo!("Cortex::run not yet implemented")
    }

    /// Emit a health snapshot to stdout/logs
    /// Called by `superagents health` command
    pub async fn health_snapshot(&self) -> HealthSnapshot {
        // TODO: Phase 1 — compute from live state
        HealthSnapshot {
            active_sessions: 0,
            stuck_sessions: 0,
            memory_export_lag_secs: None,
        }
    }
}

impl Default for Cortex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct HealthSnapshot {
    pub active_sessions: usize,
    pub stuck_sessions: usize,
    /// None if no export has run yet
    pub memory_export_lag_secs: Option<u64>,
}

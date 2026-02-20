use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session kinds — main session has no parent, branch sessions are task-scoped
///
/// Session tree:
///   Main
///   └── Branch (spawned for background tasks, deleted after returning results)
///       └── Branch (nested, if needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionKind {
    /// Primary interactive session — persistent, routes user messages
    Main,
    /// Scoped task session — spawned by Main or another Branch,
    /// writes results back to parent on completion, then is pruned
    /// (audit trail is preserved before deletion — see ADR-0005)
    Branch { parent: SessionId },
}

/// A running or completed agent session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub kind: SessionKind,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed { reason: String },
}

impl Session {
    pub fn new_main() -> Self {
        Self {
            id: SessionId::new(),
            kind: SessionKind::Main,
            status: SessionStatus::Active,
        }
    }

    pub fn new_branch(parent: SessionId) -> Self {
        Self {
            id: SessionId::new(),
            kind: SessionKind::Branch { parent },
            status: SessionStatus::Active,
        }
    }

    pub fn is_main(&self) -> bool {
        matches!(self.kind, SessionKind::Main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_session_is_main() {
        let s = Session::new_main();
        assert!(s.is_main());
    }

    #[test]
    fn branch_session_is_not_main() {
        let parent = SessionId::new();
        let s = Session::new_branch(parent);
        assert!(!s.is_main());
    }

    #[test]
    fn branch_has_correct_parent() {
        let parent_id = SessionId::new();
        let s = Session::new_branch(parent_id.clone());
        match &s.kind {
            SessionKind::Branch { parent } => assert_eq!(parent, &parent_id),
            _ => panic!("expected Branch"),
        }
    }
}

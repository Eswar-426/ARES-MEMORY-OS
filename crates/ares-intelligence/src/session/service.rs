use super::manager::SessionManager;
use super::state::{IntelligenceSession, SessionType};

pub struct SessionService {
    manager: SessionManager,
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new(SessionManager::default())
    }
}

impl SessionService {
    #[allow(dead_code)]
    pub fn new(manager: SessionManager) -> Self {
        Self { manager }
    }

    #[allow(dead_code)]
    pub fn start_collaboration_session(&self) -> IntelligenceSession {
        self.manager.create_session(SessionType::Collaboration)
    }

    #[allow(dead_code)]
    pub fn get_session_state(&self, id: &str) -> anyhow::Result<IntelligenceSession> {
        self.manager
            .get_session(id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", id))
    }
}

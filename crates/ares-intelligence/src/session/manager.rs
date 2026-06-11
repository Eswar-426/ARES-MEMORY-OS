use super::state::{IntelligenceSession, SessionStatus, SessionType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, IntelligenceSession>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn create_session(&self, session_type: SessionType) -> IntelligenceSession {
        let session = IntelligenceSession {
            session_id: uuid::Uuid::now_v7().to_string(),
            session_type,
            status: SessionStatus::Active,
            context_id: None,
        };

        let mut lock = self.sessions.write().unwrap();
        lock.insert(session.session_id.clone(), session.clone());
        session
    }

    #[allow(dead_code)]
    pub fn get_session(&self, id: &str) -> Option<IntelligenceSession> {
        let lock = self.sessions.read().unwrap();
        lock.get(id).cloned()
    }
}

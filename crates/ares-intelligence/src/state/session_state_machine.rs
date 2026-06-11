use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Active,
    Suspended,
    AwaitingApproval,
    Resumed,
    Completed,
    Terminated,
}

pub struct SessionStateMachine {
    current_state: SessionState,
}

impl SessionStateMachine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current_state: SessionState::Created,
        }
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStateMachine {
    #[allow(dead_code)]
    pub fn transition(&mut self, new_state: SessionState) -> anyhow::Result<()> {
        use SessionState::*;
        let valid = matches!(
            (self.current_state, new_state),
            (Created, Active)
                | (Active, Suspended)
                | (Active, AwaitingApproval)
                | (Active, Completed)
                | (Active, Terminated)
                | (Suspended, Resumed)
                | (Suspended, Terminated)
                | (AwaitingApproval, Resumed)
                | (AwaitingApproval, Terminated)
                | (Resumed, Active)
        );

        if valid {
            self.current_state = new_state;
            Ok(())
        } else {
            anyhow::bail!(
                "Invalid transition from {:?} to {:?}",
                self.current_state,
                new_state
            )
        }
    }

    #[allow(dead_code)]
    pub fn state(&self) -> SessionState {
        self.current_state
    }
}

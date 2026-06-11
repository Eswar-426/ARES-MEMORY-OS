use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentState {
    Draft,
    Running,
    Paused,
    Analyzing,
    Completed,
    Failed,
}

pub struct ExperimentStateMachine {
    current_state: ExperimentState,
}

impl ExperimentStateMachine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current_state: ExperimentState::Draft,
        }
    }
}

impl Default for ExperimentStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentStateMachine {
    #[allow(dead_code)]
    pub fn transition(&mut self, new_state: ExperimentState) -> anyhow::Result<()> {
        use ExperimentState::*;
        let valid = matches!(
            (self.current_state, new_state),
            (Draft, Running)
                | (Running, Paused)
                | (Running, Analyzing)
                | (Running, Failed)
                | (Paused, Running)
                | (Paused, Failed)
                | (Analyzing, Completed)
                | (Analyzing, Failed)
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
    pub fn state(&self) -> ExperimentState {
        self.current_state
    }
}

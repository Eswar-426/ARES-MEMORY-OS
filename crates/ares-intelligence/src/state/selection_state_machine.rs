use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionState {
    Pending,
    Analyzing,
    Selecting,
    Selected,
    Executing,
    Evaluated,
    Completed,
    Failed,
}

pub struct SelectionStateMachine {
    current_state: SelectionState,
}

impl SelectionStateMachine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current_state: SelectionState::Pending,
        }
    }
}

impl Default for SelectionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionStateMachine {
    #[allow(dead_code)]
    pub fn transition(&mut self, new_state: SelectionState) -> anyhow::Result<()> {
        use SelectionState::*;
        let valid = matches!(
            (self.current_state, new_state),
            (Pending, Analyzing)
                | (Analyzing, Selecting)
                | (Analyzing, Failed)
                | (Selecting, Selected)
                | (Selecting, Failed)
                | (Selected, Executing)
                | (Executing, Evaluated)
                | (Executing, Failed)
                | (Evaluated, Completed)
                | (Failed, Pending)
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
    pub fn state(&self) -> SelectionState {
        self.current_state
    }
}

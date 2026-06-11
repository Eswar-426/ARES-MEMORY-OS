#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorState {
    Created,
    Analyzed,
    Selected,
    Routed,
    Evaluated,
    Learned,
    Synced,
    Failed,
}

pub struct CoordinatorStateMachine {
    current_state: CoordinatorState,
}

impl Default for CoordinatorStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinatorStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: CoordinatorState::Created,
        }
    }

    pub fn current_state(&self) -> CoordinatorState {
        self.current_state
    }

    pub fn transition_to(&mut self, next_state: CoordinatorState) -> anyhow::Result<()> {
        let valid = match (self.current_state, next_state) {
            (CoordinatorState::Created, CoordinatorState::Analyzed) => true,
            (CoordinatorState::Analyzed, CoordinatorState::Selected) => true,
            (CoordinatorState::Selected, CoordinatorState::Routed) => true,
            (CoordinatorState::Routed, CoordinatorState::Evaluated) => true,
            (CoordinatorState::Evaluated, CoordinatorState::Learned) => true,
            (CoordinatorState::Learned, CoordinatorState::Synced) => true,
            // Any state can go to Failed
            (_, CoordinatorState::Failed) => true,
            _ => false,
        };

        if valid {
            self.current_state = next_state;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Invalid transition from {:?} to {:?}",
                self.current_state,
                next_state
            ))
        }
    }
}

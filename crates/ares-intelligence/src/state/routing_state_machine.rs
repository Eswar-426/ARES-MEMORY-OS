use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingState {
    Init,
    ProviderSelecting,
    Routing,
    Routed,
    Executing,
    FallbackTriggered,
    Completed,
    Failed,
}

pub struct RoutingStateMachine {
    current_state: RoutingState,
}

impl RoutingStateMachine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current_state: RoutingState::Init,
        }
    }
}

impl Default for RoutingStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingStateMachine {
    #[allow(dead_code)]
    pub fn transition(&mut self, new_state: RoutingState) -> anyhow::Result<()> {
        use RoutingState::*;
        let valid = matches!(
            (self.current_state, new_state),
            (Init, ProviderSelecting)
                | (ProviderSelecting, Routing)
                | (ProviderSelecting, Failed)
                | (Routing, Routed)
                | (Routing, FallbackTriggered)
                | (Routed, Executing)
                | (Executing, Completed)
                | (Executing, FallbackTriggered)
                | (Executing, Failed)
                | (FallbackTriggered, Routing)
                | (FallbackTriggered, Failed)
                | (Failed, Init)
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
    pub fn state(&self) -> RoutingState {
        self.current_state
    }
}

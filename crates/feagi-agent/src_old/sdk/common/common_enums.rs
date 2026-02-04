#[derive(Default, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum AgentConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Authenticating,
    Running,
    Reconnecting,
}

impl AgentConnectionState {
    pub fn change_state_and_return_before_and_after(
        current_state_var: &mut AgentConnectionState,
        new_state: AgentConnectionState,
    ) -> Option<(AgentConnectionState, AgentConnectionState)> {
        if *current_state_var == new_state {
            return None;
        }
        let prior_state = *current_state_var;
        *current_state_var = new_state;
        Some((prior_state, new_state))
    }

    pub fn is_active(&self) -> bool {
        *self != AgentConnectionState::Disconnected
    }
}



#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub enum FeagiConnectionConfiguration {
    NeuroRoboticsStudio,
    DummyTesting,
    ZMQDirect { host: String },
    WebsocketDirect { host: String },
}

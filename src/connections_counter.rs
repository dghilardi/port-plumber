use std::time::SystemTime;

enum CounterState {
    HasConnections { count: usize },
    NoConnections { since: SystemTime },
}

pub struct ConnectionCounter {
    state: CounterState,
}

impl ConnectionCounter {
    pub fn new() -> Self {
        Self {
            state: CounterState::NoConnections { since: SystemTime::now() },
        }
    }

    pub fn add_connection(&mut self) {
        self.state = match self.state {
            CounterState::HasConnections { count } => CounterState::HasConnections { count: count + 1 },
            CounterState::NoConnections { .. } => CounterState::HasConnections { count: 1 },
        }
    }

    pub fn rem_connection(&mut self) {
        self.state = match self.state {
            CounterState::HasConnections { count } if count > 1 => CounterState::HasConnections { count: count - 1 },
            CounterState::HasConnections { .. } => CounterState::NoConnections { since: SystemTime::now() },
            CounterState::NoConnections { .. } => panic!("Trying to remove connections but no one was found"),
        }
    }

    pub fn no_connections_since(&self) -> Option<SystemTime> {
        match &self.state {
            CounterState::HasConnections { .. } => None,
            CounterState::NoConnections { since } => Some(since.clone()),
        }
    }
}


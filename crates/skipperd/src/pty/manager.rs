use crate::state::SharedState;

pub struct PtyManager;

impl PtyManager {
    pub async fn increment_active_sessions(state: &SharedState) {
        let mut guard = state.write().await;
        guard.active_sessions_count += 1;
    }

    pub async fn decrement_active_sessions(state: &SharedState) {
        let mut guard = state.write().await;
        if guard.active_sessions_count > 0 {
            guard.active_sessions_count -= 1;
        }
    }
}

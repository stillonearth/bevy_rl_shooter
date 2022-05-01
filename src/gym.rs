use crossbeam_channel::*;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rl::state::AIGymState;
use heron::*;

use crate::{actions::*, app_states::*};

pub(crate) fn execute_reset_request(
    mut app_state: ResMut<State<AppState>>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    let reset_channel_rx: Receiver<bool>;
    {
        let ai_gym_state = ai_gym_state.lock().unwrap();
        reset_channel_rx = ai_gym_state.__reset_channel_rx.clone();
    }

    if reset_channel_rx.is_empty() {
        return;
    }

    reset_channel_rx.recv().unwrap();
    {
        let mut ai_gym_state = ai_gym_state.lock().unwrap();
        ai_gym_state.is_terminated = true;
    }
    physics_time.resume();
    app_state.set(AppState::Reset).unwrap();
}

pub(crate) fn restart_round(
    mut app_state: ResMut<State<AppState>>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    ai_gym_state.is_terminated = false;
    ai_gym_state.rewards = Vec::new();

    app_state.set(AppState::InGame).unwrap();
}

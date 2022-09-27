use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rl::{state::AIGymState, AIGymSettings};
use heron::*;

use crate::{actions::*, actors::*, app_states::*, control::*, events::*};

pub(crate) fn execute_reset_request(
    mut app_state: ResMut<State<AppState>>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();
    if !ai_gym_state.is_reset_request() {
        return;
    }

    ai_gym_state.receive_reset_request();
    app_state.set(AppState::Reset).unwrap();
}

pub(crate) fn turnbased_control_system_switch(
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<DelayedControlTimer>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    ai_gym_settings: Res<AIGymSettings>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.overwrite_push(AppState::Control).unwrap();
        physics_time.pause();

        {
            let ai_gym_state = ai_gym_state.lock().unwrap();
            let results = (0..ai_gym_settings.num_agents).map(|_| true).collect();
            ai_gym_state.send_step_result(results);
        }
    }
}

pub(crate) fn execute_step_request(
    agent_movement_q: Query<(&mut heron::prelude::Velocity, &mut Transform, &Actor)>,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    ai_gym_settings: Res<AIGymSettings>,
    mut app_state: ResMut<State<AppState>>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();

    if !ai_gym_state.is_next_action() {
        return;
    }

    let unparsed_actions = ai_gym_state.receive_action_strings();
    let mut actions: Vec<Option<PlayerActionFlags>> =
        (0..ai_gym_settings.num_agents).map(|_| None).collect();

    for i in 0..unparsed_actions.len() {
        let unparsed_action = unparsed_actions[i].clone();
        ai_gym_state.set_reward(i, 0.0);

        if unparsed_action.is_none() {
            actions[i] = None;
            continue;
        }

        let action = match unparsed_action.unwrap().as_str() {
            "FORWARD" => Some(PlayerActionFlags::FORWARD),
            "BACKWARD" => Some(PlayerActionFlags::BACKWARD),
            "LEFT" => Some(PlayerActionFlags::LEFT),
            "RIGHT" => Some(PlayerActionFlags::RIGHT),
            "TURN_LEFT" => Some(PlayerActionFlags::TURN_LEFT),
            "TURN_RIGHT" => Some(PlayerActionFlags::TURN_RIGHT),
            "SHOOT" => Some(PlayerActionFlags::SHOOT),
            _ => None,
        };

        actions[i] = action;
    }

    physics_time.resume();
    control_agents(actions, agent_movement_q, collision_events, event_gun_shot);

    app_state.pop().unwrap();
}

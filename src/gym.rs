use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rl::state::AIGymState;
use heron::*;

use crate::{actions::*, actors::Actor, actors::*, app_states::*, control::*, events::*};

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
    mut physics_time: ResMut<PhysicsTime>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.overwrite_push(AppState::Control).unwrap();
        physics_time.pause();

        let ai_gym_state = ai_gym_state.lock().unwrap();
        ai_gym_state.send_step_result(true);
    }
}

pub(crate) fn turnbased_text_control_system(
    player_movement_q: Query<
        (&mut heron::prelude::Velocity, &mut Transform),
        With<PlayerPerspective>,
    >,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut app_state: ResMut<State<AppState>>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();

    if !ai_gym_state.is_next_action() {
        return;
    }

    let unparsed_action = ai_gym_state.receive_action_string();

    if unparsed_action == "" {
        ai_gym_state.send_step_result(false);
        return;
    }

    let action = match unparsed_action.as_str() {
        "FORWARD" => Some(PlayerActionFlags::FORWARD),
        "BACKWARD" => Some(PlayerActionFlags::BACKWARD),
        "LEFT" => Some(PlayerActionFlags::LEFT),
        "RIGHT" => Some(PlayerActionFlags::RIGHT),
        "TURN_LEFT" => Some(PlayerActionFlags::TURN_LEFT),
        "TURN_RIGHT" => Some(PlayerActionFlags::TURN_RIGHT),
        "SHOOT" => Some(PlayerActionFlags::SHOOT),
        _ => None,
    };

    if action.is_none() {
        ai_gym_state.send_step_result(false);
        return;
    }

    ai_gym_state.set_score(0.0);

    physics_time.resume();

    control_player(
        action.unwrap(),
        player_movement_q,
        collision_events,
        event_gun_shot,
    );

    app_state.pop().unwrap();
}

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rl::{state::AIGymState, AIGymSettings};

use serde::Serialize;

use crate::{actions::*, actors::*, app_states::*, control::*, events::*, level::GameMap};

#[derive(Default, Serialize, Clone)]
pub struct EnvironmentState {
    pub map: GameMap,
    pub actors: Vec<Actor>,
}

pub(crate) fn execute_reset_request(
    mut app_state: ResMut<State<AppState>>,
    ai_gym_state: ResMut<AIGymState<PlayerActionFlags, EnvironmentState>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();
    if !ai_gym_state.is_reset_request() {
        return;
    }

    ai_gym_state.receive_reset_request();
    app_state.set(AppState::Reset).unwrap();
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn turnbased_control_system_switch(
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<DelayedControlTimer>,
    ai_gym_state: ResMut<AIGymState<PlayerActionFlags, EnvironmentState>>,
    ai_gym_settings: Res<AIGymSettings>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    actor_query: Query<(Entity, &Actor)>,
    game_map: Res<GameMap>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.overwrite_push(AppState::Control).unwrap();
        rapier_configuration.physics_pipeline_active = false;

        {
            let mut ai_gym_state = ai_gym_state.lock().unwrap();
            let results = (0..ai_gym_settings.num_agents).map(|_| true).collect();
            ai_gym_state.send_step_result(results);

            let env_state = EnvironmentState {
                map: game_map.clone(),
                actors: actor_query.iter().map(|(_, a)| a.clone()).collect(),
            };

            ai_gym_state.set_env_state(env_state);
        }
    }
}

pub(crate) fn execute_step_request(
    agent_movement_q: Query<(&mut Velocity, &mut Transform, &Actor)>,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    ai_gym_state: ResMut<AIGymState<PlayerActionFlags, EnvironmentState>>,
    ai_gym_settings: Res<AIGymSettings>,
    mut app_state: ResMut<State<AppState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
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

    rapier_configuration.physics_pipeline_active = true;
    control_agents(actions, agent_movement_q, collision_events, event_gun_shot);

    app_state.pop().unwrap();
}

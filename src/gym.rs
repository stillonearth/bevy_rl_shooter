use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rl::*;

use serde::Serialize;

use crate::{actions::*, actors::*, control::*, events::*, level::*};

#[derive(Default, Serialize, Clone)]
pub(crate) struct EnvironmentState {
    pub(crate) map: GameMap,
    pub(crate) actors: Vec<Actor>,
}

/// Handle bevy_rl::EventPauseResume
pub(crate) fn bevy_rl_pause_request(
    mut pause_event_reader: EventReader<EventPause>,
    ai_gym_state: Res<AIGymState<Actions, EnvironmentState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    game_map: Res<GameMap>,
    query_actors: Query<(Entity, &Actor)>,
) {
    if pause_event_reader.iter().count() == 0 {
        return;
    }

    let _ = pause_event_reader.iter().last();
    // Pause simulation (physics engine)
    rapier_configuration.physics_pipeline_active = false;
    // Collect state into serializable struct
    let env_state = EnvironmentState {
        map: game_map.clone(),
        actors: query_actors.iter().map(|(_, a)| a.clone()).collect(),
    };
    // Set bevy_rl gym state
    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    ai_gym_state.set_env_state(env_state);
}

/// Handle bevy_rl::EventControl
pub(crate) fn bevy_rl_control_request(
    ai_gym_state: Res<AIGymState<Actions, EnvironmentState>>,
    mut control_event_reader: EventReader<EventControl>,
    mut simulation_state: ResMut<State<SimulationState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    query_actors: Query<(&mut Velocity, &mut Transform, &Actor)>,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
) {
    for control in control_event_reader.iter() {
        let mut ai_gym_state = ai_gym_state.lock().unwrap();
        let ai_gym_settings = ai_gym_state.settings.clone();
        let unparsed_actions = &control.0;
        let mut actions: Vec<Option<Actions>> =
            (0..ai_gym_settings.num_agents).map(|_| None).collect();

        for i in 0..unparsed_actions.len() {
            if let Some(unparsed_action) = unparsed_actions[i].clone() {
                ai_gym_state.set_reward(i, 0.0);
                // Pass control inputs to your agents

                let action = match unparsed_action.as_str() {
                    "FORWARD" => Some(Actions::FORWARD),
                    "BACKWARD" => Some(Actions::BACKWARD),
                    "LEFT" => Some(Actions::LEFT),
                    "RIGHT" => Some(Actions::RIGHT),
                    "TURN_LEFT" => Some(Actions::TURN_LEFT),
                    "TURN_RIGHT" => Some(Actions::TURN_RIGHT),
                    "SHOOT" => Some(Actions::SHOOT),
                    _ => None,
                };

                actions[i] = action;
            } else {
                actions[i] = None;
            }
        }

        control_agents(actions, query_actors, collision_events, event_gun_shot);
        // Resume simulation (physics engine)
        rapier_configuration.physics_pipeline_active = true;

        // Return to running state; note that it uses pop/push to avoid
        // entering `SystemSet::on_enter(SimulationState::Running)` which initialized game world anew
        simulation_state.pop().unwrap();
        return;
    }
}

/// Handle bevy_rl::EventReset
pub(crate) fn bevy_rl_reset_request(
    mut reset_event_reader: EventReader<EventReset>,
    mut commands: Commands,
    mut walls: Query<Entity, &Wall>,
    mut players: Query<(Entity, &Actor)>,
    mut simulation_state: ResMut<State<SimulationState>>,
    ai_gym_state: Res<AIGymState<Actions, EnvironmentState>>,
) {
    if reset_event_reader.iter().count() == 0 {
        return;
    }

    for e in walls.iter_mut() {
        commands.entity(e).despawn_recursive();
    }

    for (e, _) in players.iter_mut() {
        commands.entity(e).despawn_recursive();
    }

    simulation_state.set(SimulationState::Running).unwrap();

    let ai_gym_state = ai_gym_state.lock().unwrap();
    ai_gym_state.send_reset_result(true);
}

/// Handle EventRoundOver
pub(crate) fn event_round_over(
    ai_gym_state: Res<AIGymState<Actions, EnvironmentState>>,
    mut event_round_over_reader: EventReader<EventRoundOver>,
    mut pause_event_writer: EventWriter<EventPause>,
) {
    if event_round_over_reader.iter().count() == 0 {
        return;
    }

    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    let ai_gym_settings = ai_gym_state.settings.clone();

    for i in 0..ai_gym_settings.num_agents {
        ai_gym_state.set_terminated(i as usize, true);
    }

    pause_event_writer.send(EventPause);
}

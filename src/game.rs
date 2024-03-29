use bevy::prelude::*;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin};
use bevy_rapier3d::prelude::*;
use bevy_rl::*;

use crate::{actions::*, actors::*, events::*, gym::*, level::*};

// ----------
// Components
// ----------

#[derive(Component, Resource)]
pub(crate) struct RoundTimer(pub(crate) Timer);

#[derive(Clone, Reflect)]
pub(crate) struct RaycastMarker;

// -------
// Systems
// -------

fn restart_round_timer(
    mut timer: ResMut<RoundTimer>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
) {
    timer.0.reset();
    simulation_state.set(SimulationState::Running);
}

fn check_termination(
    player_query: Query<&Actor>,
    time: Res<Time>,
    // mut app_state: ResMut<State<AppState>>,
    mut round_timer: ResMut<RoundTimer>,
    ai_gym_state: ResMut<AIGymState<Actions, EnvironmentState>>,
    mut event_round_over_writer: EventWriter<EventRoundOver>,
) {
    let zero_health_actors = player_query.iter().filter(|p| p.health == 0).count() as u32;
    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    let ai_gym_settings = ai_gym_state.settings.clone();
    let agents: Vec<&Actor> = player_query.iter().collect();
    #[allow(clippy::needless_range_loop)]
    for i in 0..agents.len() {
        if agents[i].health == 0 {
            ai_gym_state.set_terminated(i, true);
        }
    }

    if ai_gym_settings.num_agents == zero_health_actors || seconds_left == 0 {
        event_round_over_writer.send(EventRoundOver);
    }
}

pub(crate) fn build_game_app(_mode: String) -> App {
    let gym_settings = AIGymSettings {
        width: 256,
        height: 256,
        num_agents: 16,
        pause_interval: 0.1,
        render_to_buffer: true,
    };

    let mut app = App::new();

    // Resources
    app.insert_resource(ClearColor(Color::WHITE))
        .insert_resource(DefaultPluginState::<RaycastMarker>::default())
        .insert_resource(AIGymState::<Actions, EnvironmentState>::new(gym_settings))
        .insert_resource(RoundTimer(Timer::from_seconds(60.0, TimerMode::Repeating)))
        .init_resource::<GameMap>();

    // Events
    app.add_event::<EventGunShot>()
        .add_event::<EventDamage>()
        .add_event::<EventRoundOver>();

    // Plugins
    app.add_plugins(DefaultPlugins)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(DefaultRaycastingPlugin::<RaycastMarker>::default())
        // bevy_rl initialization
        .add_plugin(AIGymPlugin::<Actions, EnvironmentState>::default());

    // Game world logic
    app.add_state::<SimulationState>();

    app.add_systems(
        (spawn_game_world, spawn_computer_actors, restart_round_timer)
            .chain()
            .in_set(OnUpdate(SimulationState::Initializing)),
    );

    app.add_systems(
        (
            event_gun_shot,
            event_damage,
            event_round_over,
            check_termination,
        )
            .in_set(OnUpdate(SimulationState::Running)),
    );

    app.add_systems(
        (
            bevy_rl_control_request,
            bevy_rl_reset_request,
            bevy_rl_pause_request,
        )
            .chain()
            .in_set(OnUpdate(SimulationState::PausedForControl)),
    );

    app
}

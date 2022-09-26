use std::sync::{Arc, Mutex};

use bevy::prelude::*;
// use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin};
use bevy_rl::{state::AIGymState, AIGymPlugin, AIGymSettings};
use heron::*;

use crate::{
    actions::*, actors::Actor, actors::*, app_states::*, control::*, events::*, gym::*, level::*,
};

// ----------
// Components
// ----------

#[derive(Component)]
pub(crate) struct RoundTimer(pub(crate) Timer);

pub(crate) struct RaycastMarker;

// -------
// Systems
// -------

fn clear_world(
    mut commands: Commands,
    mut walls: Query<Entity, &Wall>,
    mut players: Query<(Entity, &Actor)>,
) {
    // for e in walls.iter_mut() {
    //     commands.entity(e).despawn_recursive();
    // }

    for (e, _) in players.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}

fn restart_round_timer(mut timer: ResMut<RoundTimer>) {
    timer.0.reset();
}

fn check_termination(
    player_query: Query<&Actor>,
    time: Res<Time>,
    mut app_state: ResMut<State<AppState>>,
    mut round_timer: ResMut<RoundTimer>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    ai_gym_settings: Res<AIGymSettings>,
) {
    let zero_health_actors = player_query.iter().filter(|p| p.health == 0).count() as u32;
    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    let agents: Vec<&Actor> = player_query.iter().collect();
    for i in 0..agents.len() {
        if agents[i].health == 0 || seconds_left <= 0 {
            ai_gym_state.set_terminated(i, true);
        }
    }

    if ai_gym_settings.num_agents == zero_health_actors || seconds_left <= 0 {
        app_state.overwrite_set(AppState::RoundOver).unwrap();

        let results = (0..ai_gym_settings.num_agents).map(|_| true).collect();
        ai_gym_state.send_step_result(results);
    }
}

pub(crate) fn restart_round(
    mut app_state: ResMut<State<AppState>>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    ai_gym_state.reset();
    physics_time.resume();

    app_state.set(AppState::InGame).unwrap();
}

pub(crate) fn build_game_app(mode: String) -> App {
    let mut app = App::new();

    let gym_settings = AIGymSettings {
        width: 256,
        height: 256,
        num_agents: 16,
    };

    // Resources
    app.insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(DefaultPluginState::<RaycastMarker>::default())
        // Events
        .add_event::<EventGunShot>()
        .add_event::<EventDamage>()
        .add_event::<EventNewRound>()
        // Plugins
        .add_plugins(DefaultPlugins)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(DefaultRaycastingPlugin::<RaycastMarker>::default())
        // bevy_rl initialization
        .insert_resource(gym_settings.clone())
        .insert_resource(Arc::new(Mutex::new(AIGymState::<PlayerActionFlags>::new(
            gym_settings.clone(),
        ))))
        .add_plugin(AIGymPlugin::<PlayerActionFlags>::default())
        // game settings: round duration
        .insert_resource(RoundTimer(Timer::from_seconds(60.0, false)))
        // State chain
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(clear_world))
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(spawn_game_world.label("spawn_game_world"))
                .with_system(
                    spawn_computer_actors
                        .label("spawn_computer_actors")
                        .after("spawn_game_world"),
                )
                .with_system(
                    restart_round_timer
                        .label("restart_round_timer")
                        .after("spawn_game_world"),
                ),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Event handlers
                .with_system(event_gun_shot)
                .with_system(event_damage),
        )
        .add_system_set(SystemSet::on_enter(AppState::Reset).with_system(clear_world))
        .add_system_set(SystemSet::on_update(AppState::Reset).with_system(restart_round))
        // Initialize Resources
        .init_resource::<GameMap>();
    if mode == "train" {
        app.add_state(AppState::InGame);

        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(check_termination)
                .with_system(turnbased_control_system_switch),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::Control)
                // Game Systems
                .with_system(execute_step_request)
                .with_system(execute_reset_request),
        );

        app.add_system_set(SystemSet::on_exit(AppState::RoundOver).with_system(clear_world));
        app.add_system_set(
            SystemSet::on_update(AppState::RoundOver).with_system(execute_reset_request),
        );

        app.insert_resource(DelayedControlTimer(Timer::from_seconds(0.1, true)));
    }

    return app;
}

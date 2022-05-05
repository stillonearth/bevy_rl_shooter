use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin};
use bevy_rl::{state::AIGymState, AIGymPlugin, AIGymSettings};
use big_brain::prelude::*;
use heron::*;

use crate::{
    actions::*, actors::Actor, actors::*, ai::*, animations::*, app_states::*, assets::*,
    control::*, events::*, gym::*, hud::*, level::*, render::*, screens::*,
};

// ----------
// Components
// ----------

#[derive(Component)]
struct PlayerAvatar;

#[derive(Component)]
pub(crate) struct RoundTimer(pub(crate) Timer);

#[derive(Component)]
struct Weapon;

pub(crate) struct RaycastMarker;

// -------
// Systems
// -------

// Main Menu

fn clear_world(
    mut commands: Commands,
    mut walls: Query<Entity, With<Wall>>,
    mut players: Query<(Entity, &Actor)>,
    mut interface: Query<Entity, With<Interface>>,
) {
    for e in walls.iter_mut() {
        commands.entity(e).despawn_recursive();
    }

    for (e, _) in players.iter_mut() {
        commands.entity(e).despawn_recursive();
    }

    for e in interface.iter_mut() {
        commands.entity(e).despawn_recursive();
    }
}

// InGame

fn restart_round_timer(mut commands: Commands) {
    commands.insert_resource(RoundTimer(Timer::from_seconds(60.0, false)));
}

fn check_termination(
    player_query: Query<&Actor>,
    time: Res<Time>,
    mut app_state: ResMut<State<AppState>>,
    mut round_timer: ResMut<RoundTimer>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
) {
    let player_1 = player_query.iter().find(|p| p.name == "Player 1").unwrap();
    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    if player_1.health == 0 || seconds_left <= 0 {
        let mut ai_gym_state = ai_gym_state.lock().unwrap();

        ai_gym_state.set_terminated(true);
        ai_gym_state.send_step_result(true);

        app_state.set(AppState::RoundOver);
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
    app_state.set(AppState::InGame);
}

pub(crate) fn build_game_app(mode: String) -> App {
    let mut app = App::new();

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
        .insert_resource(AIGymSettings {
            width: 768,
            height: 768,
        })
        .insert_resource(Arc::new(Mutex::new(AIGymState::<PlayerActionFlags>::new())))
        .add_plugin(AIGymPlugin::<PlayerActionFlags>::default())
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(DefaultRaycastingPlugin::<RaycastMarker>::default())
        .add_plugin(BigBrainPlugin)
        // State chain
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(main_screen))
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(button_system))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(clear_world))
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(spawn_game_world)
                .with_system(spawn_player_actor)
                .with_system(spawn_computer_actors)
                .with_system(draw_gun)
                .with_system(restart_round_timer),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(animate_gun)
                .with_system(animate_enemy)
                .with_system(render_billboards)
                // Event handlers
                .with_system(event_gun_shot)
                .with_system(event_damage),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Reset).with_system(clear_world.chain(restart_round)),
        )
        // AI -- global due to
        .add_system(bloodthirst_system)
        .add_system_to_stage(BigBrainStage::Actions, kill_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, bloodthirsty_scorer_system)
        // Initialize Resources
        .init_resource::<GameMap>()
        .init_resource::<GameAssets>();

    if mode == "train" {
        app.add_state(AppState::InGame);

        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(draw_hud));
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(update_hud)
                .with_system(check_termination)
                .with_system(turnbased_control_system_switch),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::Control)
                // Game Systems
                .with_system(turnbased_text_control_system)
                .with_system(execute_reset_request),
        );

        app.add_system_set(SystemSet::on_exit(AppState::RoundOver).with_system(clear_world));
        app.add_system_set(
            SystemSet::on_update(AppState::RoundOver).with_system(execute_reset_request),
        );

        app.insert_resource(DelayedControlTimer(Timer::from_seconds(0.1, true)));
    } else if mode == "playtest" {
        app.add_state(AppState::InGame);

        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(draw_hud));
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(update_hud)
                .with_system(check_termination)
                .with_system(turnbased_control_system_switch),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::Control)
                // Game Systems
                .with_system(turnbased_control_player_keyboard)
                .with_system(execute_reset_request),
        );

        app.add_system_set(SystemSet::on_exit(AppState::RoundOver).with_system(clear_world));
        app.add_system_set(
            SystemSet::on_update(AppState::RoundOver).with_system(execute_reset_request),
        );

        app.insert_resource(DelayedControlTimer(Timer::from_seconds(0.1, true)));
    } else {
        // This branch would panic on current version
        app.add_state(AppState::MainMenu);
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(draw_hud));
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(update_hud)
                .with_system(control_player_keyboard)
                .with_system(check_termination),
        );
        app.add_system_set(SystemSet::on_enter(AppState::RoundOver).with_system(round_over));
        app.add_system_set(SystemSet::on_update(AppState::RoundOver).with_system(button_system));
        app.add_system_set(SystemSet::on_exit(AppState::RoundOver).with_system(clear_world));
    }

    return app;
}

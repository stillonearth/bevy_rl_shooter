use heron::*;

// use std::ops::Range;
use std::sync::{Arc, Mutex};
// use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::utils::Instant;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastSource};

use big_brain::prelude::*;

use bevy_rl::state::AIGymState;
use bevy_rl::AIGymCamera;
use bevy_rl::AIGymPlugin;
use bevy_rl::AIGymSettings;

use names::Generator;
use rand::thread_rng;
use rand::{seq::SliceRandom, Rng};
// use serde::{Deserialize, Serialize};
// use serde_json;

const DEBUG: bool = false;

use crate::{
    actions::*, ai::*, animations::*, app_states::*, assets::*, control::*, events::*, gym::*,
    hud::*, level::*, physics::*, player::*, render::*, screens::*,
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
    mut players: Query<(Entity, &Player)>,
    mut interface: Query<Entity, With<Interface>>,
    ai_gym_state: Res<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
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

    let ai_gym_state = ai_gym_state.lock().unwrap();

    if ai_gym_state.__result_reset_channel_tx.is_empty() {
        ai_gym_state.__result_reset_channel_tx.send(true).unwrap();
    }
}

// InGame

#[derive(Component)]
pub(crate) struct Wall;

fn spawn_game_world(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = 255.0 * 255.0;
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: (size as f32),
    }));

    let white_material_handle = materials.add(Color::WHITE.into());

    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: white_material_handle.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::HeightField {
            size: Vec2::new((100 * 255) as f32, (100 * 255) as f32),
            heights: vec![
                vec![100.5, 0.8, 0., 0., 3000.0],
                vec![0.8, 0.2, 0., 0., 300.0],
                vec![0., 0.5, 0., 0., 300.0],
                vec![0., 0., 0.6, 0., 300.0],
                vec![300., 300., 300., 300., 300.0],
            ],
        });

    if DEBUG {
        return;
    }

    for (x, z) in game_map.walls.iter() {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                material: white_material_handle.clone(),
                transform: Transform::from_translation(Vec3::new(*x as f32, 1.0, *z as f32)),
                global_transform: GlobalTransform::identity(),
                ..Default::default()
            })
            .insert(RigidBody::Static)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(1.0, 1.0, 1.0),
                border_radius: None,
            })
            .insert(Wall)
            .insert(CollisionLayers::new(Layer::World, Layer::Player))
            .insert(RayCastMesh::<RaycastMarker>::default()); // Make this mesh ray cast-able
    }
}

fn init_round(mut commands: Commands) {
    commands.insert_resource(RoundTimer(Timer::from_seconds(60.0, false)));
}

fn spawn_player(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    ai_gym_state: Res<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();
    let mut rng = thread_rng();
    let pos = game_map.empty_space.choose(&mut rng).unwrap();
    let player = Player {
        position: (pos.0 as f32, pos.1 as f32),
        rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        name: "Player 1".to_string(),
        health: 100,
        score: 0,
    };

    let mut ec: EntityCommands;
    ec = commands.spawn_bundle(());

    ec.insert(Transform {
        translation: Vec3::new(player.position.0 as f32, 1.0, player.position.1 as f32),
        rotation: Quat::from_rotation_y(player.rotation),
        ..Default::default()
    });
    ec.insert(GlobalTransform::identity());
    ec.insert(Velocity::from_linear(Vec3::ZERO));
    ec.insert(CollisionShape::Sphere { radius: 1.0 })
        .insert(PlayerPerspective)
        .insert(RigidBody::Dynamic)
        .insert(PhysicMaterial {
            density: 200.0,
            ..Default::default()
        })
        .insert(CollisionLayers::new(Layer::Player, Layer::World))
        .insert(RotationConstraints::lock())
        .insert(player);

    ec.with_children(|cell| {
        cell.spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 500.0,
                shadows_enabled: false,
                ..Default::default()
            },
            ..Default::default()
        });

        // Camera
        cell.spawn_bundle(PerspectiveCameraBundle::<AIGymCamera> {
            camera: Camera {
                target: ai_gym_state.__render_target.clone().unwrap(),
                ..default()
            },
            ..PerspectiveCameraBundle::new()
        })
        .insert(RayCastSource::<RaycastMarker>::new_transform_empty());

        let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.8, 1.7))));
        cell.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            transform: Transform {
                rotation: Quat::from_rotation_y(std::f32::consts::PI),
                ..Default::default()
            },
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .insert(RayCastMesh::<RaycastMarker>::default());
    });
}

pub(crate) fn spawn_enemies(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    game_sprites: Res<GameAssets>,
    wolfenstein_sprites: Res<GameAssets>,
) {
    let enemy_count = match DEBUG {
        true => 64,
        false => 64,
    };

    for _ in 0..enemy_count {
        // choose player random spawn point
        let mut rng = thread_rng();
        let pos = game_map.empty_space.choose(&mut rng).unwrap();
        let player = Player {
            position: (pos.0 as f32, pos.1 as f32),
            rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
            name: Generator::default().next().unwrap(),
            health: 100,
            score: 0,
        };

        let transform = Transform {
            translation: Vec3::new((player.position.0) as f32, 1.0, (player.position.1) as f32),
            rotation: Quat::from_rotation_y(player.rotation),
            ..Default::default()
        };

        commands
            .spawn_bundle((transform, GlobalTransform::identity()))
            .with_children(|cell| {
                let mut mesh = Mesh::from(shape::Quad::new(Vec2::new(0.8, 1.7)));
                let uv = wolfenstein_sprites.guard_standing_animation[0][0].clone();
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                let mesh = meshes.add(mesh);

                cell.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: game_sprites.guard_billboard_material.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Billboard)
                .insert(EnemyAnimation {
                    frame: 0,
                    handle: mesh,
                    animation_type: AnimationType::Standing,
                })
                .insert(AnimationTimer(Timer::from_seconds(0.3, true)))
                .insert(RayCastMesh::<RaycastMarker>::default());

                let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.8, 1.7))));
                cell.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: game_sprites.guard_billboard_material.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO,
                        ..Default::default()
                    },
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(RayCastSource::<RaycastMarker>::new_transform_empty());
            })
            .insert(CollisionShape::Sphere { radius: 0.8 })
            .insert(RigidBody::Dynamic)
            .insert(PhysicMaterial {
                density: 1.0,
                ..Default::default()
            })
            .insert(player)
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(CollisionLayers::new(Layer::Player, Layer::World))
            .insert(RotationConstraints::lock())
            .insert(BloodThirst { enemies_near: 0 })
            .insert(
                Thinker::build()
                    .picker(FirstToScore { threshold: 0.0 })
                    .when(
                        BloodThirsty,
                        Kill {
                            last_action: Instant::now(),
                        },
                    ),
            );
    }
}

fn check_termination(
    player_query: Query<&Player>,
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
        ai_gym_state.is_terminated = true;

        if ai_gym_state.__result_channel_rx.is_empty() {
            ai_gym_state.__result_channel_tx.send(true).unwrap();
        }

        app_state.set(AppState::RoundOver);
    }
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
        .insert_resource(Arc::new(Mutex::new(AIGymState::<PlayerActionFlags> {
            ..Default::default()
        })))
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
                .with_system(spawn_game_world.label("spawn_world"))
                .with_system(spawn_player.label("spawn_player").after("spawn_world"))
                .with_system(spawn_enemies.after("spawn_player").label("spawn_enemies"))
                .with_system(draw_gun)
                .with_system(init_round.after("spawn_enemies")),
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
            SystemSet::on_enter(AppState::Reset)
                .with_system(restart_round)
                .with_system(clear_world),
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

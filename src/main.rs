use rand::thread_rng;
use rand::{seq::SliceRandom, Rng};
use std::ops::Range;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::utils::Instant;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastSource};
use big_brain::prelude::*;
use heron::*;

use bitflags::bitflags;
use clap::Parser;
use names::Generator;
use serde::{Deserialize, Serialize};

mod gym;
mod map;

const DEBUG: bool = false;

// clap command line arguments

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    mode: String,
}

// Physics

#[derive(PhysicsLayer)]
enum Layer {
    World,
    Player,
}

// ----------
// Components
// ----------

#[derive(Component, Clone)]
struct Player {
    position: (f32, f32),
    rotation: f32,
    name: String,
    health: u16,
    score: u16,
}

#[derive(Component)]
struct PlayerPerspective;

#[derive(Component)]
struct PlayerAvatar;

#[derive(PartialEq, Clone)]
enum AnimationType {
    Standing,
    Walking,
    Shooting,
    Dying,
}

#[derive(Component, Clone)]
struct EnemyAnimation {
    pub frame: u8,
    pub handle: Handle<Mesh>,
    pub animation_type: AnimationType,
}

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct RoundTimer(Timer);

#[derive(Component)]
struct Weapon;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimeLeftText;

struct RaycastMarker;

// ------
// Events
// ------
#[derive(Debug)]
pub struct EventGunShot {
    from: String,
}

#[derive(Debug)]
pub struct EventDamage {
    from: String,
    to: String,
}

#[derive(Debug)]
pub struct EventNewRound;

// ------
// States
// ------

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    Control,
    RoundOver,
}

// -------
// Systems
// -------

// Main Menu

pub fn main_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("Roboto-Regular.ttf");

    let text = Text::with_section(
        "ROYAL BATTLE BEVYSTEIN",
        TextStyle {
            font_size: 75.0,
            font: font.clone(),
            color: Color::rgb(0.2, 0.2, 0.2),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    // commands.spawn_bundle(UiCameraBundle::default());
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(170.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position: Rect {
                            top: Val::Px(50.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::BLACK.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "NEW ROUND",
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn_bundle(TextBundle {
                text,
                ..Default::default()
            });
        });
}

fn button_system(
    mut app_state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                app_state.set(AppState::InGame).unwrap();
            }
            Interaction::Hovered => {
                *color = Color::GRAY.into();
            }
            Interaction::None => {
                *color = Color::BLACK.into();
            }
        }
    }
}

fn clear_world(mut commands: Commands, mut q: Query<Entity>) {
    for q in q.iter_mut() {
        commands.entity(q).despawn();
    }
}

// InGame

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

    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: materials.add(Color::WHITE.into()),
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
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_translation(Vec3::new(*x as f32, 1.0, *z as f32)),
                global_transform: GlobalTransform::identity(),
                ..Default::default()
            })
            .insert(RigidBody::Static)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(1.0, 1.0, 1.0),
                border_radius: None,
            })
            .insert(CollisionLayers::new(Layer::World, Layer::Player))
            .insert(RayCastMesh::<RaycastMarker>::default()); // Make this mesh ray cast-able
    }
}

pub fn init_round(mut commands: Commands) {
    commands.insert_resource(RoundTimer(Timer::from_seconds(300.0, false)));
}

pub fn spawn_player(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    ai_gym_state: Res<Arc<Mutex<gym::AIGymState<PlayerActionFlags>>>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();
    let mut rng = thread_rng();
    let pos = game_map.empty_space.choose(&mut rng).unwrap();
    let player = Player {
        position: (pos.0 as f32, pos.1 as f32),
        rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        name: "Player 1".to_string(),
        health: 1000,
        score: 0,
    };

    commands
        .spawn_bundle((
            Transform {
                translation: Vec3::new(player.position.0 as f32, 1.0, player.position.1 as f32),
                rotation: Quat::from_rotation_y(player.rotation),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .with_children(|cell| {
            cell.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 500.0,
                    shadows_enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            });

            // Camera
            cell.spawn_bundle(PerspectiveCameraBundle::<gym::FirstPassCamera> {
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
        })
        .insert(CollisionShape::Sphere { radius: 1.0 })
        .insert(PlayerPerspective)
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(RigidBody::Dynamic)
        .insert(PhysicMaterial {
            density: 200.0,
            ..Default::default()
        })
        .insert(CollisionLayers::new(Layer::Player, Layer::World))
        .insert(RotationConstraints::lock())
        .insert(player)
        .insert(BloodThirst { enemies_near: 0 });
}

pub fn spawn_enemies(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
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
                let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.8, 1.7))));

                cell.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: wolfenstein_sprites.guard_billboard_material.clone(),
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
                    material: wolfenstein_sprites.guard_billboard_material.clone(),
                    transform: Transform {
                        translation: Vec3::ZERO,
                        ..Default::default()
                    },
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(RayCastSource::<RaycastMarker>::new_transform_empty());

                // cell.spawn_scene(asset_server.load("craft_speederA.glb#Scene0"));
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

bitflags! {
    #[derive(Default)]
    pub struct PlayerActionFlags: u32 {
        const IDLE = 1 << 0;
        const FORWARD = 1 << 1;
        const BACKWARD = 1 << 2;
        const LEFT = 1 << 3;
        const RIGHT = 1 << 4;
        const TURN_LEFT = 1 << 5;
        const TURN_RIGHT = 1 << 6;
        const SHOOT = 1 << 7;
    }
}

fn control_player_keyboard(
    keys: Res<Input<KeyCode>>,
    player_movement_q: Query<(&mut heron::prelude::Velocity, &Transform), With<PlayerPerspective>>,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
) {
    let mut player_action = PlayerActionFlags::IDLE;

    for key in keys.get_pressed() {
        if *key == KeyCode::W {
            player_action |= PlayerActionFlags::FORWARD;
        }
        if *key == KeyCode::A {
            player_action |= PlayerActionFlags::BACKWARD;
        }
        if *key == KeyCode::S {
            player_action |= PlayerActionFlags::LEFT;
        }
        if *key == KeyCode::D {
            player_action |= PlayerActionFlags::RIGHT;
        }
        if *key == KeyCode::Q {
            player_action |= PlayerActionFlags::TURN_LEFT;
        }
        if *key == KeyCode::E {
            player_action |= PlayerActionFlags::TURN_RIGHT;
        }
        if keys.just_pressed(KeyCode::Space) {
            player_action |= PlayerActionFlags::SHOOT;
        }
    }

    control_player(
        player_action,
        player_movement_q,
        collision_events,
        event_gun_shot,
    );
}

fn control_player(
    player_action: PlayerActionFlags,
    mut player_movement_q: Query<
        (&mut heron::prelude::Velocity, &Transform),
        With<PlayerPerspective>,
    >,
    mut collision_events: EventReader<CollisionEvent>,
    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    fn is_player(layers: CollisionLayers) -> bool {
        layers.contains_group(Layer::Player) && !layers.contains_group(Layer::World)
    }

    fn is_world(layers: CollisionLayers) -> bool {
        !layers.contains_group(Layer::Player) && layers.contains_group(Layer::World)
    }

    for (mut velocity, transform) in player_movement_q.iter_mut() {
        *velocity = Velocity::from_linear(Vec3::ZERO);
        if player_action.contains(PlayerActionFlags::FORWARD) {
            *velocity =
                velocity.with_linear(velocity.linear + 10. * transform.forward().normalize());
        }
        if player_action.contains(PlayerActionFlags::BACKWARD) {
            *velocity = velocity.with_linear(velocity.linear + 10. * transform.left().normalize());
        }
        if player_action.contains(PlayerActionFlags::LEFT) {
            *velocity =
                velocity.with_linear(velocity.linear + 10. * -transform.forward().normalize());
        }
        if player_action.contains(PlayerActionFlags::RIGHT) {
            *velocity = velocity.with_linear(velocity.linear + 10. * transform.right().normalize());
        }
        if player_action.contains(PlayerActionFlags::TURN_LEFT) {
            *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
        }
        if player_action.contains(PlayerActionFlags::TURN_RIGHT) {
            *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
        }
        if player_action.contains(PlayerActionFlags::SHOOT) {
            event_gun_shot.send(EventGunShot {
                from: "Player 1".to_string(),
            });
        }

        collision_events
            .iter()
            .filter_map(|event| {
                let (entity_1, entity_2) = event.rigid_body_entities();
                let (layers_1, layers_2) = event.collision_layers();

                if is_player(layers_1) && is_world(layers_2) {
                    Some(entity_2)
                } else if is_player(layers_2) && is_world(layers_1) {
                    Some(entity_1)
                } else {
                    // This event is not the collision between an enemy and the player. We can ignore it.
                    None
                }
            })
            .for_each(|_| {
                *velocity = Velocity::from_linear(Vec3::X * 0.0);
            });
    }
}

fn render_billboards(
    mut q: QuerySet<(
        QueryState<(&Parent, &mut Transform), With<Billboard>>,
        QueryState<(&GlobalTransform, &Transform), With<PlayerPerspective>>,
    )>,
    parent_query: Query<(&Player, &GlobalTransform)>,
) {
    let viewer_transform = q.q1().iter().last().unwrap().1.translation;
    for (parent, mut t) in q.q0().iter_mut() {
        let parent_position = parent_query.get(parent.0).unwrap().1.translation;
        let parent_rotation = parent_query.get(parent.0).unwrap().0.rotation;

        let delta_z = parent_position.z - viewer_transform.z;
        let delta_x = parent_position.x - viewer_transform.x;
        let angle = delta_x.atan2(delta_z);

        let rot_y = Quat::from_rotation_y(std::f32::consts::PI + angle - parent_rotation);

        t.rotation = rot_y;
    }
}

// PostGame

pub fn round_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("Roboto-Regular.ttf");

    let text = Text::with_section(
        "YOU WIN",
        TextStyle {
            font_size: 75.0,
            font: font.clone(),
            color: Color::rgb(0.2, 0.2, 0.2),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    // commands.spawn_bundle(UiCameraBundle::default());
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color: Color::WHITE.clone().into(),
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(170.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position: Rect {
                            top: Val::Px(50.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::BLACK.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "NEW ROUND",
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn_bundle(TextBundle {
                text,
                ..Default::default()
            });
        });
}

// ------
// Events
// ------

fn event_gun_shot(
    mut commands: Commands,

    mut gun_sprite_query: Query<(&Weapon, &mut UiImage)>,
    shooting_query: Query<(&Parent, &RayCastSource<RaycastMarker>)>,
    player_query: Query<(Entity, &Children, &Player)>,

    mut gunshot_event: EventReader<EventGunShot>,
    mut event_damage: EventWriter<EventDamage>,

    mut wolfenstein_sprites: ResMut<GameAssets>,
) {
    for gunshot_event in gunshot_event.iter() {
        if gunshot_event.from == "Player 1".to_string() {
            for (_, mut ui_image) in gun_sprite_query.iter_mut() {
                wolfenstein_sprites.gun_index = 1;
                ui_image.0 = wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                    .clone()
                    .into();
            }
        }

        let (_, raycast_source) = shooting_query
            .iter()
            .find(|(p, _)| {
                !player_query
                    .iter()
                    .find(|(e, _, _p)| e.id() == p.id() && _p.name == gunshot_event.from)
                    .is_none()
            })
            .unwrap();

        let r = raycast_source.intersect_top();
        if r.is_none() {
            continue;
        }

        let hit_entity = r.unwrap().0;

        let mut player_hit = false;
        // println!("{:?}", r.1.);
        for (_, children, enemy) in player_query.iter() {
            let other_entity = children.iter().find(|c| c.id() == hit_entity.id());
            if other_entity.is_none() {
                continue;
            }

            event_damage.send(EventDamage {
                from: gunshot_event.from.clone(),
                to: enemy.name.clone(),
            });

            player_hit = true;
            continue;
        }

        // despawn a wall
        if !player_hit {
            commands.entity(hit_entity).despawn();
        }
    }
}

fn event_new_round(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Children, &mut Player, &mut Velocity)>,
    mut billboard_query: Query<(Entity, &mut EnemyAnimation, &Billboard)>,
    mut event_damage: EventReader<EventDamage>,
) {
}

fn event_damage(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Children, &mut Player, &mut Velocity)>,
    mut billboard_query: Query<(Entity, &mut EnemyAnimation, &Billboard)>,
    mut event_damage: EventReader<EventDamage>,
) {
    for damage_event in event_damage.iter() {
        if damage_event.from == damage_event.to {
            continue;
        }

        if let Some((entity, children, mut player, mut velocity)) = player_query
            .iter_mut()
            .find(|p| p.2.name == damage_event.to)
        {
            if player.health == 0 {
                continue;
            }

            player.health -= 100;

            if player.health == 0 {
                for c in children.iter() {
                    if let Some((billboard_entity, mut animation, _)) =
                        billboard_query.iter_mut().find(|c_| {
                            return c_.0.id() == c.id();
                        })
                    {
                        animation.frame = 1;
                        animation.animation_type = AnimationType::Dying;

                        velocity.clone_from(&Velocity::from_linear(Vec3::ZERO));

                        commands
                            .entity(billboard_entity)
                            .insert(EnemyAnimation {
                                frame: 0,
                                handle: animation.handle.clone(),
                                animation_type: AnimationType::Dying,
                            })
                            .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
                            .remove::<RayCastMesh<RaycastMarker>>();
                    }
                }

                commands
                    .entity(entity)
                    .insert(Velocity::from_linear(Vec3::ZERO));
            }

            let (_, _, mut hit_player, _) = player_query
                .iter_mut()
                .find(|(_, _, p, _)| p.name == damage_event.from)
                .unwrap();

            hit_player.score += 10;
        }
    }
}

// ---------
// Resources
// ---------

pub struct GameAssets {
    pub gun: Vec<Handle<Image>>,
    pub gun_index: u8,

    pub guard_billboard_material: Handle<StandardMaterial>,
    pub guard_walking_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_standing_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_dying_animation: Vec<Vec<[f32; 2]>>,
    pub guard_shooting_animation: Vec<Vec<[f32; 2]>>,

    pub font: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        // gun
        let mut gun: Vec<Handle<Image>> = Vec::new();
        gun.push(asset_server.load("gun/gun_0.png"));
        gun.push(asset_server.load("gun/gun_1.png"));
        gun.push(asset_server.load("gun/gun_2.png"));

        // soldier
        let guard_billboard_material = materials.add(StandardMaterial {
            base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            base_color_texture: Some(asset_server.load("guard-sheet.png")),
            unlit: true,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        fn gather_angle_animation_uvs(
            row: f32,
            frames: Range<u8>,
            offset: f32,
        ) -> Vec<Vec<[f32; 2]>> {
            let mut frame_set: Vec<Vec<[f32; 2]>> = Vec::new();
            // let offset = 0.042;

            for column in frames {
                let mut uvs1: Vec<[f32; 2]> = Vec::<[f32; 2]>::new();
                let column = column as f32;

                uvs1.push([(row - 1.0) / 8.0 + offset, column / 7.0]);
                uvs1.push([(row - 1.0) / 8.0 + offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column) / 7.0]);

                frame_set.push(uvs1);
            }

            return frame_set;
        }

        fn gather_full_animation_uvs(frames: Range<u8>) -> Vec<Vec<Vec<[f32; 2]>>> {
            let mut animations: Vec<Vec<Vec<[f32; 2]>>> = Vec::new();

            for i in 1..9 {
                let angle_animations = gather_angle_animation_uvs(i as f32, frames.clone(), 0.042);
                animations.push(angle_animations);
            }

            return animations;
        }

        fn gather_row_animation_uvs(
            column: f32,
            frames: Range<u8>,
            offset: f32,
        ) -> Vec<Vec<[f32; 2]>> {
            let mut frame_set: Vec<Vec<[f32; 2]>> = Vec::new();
            // let offset = 0.0;

            for row in frames {
                let mut uvs1: Vec<[f32; 2]> = Vec::<[f32; 2]>::new();
                let row = row as f32;

                uvs1.push([(row - 1.0) / 8.0 + offset, column / 7.0]);
                uvs1.push([(row - 1.0) / 8.0 + offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column) / 7.0]);

                frame_set.push(uvs1);
            }

            return frame_set;
        }

        return Self {
            gun,
            gun_index: 0,
            font: asset_server.load("Roboto-Regular.ttf"),
            guard_billboard_material,
            guard_walking_animation: gather_full_animation_uvs(Range { start: 2, end: 6 }),
            guard_standing_animation: gather_full_animation_uvs(Range { start: 1, end: 2 }),
            guard_dying_animation: gather_row_animation_uvs(6.0, Range { start: 1, end: 6 }, 0.0),
            guard_shooting_animation: gather_row_animation_uvs(
                7.0,
                Range { start: 1, end: 4 },
                0.042,
            ),
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameMap {
    pub empty_space: Vec<(usize, usize)>,
    pub walls: Vec<(usize, usize)>,
}

impl FromWorld for GameMap {
    fn from_world(_: &mut World) -> Self {
        let deserialized: GameMap = serde_json::from_str(&map::JSON).unwrap();
        return deserialized;
    }
}

// ----------
// Animations
// ----------

fn animate_gun(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<GameAssets>,
    mut query: Query<(&Weapon, &mut AnimationTimer, &mut UiImage)>,
) {
    if wolfenstein_sprites.gun_index == 0 {
        return;
    }

    for (_, mut timer, mut ui_image) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            wolfenstein_sprites.gun_index += 1;
            if wolfenstein_sprites.gun_index >= (wolfenstein_sprites.gun.len() as u8) {
                wolfenstein_sprites.gun_index = 0;
            }

            ui_image.0 = wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                .clone()
                .into();
        }
    }
}

fn animate_enemy(
    time: Res<Time>,
    wolfenstein_sprites: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,

    mut q: QuerySet<(
        QueryState<(&mut AnimationTimer, &Parent, &mut EnemyAnimation), With<Billboard>>,
        QueryState<&GlobalTransform, With<PlayerPerspective>>,
    )>,
    parent_query: Query<(&Player, &GlobalTransform)>,
) {
    let player_transform = q.q1().iter().last().unwrap();
    let player_position = player_transform.translation;
    let player_fwd = player_transform.forward().normalize();

    for (mut timer, parent, mut animation) in q.q0().iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // 2D animations

            let mut animations: Vec<Vec<[f32; 2]>> = Vec::new();
            if animation.animation_type == AnimationType::Dying {
                animations = wolfenstein_sprites.guard_dying_animation.clone();
            }

            if animation.animation_type == AnimationType::Shooting {
                animations = wolfenstein_sprites.guard_shooting_animation.clone();
            }

            if (animation.animation_type == AnimationType::Standing)
                || (animation.animation_type == AnimationType::Walking)
            {
                // 3D animations
                let frameset = match animation.animation_type {
                    AnimationType::Standing => wolfenstein_sprites.guard_standing_animation.clone(),
                    AnimationType::Walking => wolfenstein_sprites.guard_walking_animation.clone(),
                    _ => Vec::new(),
                };

                let parent_transform = parent_query.get(parent.0).unwrap().1;
                let enemy_fwd = parent_transform.forward().normalize();
                let enemy_position = parent_transform.translation;

                // this angle code was a major headache
                // brotip:
                //  * acos of dot product = absolute value of angle btwn vectors
                //  * crossproduct -> 3 vector, sign of a perpendiculat component indicates whether vectors left / right

                let mut angle = f32::acos(enemy_fwd.dot(player_fwd));
                let sign = -player_fwd.cross((enemy_fwd).normalize()).y.signum();
                angle *= sign;

                let mut view_angle =
                    f32::acos(player_fwd.dot((enemy_position - player_position).normalize()));

                let sign = -player_fwd
                    .cross((enemy_position - player_position).normalize())
                    .y
                    .signum();

                view_angle *= sign;

                angle += view_angle;
                angle *= 180.0 / std::f32::consts::PI;

                angle += 180.0;

                if angle < 0.0 {
                    angle += 360.0;
                }

                let mut index = 0;
                if angle >= 0.0 && angle < 45.0 {
                    index = 0
                } else if angle >= 45.0 && angle < 90.0 {
                    index = 1
                } else if angle >= 90.0 && angle < 135.0 {
                    index = 2
                } else if angle >= 135.0 && angle < 180.0 {
                    index = 3
                } else if angle >= 180.0 && angle < 225.0 {
                    index = 4
                } else if angle >= 225.0 && angle < 270.0 {
                    index = 5
                } else if angle >= 270.0 && angle < 315.0 {
                    index = 6
                } else if angle >= 315.0 && angle < 360.0 {
                    index = 7
                }
                animations = frameset[index].clone();
            }

            if let Some(mesh) = meshes.get_mut(animation.handle.clone()) {
                let uv = animations[animation.frame as usize].clone();

                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            }

            if animation.frame >= (animations.len() as u8 - 1) {
                if animation.animation_type == AnimationType::Shooting {
                    animation.animation_type = AnimationType::Standing;
                }

                if animation.animation_type != AnimationType::Dying {
                    animation.frame = 0;
                }
            } else {
                animation.frame += 1;
            }
        }
    }
}

fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn draw_hud(mut commands: Commands, game_assets: Res<GameAssets>) {
    let text = Text::with_section(
        "",
        TextStyle {
            font_size: 45.0,
            font: game_assets.font.clone(),
            color: Color::rgb(0.0, 0.0, 0.0),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // size: Size::new(Val::Percent(100.0), Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(50.0),
                    top: Val::Px(25.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        // size: Size::new(Val::Percent(100.0), Val::Auto),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ScoreText);

            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Auto),
                        position_type: PositionType::Relative,
                        position: Rect {
                            left: Val::Px(55.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(TimeLeftText);
        });
}

fn update_hud(
    player_query: Query<&Player>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    mut time_left_query: Query<&mut Text, Without<ScoreText>>,
    time: Res<Time>,
    mut round_timer: ResMut<RoundTimer>,
) {
    let player_1 = player_query.iter().find(|p| p.name == "Player 1");

    for mut text in score_text_query.iter_mut() {
        let str = format!("HEALTH {}", player_1.unwrap().health).to_string();
        text.sections[0].value = str;
    }

    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    for mut text in time_left_query.iter_mut() {
        let str = format!("TIME LEFT {}", seconds_left).to_string();
        text.sections[0].value = str;
    }
}

fn check_termination_play(
    player_query: Query<&Player>,
    time: Res<Time>,
    mut app_state: ResMut<State<AppState>>,
    mut round_timer: ResMut<RoundTimer>,
) {
    let player_1 = player_query.iter().find(|p| p.name == "Player 1").unwrap();
    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    if player_1.health == 0 || seconds_left == 0 {
        app_state.set(AppState::RoundOver).unwrap();
    }
}

fn check_termination_training(
    player_query: Query<&Player>,
    time: Res<Time>,
    mut round_timer: ResMut<RoundTimer>,
    mut event_writer_new_round: EventWriter<EventNewRound>,
) {
    let player_1 = player_query.iter().find(|p| p.name == "Player 1").unwrap();
    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    if player_1.health == 0 || seconds_left == 0 {
        event_writer_new_round.send(EventNewRound {});
    }
}

fn draw_gun(mut commands: Commands, wolfenstein_sprites: Res<GameAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(0.),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Auto),
                        ..Default::default()
                    },
                    image: wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                        .clone()
                        .into(),
                    ..Default::default()
                })
                .insert(Weapon)
                .insert(AnimationTimer(Timer::from_seconds(0.1, true)));
        });
}

// ----------
// --- AI ---
// ----------

#[derive(Component, Debug)]
pub struct BloodThirst {
    pub enemies_near: u8,
}

fn bloodthirst_system(mut thirsts: Query<(&GlobalTransform, &Player, &mut BloodThirst)>) {
    let _transforms: Vec<(GlobalTransform, Player)> = thirsts
        .iter()
        .map(|(p, g, _)| (p.clone(), g.clone()))
        .collect();

    for (gt, player, mut thirst) in thirsts.iter_mut() {
        if player.health == 0 {
            thirst.enemies_near = 0;
        } else {
            thirst.enemies_near = _transforms
                .iter()
                .filter(|(g, p)| {
                    if p.health == 0 {
                        return false;
                    }
                    let distance = ((gt.translation.x - g.translation.x).powf(2.0)
                        + (gt.translation.z - g.translation.z).powf(2.0))
                    .sqrt();
                    return (distance < 20.0) && (distance != 0.0);
                })
                .count() as u8;
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct Kill {
    last_action: Instant,
}

fn kill_action_system(
    mut bloodthirsts: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &mut BloodThirst,
        &mut Player,
    )>,
    mut actors: Query<(&Actor, &mut ActionState, &mut Kill)>,
    mut enemy_animations: Query<(Entity, &Parent, &mut EnemyAnimation)>,

    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    let players: Vec<(Entity, Transform, Player)> = bloodthirsts
        .iter()
        .map(|(e, _, t, _, p)| (e.clone(), t.clone(), p.clone()))
        .collect();

    for (Actor(actor), mut state, mut kill) in actors.iter_mut() {
        if let Some((_, mut velocity, mut transform, thirst, mut player)) =
            bloodthirsts.iter_mut().find(|e| e.0.id() == actor.id())
        {
            let (_, _, mut animation) = enemy_animations
                .iter_mut()
                .find(|p| p.1.id() == actor.id())
                .unwrap();

            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if thirst.enemies_near == 0 {
                        if player.health == 0 {
                            animation.animation_type = AnimationType::Dying;
                            // animation.frame = 0;
                        } else {
                            *velocity =
                                Velocity::from_linear(transform.forward().normalize() * 2.0);
                            animation.animation_type = AnimationType::Walking;
                        }

                        *state = ActionState::Success;
                    } else {
                        // turn to next target

                        *velocity = Velocity::from_linear(Vec3::ZERO);

                        let duration = kill.last_action.elapsed().as_secs_f32();
                        if duration <= 0.5 {
                            continue;
                        }

                        animation.animation_type = AnimationType::Shooting;
                        animation.frame = 0;

                        kill.last_action = Instant::now();

                        let near_enemy = players.iter().find(|(_, gt, e)| {
                            if e.health == 0 {
                                return false;
                            }
                            let distance = ((gt.translation.x - transform.translation.x).powf(2.0)
                                + (gt.translation.z - transform.translation.z).powf(2.0))
                            .sqrt();

                            return distance <= 20.0 && distance != 0.0;
                        });

                        if near_enemy.is_none() {
                            continue;
                        }

                        let mut view_angle = f32::acos(transform.forward().dot(
                            (near_enemy.unwrap().1.translation - transform.translation).normalize(),
                        ));

                        let sign = transform
                            .forward()
                            .normalize()
                            .cross(
                                (near_enemy.unwrap().1.translation - transform.translation)
                                    .normalize(),
                            )
                            .y
                            .signum();

                        view_angle *= sign;

                        if view_angle.is_nan() {
                            return;
                        }

                        transform.rotate(Quat::from_rotation_y(view_angle));
                        player.rotation += view_angle;

                        event_gun_shot.send(EventGunShot {
                            from: player.name.clone(),
                        })
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct BloodThirsty;

pub fn bloodthirsty_scorer_system(
    thirsts: Query<&BloodThirst>,
    mut query: Query<(&Actor, &mut Score), With<BloodThirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            score.set((thirst.enemies_near as f32) / 100.);
        }
    }
}

// -------------
// Training mode
// -------------

struct DelayedControlTimer(Timer);

fn turnbased_control_system_switch(
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<DelayedControlTimer>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.push(AppState::Control).unwrap();
        physics_time.pause();
    }
}

fn turnbased_control_system(
    keys: Res<Input<KeyCode>>,
    player_movement_q: Query<(&mut heron::prelude::Velocity, &Transform), With<PlayerPerspective>>,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    ai_gym_state: ResMut<Arc<Mutex<gym::AIGymState<PlayerActionFlags>>>>,
    mut app_state: ResMut<State<AppState>>,
    mut physics_time: ResMut<PhysicsTime>,
    player_query: Query<&Player>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();

    if keys.get_pressed().len() == 0 {
        return;
    }

    for key in keys.get_pressed() {
        if *key == KeyCode::W {
            ai_gym_state.action = PlayerActionFlags::FORWARD;
        } else if *key == KeyCode::A {
            ai_gym_state.action = PlayerActionFlags::BACKWARD;
        } else if *key == KeyCode::S {
            ai_gym_state.action = PlayerActionFlags::LEFT;
        } else if *key == KeyCode::D {
            ai_gym_state.action = PlayerActionFlags::RIGHT;
        } else if *key == KeyCode::Q {
            ai_gym_state.action = PlayerActionFlags::TURN_LEFT;
        } else if *key == KeyCode::E {
            ai_gym_state.action = PlayerActionFlags::TURN_RIGHT;
        } else if keys.just_pressed(KeyCode::Space) {
            ai_gym_state.action = PlayerActionFlags::SHOOT;
        }
    }

    let player = player_query.iter().find(|e| e.name == "Player 1").unwrap();
    ai_gym_state.reward.push(player.score as f32);

    let mut reward = ai_gym_state.reward[ai_gym_state.reward.len() - 1];
    if ai_gym_state.reward.len() > 1 {
        reward -= ai_gym_state.reward[ai_gym_state.reward.len() - 2];
    }

    println!("{:?}", ai_gym_state.action);
    println!("reward: {}", reward);
    println!("----");

    control_player(
        ai_gym_state.action,
        player_movement_q,
        collision_events,
        event_gun_shot,
    );

    app_state.pop().unwrap();
    physics_time.resume();
}

// -----------
// Entry Point
// -----------

fn main() {
    let mut bevy_app = build_game_app();

    bevy_app.run();
}

fn build_game_app() -> App {
    let args = Args::parse();

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
        .insert_resource(gym::AIGymSettings {
            width: 256,
            height: 256,
        })
        .insert_resource(Arc::new(Mutex::new(gym::AIGymState::<PlayerActionFlags> {
            action: PlayerActionFlags::IDLE,
            ..Default::default()
        })))
        .add_plugin(gym::AIGymPlugin::<PlayerActionFlags>::default())
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(DefaultRaycastingPlugin::<RaycastMarker>::default())
        .add_plugin(BigBrainPlugin)
        // State chain
        .add_system_set(
            SystemSet::on_enter(AppState::MainMenu)
                .with_system(spawn_ui_camera)
                .with_system(main_screen),
        )
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(button_system))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(clear_world))
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(spawn_ui_camera)
                .with_system(spawn_game_world)
                .with_system(spawn_player)
                .with_system(spawn_enemies)
                .with_system(draw_gun)
                .with_system(init_round),
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
            SystemSet::on_enter(AppState::RoundOver)
                .with_system(spawn_ui_camera)
                .with_system(round_over),
        )
        .add_system_set(SystemSet::on_update(AppState::RoundOver).with_system(button_system))
        .add_system_set(
            SystemSet::on_exit(AppState::RoundOver).with_system(clear_world.exclusive_system()),
        )
        // AI -- global due to
        .add_system(bloodthirst_system)
        .add_system_to_stage(BigBrainStage::Actions, kill_action_system)
        .add_system_to_stage(BigBrainStage::Scorers, bloodthirsty_scorer_system)
        // Initialize Resources
        .init_resource::<GameMap>()
        .init_resource::<GameAssets>();

    if args.mode == "train" {
        app.add_state(AppState::InGame);
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(check_termination_training)
                .with_system(turnbased_control_system_switch),
        );

        app.add_system_set(
            SystemSet::on_update(AppState::Control)
                // Game Systems
                .with_system(turnbased_control_system),
        );

        app.insert_resource(DelayedControlTimer(Timer::from_seconds(0.1, true)));
    } else if args.mode == "playtest" {
        app.add_state(AppState::InGame);
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(control_player_keyboard)
                .with_system(check_termination_training),
        );

        app.insert_resource(DelayedControlTimer(Timer::from_seconds(0.1, true)));
    }

    return app;

    // This branch would panic on current version
    // else {
    //     app.add_state(AppState::MainMenu);

    //     app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(draw_hud));
    //     app.add_system_set(
    //         SystemSet::on_update(AppState::InGame)
    //             .with_system(update_hud)
    //             .with_system(check_termination_play),
    //     );
    //     app.add_system_set(SystemSet::on_exit(AppState::InGame).with_system(clear_world));
    // }
}

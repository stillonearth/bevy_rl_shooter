use rand::thread_rng;
use rand::{seq::SliceRandom, Rng};
use std::ops::Range;

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastSource};
use heron::*;
use names::Generator;

pub mod map_parser;

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
    health: u8,
    score: u8,
}

#[derive(Component)]
struct PlayerPerspective;

#[derive(Component)]
struct PlayerAvatar;

#[derive(PartialEq)]
enum AnimationType {
    Standing,
    Walking,
    Shooting,
    Dying,
}

#[derive(Component)]
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

pub struct EventGunShot {
    from: String,
}

#[derive(Debug)]
pub struct EventDamage {
    from: String,
    to: String,
}

// ------
// States
// ------

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    RoundOver,
}

// -------
// Systems
// -------

// Main Menu

pub fn main_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("Roboto-Regular.ttf");

    let text = Text::with_section(
        "Royal Battle Bevystein",
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
                            "New Game",
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
    // Plane
    let maps = map_parser::load_maps("shareware/MAPHEAD.WL1", "shareware/GAMEMAPS.WL1", Some(1));
    let map = &maps[0];
    let size = map.width_n_tiles * map.height_n_tiles;
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
            size: Vec2::new(
                (100 * map.width_n_tiles) as f32,
                (100 * map.height_n_tiles) as f32,
            ),
            heights: vec![
                vec![100.5, 0.8, 0., 0., 3000.0],
                vec![0.8, 0.2, 0., 0., 300.0],
                vec![0., 0.5, 0., 0., 300.0],
                vec![0., 0., 0.6, 0., 300.0],
                vec![300., 300., 300., 300., 300.0],
            ],
        });

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
    commands.insert_resource(RoundTimer(Timer::from_seconds(100.0, false)));
}

pub fn spawn_player(mut commands: Commands, game_map: Res<GameMap>) {
    let mut rng = thread_rng();
    let pos = game_map.empty_space.choose(&mut rng).unwrap();
    let player = Player {
        position: (pos.0 as f32, pos.1 as f32),
        rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        name: "Player 1".to_string(),
        health: 100,
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
            cell.spawn_bundle(PerspectiveCameraBundle {
                ..Default::default()
            })
            .insert(RayCastSource::<RaycastMarker>::new_transform_empty());
        })
        .insert(CollisionShape::Sphere { radius: 1.0 })
        .insert(PlayerPerspective)
        .insert(Acceleration::from_linear(Vec3::ZERO))
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(Acceleration::from_linear(Vec3::ZERO))
        .insert(RigidBody::Dynamic)
        .insert(PhysicMaterial {
            density: 200.0,
            ..Default::default()
        })
        .insert(CollisionLayers::new(Layer::Player, Layer::World))
        .insert(RotationConstraints::lock())
        .insert(player);
}

pub fn spawn_enemies(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    wolfenstein_sprites: Res<GameAssets>,
) {
    for _ in 0..32 {
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

        commands
            .spawn_bundle((
                Transform {
                    translation: Vec3::new(
                        (player.position.0) as f32,
                        1.0,
                        (player.position.1) as f32,
                    ),
                    rotation: Quat::from_rotation_y(player.rotation),
                    ..Default::default()
                },
                GlobalTransform::identity(),
            ))
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
            })
            .insert(CollisionShape::Sphere { radius: 0.8 })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(Acceleration::from_linear(Vec3::ZERO))
            .insert(RigidBody::Dynamic)
            .insert(PhysicMaterial {
                density: 200.0,
                ..Default::default()
            })
            .insert(player)
            .insert(CollisionLayers::new(Layer::Player, Layer::World))
            .insert(RotationConstraints::lock());
    }
}

fn control_player(
    keys: Res<Input<KeyCode>>,
    mut player_movement_q: Query<
        (
            &mut heron::prelude::Velocity,
            &mut heron::prelude::Acceleration,
            &Transform,
        ),
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

    for (mut velocity, mut acceleration, transform) in player_movement_q.iter_mut() {
        *velocity = Velocity::from_linear(Vec3::ZERO);
        for key in keys.get_pressed() {
            if *key == KeyCode::W {
                *acceleration = Acceleration::from_linear(0.001 * transform.forward().normalize());
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.forward().normalize());
            }
            if *key == KeyCode::A {
                *acceleration = Acceleration::from_linear(0.001 * transform.left().normalize());
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.left().normalize());
            }
            if *key == KeyCode::S {
                *acceleration = Acceleration::from_linear(0.001 * -transform.forward().normalize());
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * -transform.forward().normalize());
            }
            if *key == KeyCode::D {
                *acceleration = Acceleration::from_linear(0.001 * transform.right().normalize());
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.right().normalize());
            }
            if *key == KeyCode::Q {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
            }
            if *key == KeyCode::E {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
            }
            if keys.just_pressed(KeyCode::Space) {
                event_gun_shot.send(EventGunShot {
                    from: "Player 1".to_string(),
                });
            }
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
        "GAME OVER",
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
                            "New Game",
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
    mut shooting_query: Query<&RayCastSource<RaycastMarker>>,
    player_query: Query<(&Children, &Player)>,

    mut gunshot_event: EventReader<EventGunShot>,
    mut event_damage: EventWriter<EventDamage>,

    mut wolfenstein_sprites: ResMut<GameAssets>,
) {
    for gunshot_event in gunshot_event.iter() {
        for (_, mut ui_image) in gun_sprite_query.iter_mut() {
            wolfenstein_sprites.gun_index = 1;
            ui_image.0 = wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                .clone()
                .into();
        }

        for e in shooting_query.iter_mut() {
            let r = e.intersect_top();
            if r.is_none() {
                return;
            }
            let hit_entity = r.unwrap().0;

            for (children, enemy) in player_query.iter() {
                let other_entity = children.iter().find(|c| c.id() == hit_entity.id());
                if other_entity.is_none() {
                    continue;
                }

                event_damage.send(EventDamage {
                    from: gunshot_event.from.clone(),
                    to: enemy.name.clone(),
                });

                return;
            }

            // despawn a wall
            commands.entity(hit_entity).despawn();
        }
    }
}

fn event_damage(
    mut commands: Commands,
    player_query: Query<(Entity, &Children, &mut Player)>,
    billboard_query: Query<(Entity, &EnemyAnimation, &Billboard)>,
    mut event_damage: EventReader<EventDamage>,
) {
    for damage_event in event_damage.iter() {
        let hit_entity = player_query.iter().find(|p| p.2.name == damage_event.to);
        if hit_entity.is_none() {
            continue;
        }
        let entity = hit_entity.unwrap().0;
        let children = hit_entity.unwrap().1;
        let mut hit_player = hit_entity.unwrap().2.clone();

        if hit_player.health == 0 {
            return;
        }

        hit_player.health -= 100;
        commands.entity(entity).insert(hit_player);

        for c in children.iter() {
            let child = billboard_query.iter().find(|c_| {
                return c_.0.id() == c.id();
            });

            if child.is_none() {
                continue;
            }

            let entity = child.unwrap().0;
            let animation = child.unwrap().1;

            commands
                .entity(entity)
                .insert(EnemyAnimation {
                    frame: 0,
                    handle: animation.handle.clone(),
                    animation_type: AnimationType::Dying,
                })
                .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
                .remove::<RayCastMesh<RaycastMarker>>();
        }

        let hit_entity = player_query.iter().find(|p| p.2.name == damage_event.from);

        let entity = hit_entity.unwrap().0;
        let mut hit_player = hit_entity.unwrap().2.clone();
        hit_player.score += 10;
        commands.entity(entity).insert(hit_player);
    }
}

// ---------
// Resources
// ---------

pub struct GameAssets {
    pub face: Vec<Handle<Image>>,
    pub face_index: u8,

    pub gun: Vec<Handle<Image>>,
    pub gun_index: u8,

    pub guard_billboard_material: Handle<StandardMaterial>,
    pub guard_walking_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_standing_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_dying_animation: Vec<Vec<[f32; 2]>>,

    pub font: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();

        let mut image_assets = world.get_resource_mut::<Assets<Image>>().unwrap();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        // face
        let mut face: Vec<Handle<Image>> = Vec::new();
        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1APIC)));
        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1BPIC)));
        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1CPIC)));

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
            face,
            gun,
            font: asset_server.load("Roboto-Regular.ttf"),
            guard_billboard_material,
            guard_walking_animation: gather_full_animation_uvs(Range { start: 2, end: 6 }),
            guard_standing_animation: gather_full_animation_uvs(Range { start: 1, end: 2 }),
            guard_dying_animation: gather_row_animation_uvs(6.0, Range { start: 1, end: 6 }, 0.0),
            face_index: 0,
            gun_index: 0,
        };
    }
}

pub struct GameMap {
    pub empty_space: Vec<(usize, usize)>,
    pub walls: Vec<(usize, usize)>,
}

impl FromWorld for GameMap {
    fn from_world(_: &mut World) -> Self {
        let maps =
            map_parser::load_maps("shareware/MAPHEAD.WL1", "shareware/GAMEMAPS.WL1", Some(1));
        let map = &maps[0];

        let mut game_map = GameMap {
            empty_space: Vec::new(),
            walls: Vec::new(),
        };

        // Cubes as walls
        map.plane0
            .as_ref()
            .unwrap()
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .enumerate()
            .for_each(|(word_i, word)| {
                let x = word_i % usize::from(map.width_n_tiles) * 2;
                let z = word_i / usize::from(map.height_n_tiles) * 2;

                if word == 90 {
                } else if word == 91 {
                } else if word < 107 {
                    game_map.walls.push((x, z))
                } else {
                    game_map.empty_space.push((x, z))
                }
            });

        return game_map;
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

fn animate_face(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<GameAssets>,
    mut query: Query<(&PlayerAvatar, &mut AnimationTimer, &mut UiImage)>,
) {
    for (_, mut timer, mut ui_image) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            wolfenstein_sprites.face_index += 1;
            if wolfenstein_sprites.face_index >= (wolfenstein_sprites.face.len() as u8) {
                wolfenstein_sprites.face_index = 0;
            }

            ui_image.0 = wolfenstein_sprites.face[wolfenstein_sprites.face_index as usize]
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
    let player_vector = Vec3::X;
    let player_position = player_transform.translation;

    for (mut timer, parent, mut animation) in q.q0().iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // 2D animations

            let mut animations: Vec<Vec<[f32; 2]>> = Vec::new();
            if animation.animation_type == AnimationType::Dying {
                animations = wolfenstein_sprites.guard_dying_animation.clone();
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
                let enemy_vector = parent_transform.forward().normalize();
                let enemy_position = parent_transform.translation;

                let mut angle =
                    f32::acos(player_vector.dot(enemy_vector)) * 180.0 / std::f32::consts::PI;
                if angle < 0.0 {
                    angle += 360.0;
                }

                let delta_z = enemy_position.z - player_position.z;
                let delta_x = enemy_position.x - player_position.x;

                angle += delta_x.atan2(delta_z) * 180.0 / std::f32::consts::PI;
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

                mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            }

            if animation.frame >= (animations.len() as u8 - 1) {
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
            font_size: 75.0,
            font: game_assets.font.clone(),
            color: Color::rgb(1.0, 1.0, 1.0),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(180.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::BLUE.clone().into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(180.0)),
                        position_type: PositionType::Relative,
                        position: Rect {
                            left: Val::Px(45.0),
                            bottom: Val::Px(52.5),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ScoreText);

            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(320.), Val::Px(180.)),
                        ..Default::default()
                    },
                    image: game_assets.face[game_assets.face_index as usize]
                        .clone()
                        .into(),
                    ..Default::default()
                })
                .insert(PlayerAvatar)
                .insert(AnimationTimer(Timer::from_seconds(2.0, true)));

            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(180.0)),
                        position_type: PositionType::Relative,
                        position: Rect {
                            left: Val::Px(45.0),
                            bottom: Val::Px(52.5),
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

    for (mut text) in score_text_query.iter_mut() {
        let str = format!("SCORE {}", player_1.unwrap().score).to_string();
        text.sections[0].value = str;
    }

    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    for mut text in time_left_query.iter_mut() {
        let str = format!("TIME LEFT {}", seconds_left).to_string();
        text.sections[0].value = str;
    }
}

fn check_termination(
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

fn draw_gun(mut commands: Commands, wolfenstein_sprites: Res<GameAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(65.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(180.),
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

// -----------
// Entry Point
// -----------

fn main() {
    App::new()
        // Resources
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(DefaultPluginState::<RaycastMarker>::default())
        // Events
        .add_event::<EventGunShot>()
        .add_event::<EventDamage>()
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(DefaultRaycastingPlugin::<RaycastMarker>::default())
        // State chain
        .add_state(AppState::MainMenu)
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
                .with_system(draw_hud)
                .with_system(draw_gun)
                .with_system(init_round),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                // Game Systems
                .with_system(check_termination)
                .with_system(control_player)
                .with_system(animate_face)
                .with_system(animate_gun)
                .with_system(animate_enemy)
                .with_system(render_billboards)
                .with_system(update_hud)
                // Event handlers
                .with_system(event_gun_shot)
                .with_system(event_damage),
        )
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(clear_world))
        .add_system_set(
            SystemSet::on_enter(AppState::RoundOver)
                .with_system(spawn_ui_camera)
                .with_system(round_over),
        )
        .add_system_set(SystemSet::on_update(AppState::RoundOver).with_system(button_system))
        .add_system_set(
            SystemSet::on_exit(AppState::RoundOver).with_system(clear_world.exclusive_system()),
        )
        // Initialize Resources
        .init_resource::<GameMap>()
        .init_resource::<GameAssets>()
        .run();
}

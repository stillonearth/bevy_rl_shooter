use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastSource, SimplifiedMesh,
};

pub mod map_parser;

use heron::*;

#[derive(PhysicsLayer)]
enum Layer {
    World,
    Player,
    Enemies,
}

#[derive(Component)]
struct PlayerCamera;
#[derive(Component)]
struct Player;

pub struct EventGunShot {
    value: usize,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.2, 0.8)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0))) // Optionally define gravity
        .add_startup_system(spawn_game_world)
        .add_startup_system(spawn_player)
        .add_startup_system(setup_camera)
        .add_startup_system(draw_hud)
        .add_startup_system(draw_gun)
        .add_plugin(WorldInspectorPlugin::new())
        .init_resource::<WolfensteinSprites>()
        .add_system(control_player)
        .add_system(animate_face)
        .add_system(animate_gun)
        .add_system(event_gun_shot)
        .add_event::<EventGunShot>()
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .run();
}

fn spawn_game_world(
    mut commands: Commands,
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
            material: materials.add(Color::GREEN.into()),
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

    let plane = map.plane0.as_ref().unwrap();

    // Cubes as walls
    plane
        .chunks_exact(2)
        .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
        .enumerate()
        .for_each(|(word_i, word)| {
            let x = word_i % usize::from(map.width_n_tiles) * 2;
            let z = word_i / usize::from(map.height_n_tiles) * 2;
            let y = 1.0 as usize;

            if word == 90 {
            } else if word == 91 {
            } else if word < 107 {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                        material: materials.add(Color::RED.into()),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32, y as f32, z as f32,
                        )),
                        global_transform: GlobalTransform::identity(),
                        ..Default::default()
                    })
                    .insert(RigidBody::Static)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(1.0, 1.0, 1.0),
                        border_radius: None,
                    })
                    .insert(CollisionLayers::new(Layer::World, Layer::Player))
                    .insert(RayCastMesh::<MyRaycastSet>::default()); // Make this mesh ray cast-able
            }
        });
}

struct MyRaycastSet;

fn setup_camera(mut commands: Commands) {
    let mut camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        Vec3::new(0.0, 10.0, 0.0),
    ));

    // camera_transform.scale.z = 1.5;

    commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .insert(RayCastSource::<MyRaycastSet>::new_transform_empty())
        .insert(PlayerCamera);
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle((
            Transform {
                translation: Vec3::new(33.0, 1.0, 33.0),
                rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .with_children(|cell| {
            cell.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 200.0,
                    shadows_enabled: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            // cell.spawn_scene(asset_server.load("craft_speederA.glb#Scene0"));
        })
        .insert(CollisionShape::Sphere { radius: 1.0 })
        .insert(Player)
        .insert(Acceleration::from_linear(Vec3::X * 0.0))
        .insert(Velocity::from_linear(Vec3::X * 0.0))
        .insert(Acceleration::from_linear(Vec3::X * 0.0))
        .insert(RigidBody::Dynamic)
        .insert(PhysicMaterial {
            density: 200.0,
            ..Default::default()
        })
        .insert(CollisionLayers::new(Layer::Player, Layer::World))
        .insert(RotationConstraints::lock());
}

fn control_player(
    keys: Res<Input<KeyCode>>,
    mut velocities: Query<
        (
            &mut heron::prelude::Velocity,
            &mut heron::prelude::Acceleration,
            &Transform,
        ),
        Without<PlayerCamera>,
    >,
    mut cameras: Query<(&mut Transform), With<PlayerCamera>>,
    mut collison_events: EventReader<CollisionEvent>,
    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    for (mut velocity, mut acceleration, transform) in velocities.iter_mut() {
        *velocity = Velocity::from_linear(Vec3::X * 0.0);
        for key in keys.get_pressed() {
            if *key == KeyCode::W {
                *acceleration = Acceleration::from_linear(0.001 * transform.forward().normalize());
                *velocity = Velocity::from_linear(10. * transform.forward().normalize());
            }
            if *key == KeyCode::S {
                *acceleration = Acceleration::from_linear(0.001 * -transform.forward().normalize());
                *velocity = Velocity::from_linear(10. * -transform.forward().normalize());
            }
            if *key == KeyCode::A {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
            }
            if *key == KeyCode::D {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
            }
            if keys.just_pressed(KeyCode::LControl) {
                event_gun_shot.send(EventGunShot { value: 0 });
            }
        }

        collison_events
            .iter()
            // We care about when the entities "start" to collide
            // .filter(|e| e.)
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

        for mut _tr in cameras.iter_mut() {
            _tr.translation.x = transform.translation.x;
            _tr.translation.z = transform.translation.z;
            _tr.translation.y = 1.0;
            _tr.rotation = Quat::from_rotation_arc(-Vec3::Z, transform.forward().normalize());
        }
    }
}

fn is_player(layers: CollisionLayers) -> bool {
    layers.contains_group(Layer::Player) && !layers.contains_group(Layer::World)
}

fn is_world(layers: CollisionLayers) -> bool {
    !layers.contains_group(Layer::Player) && layers.contains_group(Layer::World)
}

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Weapon;

fn draw_hud(mut commands: Commands, wolfenstein_sprites: Res<WolfensteinSprites>) {
    commands.spawn_bundle(UiCameraBundle::default());

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
            // texture_atlas: texture_atlas_handle,
            ..Default::default()
        })
        .with_children(|parent| {
            // bevy logo (image)
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Px(180.)),
                        ..Default::default()
                    },
                    image: wolfenstein_sprites.face[wolfenstein_sprites.face_index as usize]
                        .clone()
                        .into(),
                    ..Default::default()
                })
                .insert(Player)
                .insert(AnimationTimer(Timer::from_seconds(2.0, true)));
        });
}

fn draw_gun(mut commands: Commands, wolfenstein_sprites: Res<WolfensteinSprites>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(450.0)),
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
            // bevy logo (image)
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

impl FromWorld for WolfensteinSprites {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut image_assets = world.get_resource_mut::<Assets<Image>>().unwrap();

        let mut face: Vec<Handle<Image>> = Vec::new();

        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1APIC)));
        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1BPIC)));
        face.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1CPIC)));

        let mut asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        let mut gun: Vec<Handle<Image>> = Vec::new();

        gun.push(asset_server.load("gun/gun_0.png"));
        gun.push(asset_server.load("gun/gun_1.png"));
        gun.push(asset_server.load("gun/gun_2.png"));

        return Self {
            face,
            gun,
            face_index: 0,
            gun_index: 0,
        };
    }
}

pub struct WolfensteinSprites {
    pub face: Vec<Handle<Image>>,
    pub face_index: u8,
    pub gun: Vec<Handle<Image>>,
    pub gun_index: u8,
}

fn animate_face(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<WolfensteinSprites>,
    mut query: Query<(&Player, &mut AnimationTimer, &mut UiImage)>,
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

fn event_gun_shot(
    mut commands: Commands,
    mut wolfenstein_sprites: ResMut<WolfensteinSprites>,
    mut gunshot_events: EventReader<EventGunShot>,
    mut query: Query<(&Weapon, &mut UiImage)>,
    mut shooting_query: Query<(Entity, Option<&SimplifiedMesh>), With<RayCastSource<MyRaycastSet>>>,
) {
    for e in gunshot_events.iter() {
        for (_, mut ui_image) in query.iter_mut() {
            wolfenstein_sprites.gun_index = 1;
            ui_image.0 = wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                .clone()
                .into();
        }

        if let Ok((entity, ray)) = shooting_query.get_single() {
            if let Ok(mut text) = status_query.get_single_mut() {
                if ray.is_none() {
                    commands.entity(entity).insert(SimplifiedMesh {
                        mesh: meshes.add(Mesh::from(shape::UVSphere::default())),
                    });
                    text.sections[1].value = "ON".to_string();
                    text.sections[1].style.color = Color::GREEN;
                } else {
                    commands.entity(entity).remove::<SimplifiedMesh>();
                    text.sections[1].value = "OFF".to_string();
                    text.sections[1].style.color = Color::RED;
                }
            }
        }

        for e in shooting_query.iter_mut() {
            // commands.entity(e).despawn();
            println!("{:?}", e);
        }
    }
}

fn animate_gun(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<WolfensteinSprites>,
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

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0))) // Optionally define gravity
        .add_startup_system(spawn_game_world)
        .add_startup_system(spawn_player)
        .add_startup_system(setup_camera)
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(move_player)
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
                        ..Default::default()
                    })
                    .insert(GlobalTransform::default())
                    .insert(RigidBody::Dynamic)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(1.0, 1.0, 1.0),
                        border_radius: None,
                    });
            } else {
                if (x + y) % 10 == 0 {
                    commands.spawn_bundle(PointLightBundle {
                        point_light: PointLight {
                            intensity: 500.0,
                            shadows_enabled: false,
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                        ..Default::default()
                    });
                }
            }
        });
}

fn setup_camera(mut commands: Commands) {
    let mut camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        Vec3::new(0.0, 10.0, 0.0),
    ));

    camera_transform.scale.z = 1.5;

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_transform,
            ..Default::default()
        })
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
                translation: Vec3::new(33.0, 5.0, 33.0),
                rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .with_children(|cell| {
            cell.spawn_scene(asset_server.load("craft_speederA.glb#Scene0"));
        })
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 1.0, 1.0),
            border_radius: None,
        })
        .insert(Velocity::from_linear(Vec3::X * 0.0))
        .insert(RigidBody::Dynamic);
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    mut velocities: Query<(&mut Velocity, &Transform), Without<PlayerCamera>>,
    mut cameras: Query<(&mut Transform), With<PlayerCamera>>,
) {
    for (mut velocity, transform) in velocities.iter_mut() {
        *velocity = Velocity::from_linear(Vec3::X * 0.0);
        for key in keys.get_pressed() {
            if *key == KeyCode::Up {
                *velocity = Velocity::from_linear(10. * transform.forward().normalize());
            }
            if *key == KeyCode::Down {
                *velocity = Velocity::from_linear(10. * -transform.forward().normalize());
            }
            if *key == KeyCode::Left {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
            }
            if *key == KeyCode::Right {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
            }
        }

        for mut _tr in cameras.iter_mut() {
            _tr.translation.x = transform.translation.x;
            _tr.translation.z = transform.translation.z;
            _tr.translation.y = 1.0;

            let fw_vec = transform.forward().normalize();

            _tr.rotation = Quat::from_rotation_arc(-Vec3::Z, transform.forward().normalize());
        }
    }
}

use bevy::prelude::*;
use bevy_config_cam::*;

pub mod map_parser;

use heron::*;

#[derive(PhysicsLayer)]
enum Layer {
    World,
    Player,
    Enemies,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0))) // Optionally define gravity
        .add_startup_system(spawn_game_world.system())
        .add_plugin(ConfigCam)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 15.0,          // default: 12.0
            dist: 30.0,           // Camera distance from the player in topdown view
            ..Default::default()
        })
        .insert_resource(PlayerSettings {
            pos: Vec3::new(2., 0., 0.),                //Initial position of the player
            player_asset: "craft_speederA.glb#Scene0", //Model of the player, default is a red cube
            ..Default::default()
        })
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

    let plane = map.plane0.as_ref().unwrap();

    // Cubes as walls
    plane
        .chunks_exact(2)
        .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
        .enumerate()
        .for_each(|(word_i, word)| {
            let x = word_i % usize::from(map.width_n_tiles) * 2;
            let z = word_i / usize::from(map.height_n_tiles) * 2;
            let y = 5.0 as usize;

            if word == 90 {
            } else if word == 91 {
            } else if word < 107 {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                        material: materials.add(Color::BLACK.into()),
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
            }
        });
}

fn spawn_ground_and_camera(mut commands: Commands) {}

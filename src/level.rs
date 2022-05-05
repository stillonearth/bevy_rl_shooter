use bevy::prelude::*;
use bevy_mod_raycast::RayCastMesh;
use heron::*;
use serde::{Deserialize, Serialize};

use crate::{game::*, map, physics::*};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct GameMap {
    pub empty_space: Vec<(usize, usize)>,
    pub walls: Vec<(usize, usize)>,
}

impl FromWorld for GameMap {
    fn from_world(_: &mut World) -> Self {
        let deserialized: GameMap = serde_json::from_str(&map::JSON).unwrap();
        return deserialized;
    }
}

#[derive(Component)]
pub(crate) struct Wall;

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    pbr_pundle: PbrBundle,
    rigid_body: RigidBody,
    collision_shape: CollisionShape,
    collision_layers: CollisionLayers,
    wall: Wall,
    raycast_marker: RayCastMesh<RaycastMarker>,
}

pub(crate) fn spawn_game_world(
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

    let walls_iter: Vec<WallBundle> = game_map
        .walls
        .iter()
        .map(|(x, z)| {
            return WallBundle {
                pbr_pundle: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                    material: white_material_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(*x as f32, 1.0, *z as f32)),
                    global_transform: GlobalTransform::identity(),
                    ..Default::default()
                },
                rigid_body: RigidBody::Static,
                collision_shape: CollisionShape::Cuboid {
                    half_extends: Vec3::new(1.0, 1.0, 1.0),
                    border_radius: None,
                },
                collision_layers: CollisionLayers::new(Layer::World, Layer::Player),
                raycast_marker: RayCastMesh::<RaycastMarker>::default(),
                wall: Wall,
            };
        })
        .collect();

    commands.spawn_batch(walls_iter);
}

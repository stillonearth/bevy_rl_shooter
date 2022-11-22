use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;
use bevy_rapier3d::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{game::*, map};

#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct GameMap {
    pub empty_space: Vec<(usize, usize)>,
    pub walls: Vec<(usize, usize)>,
}

impl Default for GameMap {
    fn default() -> Self {
        let deserialized: GameMap = serde_json::from_str(map::JSON).unwrap();
        deserialized
    }
}

#[derive(Component)]
pub(crate) struct Wall;

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    pbr_pundle: PbrBundle,
    rigid_body: RigidBody,
    collider: Collider,
    wall: Wall,
    raycast_marker: RaycastMesh<RaycastMarker>,
}

pub(crate) fn spawn_game_world(
    mut commands: Commands,
    game_map: Res<GameMap>,
    walls: Query<Entity, With<Wall>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = 255.0 * 255.0;
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: (size as f32),
    }));

    let white_material_handle = materials.add(Color::WHITE.into());

    // spawn floor only once
    if walls.iter().len() == 0 {
        commands
            .spawn(PbrBundle {
                mesh,
                material: white_material_handle.clone(),
                ..Default::default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(256.0, 1.0, 256.0));
    }

    let wall_mesh = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
    let walls_iter: Vec<WallBundle> = game_map
        .walls
        .iter()
        .map(|(x, z)| {
            WallBundle {
                pbr_pundle: PbrBundle {
                    mesh: wall_mesh.clone(),
                    material: white_material_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(*x as f32, 1.0, *z as f32)),
                    global_transform: GlobalTransform::IDENTITY,
                    ..Default::default()
                },
                rigid_body: RigidBody::Fixed,
                collider: Collider::cuboid(1.0, 1.0, 1.0),
                raycast_marker: RaycastMesh::<RaycastMarker>::default(),
                wall: Wall,
            }
        })
        .collect();

    commands.spawn_batch(walls_iter);
}

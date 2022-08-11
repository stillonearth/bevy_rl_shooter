use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::render::camera::RenderTarget;
use bevy_rl::AIGymSettings;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_mod_raycast::RayCastSource;
use bevy_rl::state::AIGymState;
use heron::*;

use names::Generator;

use crate::{actions::*, game::*, level::*, physics::*};

#[derive(Component, Clone)]
pub(crate) struct Actor {
    pub position: (f32, f32),
    pub rotation: f32,
    pub name: String,
    pub health: u16,
    pub score: u16,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerPerspective;

#[derive(Bundle)]
pub(crate) struct ActorBundle {
    collision_layers: CollisionLayers,
    collision_shape: CollisionShape,
    global_transform: GlobalTransform,
    actor: Actor,
    rigid_body: RigidBody,
    rotation_constraints: RotationConstraints,
    transform: Transform,
    velocity: Velocity,
    physics_material: PhysicMaterial,
}

fn new_actor_bundle(game_map: GameMap, actor_name: String) -> ActorBundle {
    let mut rng = thread_rng();
    let pos = game_map.empty_space.choose(&mut rng).unwrap();

    let actor = Actor {
        position: (pos.0 as f32, pos.1 as f32),
        rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        name: actor_name,
        health: 100,
        score: 0,
    };

    return ActorBundle {
        transform: Transform {
            translation: Vec3::new(actor.position.0 as f32, 1.0, actor.position.1 as f32),
            rotation: Quat::from_rotation_y(actor.rotation),
            ..Default::default()
        },
        global_transform: GlobalTransform::identity(),
        velocity: Velocity::from_linear(Vec3::ZERO),
        collision_shape: CollisionShape::Sphere { radius: 1.0 },
        rigid_body: RigidBody::Dynamic,
        physics_material: PhysicMaterial {
            density: 200.0,
            ..Default::default()
        },
        collision_layers: CollisionLayers::new(Layer::Player, Layer::World),
        actor: actor,
        rotation_constraints: RotationConstraints::lock(),
    };
}

#[derive(Bundle)]
struct ActorWeaponBundle {
    #[bundle]
    pbr_bundle: PbrBundle,
    raycast_source: RayCastSource<RaycastMarker>,
}

fn new_actor_weapon_bundle(mesh: Handle<Mesh>) -> ActorWeaponBundle {
    return ActorWeaponBundle {
        pbr_bundle: PbrBundle {
            mesh: mesh,
            transform: Transform {
                translation: Vec3::ZERO,
                ..Default::default()
            },
            // visibility: Visibility { is_visible: false },
            ..Default::default()
        },
        raycast_source: RayCastSource::<RaycastMarker>::new_transform_empty(),
    };
}

pub(crate) fn spawn_computer_actors(
    mut commands: Commands,
    game_map: Res<GameMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    ai_gym_settings: Res<AIGymSettings>,
    ai_gym_state: Res<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();

    for i in 0..ai_gym_settings.num_agents {
        let actor_bundle = new_actor_bundle(game_map.clone(), Generator::default().next().unwrap());

        commands.spawn_bundle(actor_bundle).with_children(|cell| {
            // Weapon
            let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.8, 1.7))));
            let actor_weapon_bundle = new_actor_weapon_bundle(mesh);
            cell.spawn_bundle(actor_weapon_bundle);

            // Model
            let red_material_handle = materials.add(Color::RED.into());
            let mesh = meshes.add(Mesh::from(shape::UVSphere {
                sectors: 128,
                stacks: 64,
                ..default()
            }));
            cell.spawn_bundle(PbrBundle {
                mesh,
                material: red_material_handle.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(10.0)),
                ..Default::default()
            });

            // Camera
            cell.spawn_bundle(Camera3dBundle {
                camera: Camera {
                    target: RenderTarget::Image(
                        ai_gym_state.render_image_handles[i as usize].clone(),
                    ),
                    ..default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::WHITE),
                    ..default()
                },
                ..default()
            });
        });
    }
}

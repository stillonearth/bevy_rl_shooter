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

// Components

#[derive(Component, Clone)]
pub(crate) struct Actor {
    pub position: (f32, f32),
    pub rotation: f32,
    pub name: String,
    pub health: u16,
    pub score: u16,
}

// Bundles

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

#[derive(Bundle)]
struct ActorWeaponBundle {
    #[bundle]
    camera_bundle: Camera3dBundle,
    raycast_source: RayCastSource<RaycastMarker>,
}

// Constructors

fn new_agent_bundle(
    game_map: GameMap,
    actor_name: String,
) -> ActorBundle {
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

fn new_agent_camera_bundle(render_target: RenderTarget) -> ActorWeaponBundle {
    return ActorWeaponBundle {
        camera_bundle: Camera3dBundle {
            camera: Camera {
                target: render_target,
                priority: -1,
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
                ..default()
            },
            ..default()
        },
        raycast_source: RayCastSource::<RaycastMarker>::new_transform_empty(),
    };
}

// Systems

pub(crate) fn spawn_computer_actors(
    mut commands: Commands,
    game_map: Res<GameMap>,
    ai_gym_settings: Res<AIGymSettings>,
    ai_gym_state: Res<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();

    let red_material_handle = materials.add(Color::RED.into());
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    for i in 0..ai_gym_settings.num_agents {
        let agent_bundle = new_agent_bundle(
            game_map.clone(),
            Generator::default().next().unwrap(),
        );

        commands.spawn_bundle(agent_bundle).with_children(|cell| {
            // Agent model
            let pbr_bundle = PbrBundle {
                mesh: mesh.clone(),
                material: red_material_handle.clone(),
                ..default()
            };
            cell.spawn_bundle(pbr_bundle);

            // Camera
            let agent_camera_bundle = new_agent_camera_bundle(RenderTarget::Image(
                ai_gym_state.render_image_handles[i as usize].clone(),
            ));
            cell.spawn_bundle(agent_camera_bundle);
        });
    }
}

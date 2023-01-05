use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::RenderTarget,
};

use bevy_mod_raycast::{RaycastMesh, RaycastSource};
use bevy_rapier3d::prelude::*;
use bevy_rl::*;

use names::Generator;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use serde::Serialize;

use crate::gym::EnvironmentState;
use crate::{actions::*, game::*, level::*};

// Components

#[derive(Component, Clone, Serialize)]
pub struct Actor {
    pub position: (f32, f32),
    pub rotation: f32,
    pub name: String,
    pub health: u16,
}

// Bundles

#[derive(Bundle)]
pub(crate) struct ActorBundle {
    collider: Collider,
    actor: Actor,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    velocity: Velocity,
    #[bundle]
    spacial_bundle: SpatialBundle,
}

#[derive(Bundle)]
struct ActorWeaponBundle {
    #[bundle]
    camera_bundle: Camera3dBundle,
    raycast_source: RaycastSource<RaycastMarker>,
}

// Constructors
fn new_agent_bundle(game_map: GameMap, actor_name: String) -> ActorBundle {
    let mut rng = thread_rng();
    let pos = game_map.empty_space.choose(&mut rng).unwrap();

    let actor = Actor {
        position: (pos.0 as f32, pos.1 as f32),
        rotation: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
        name: actor_name,
        health: 100,
    };

    ActorBundle {
        spacial_bundle: SpatialBundle {
            transform: Transform {
                translation: Vec3::new(actor.position.0 as f32, 1.0, actor.position.1 as f32),
                rotation: Quat::from_rotation_y(actor.rotation),
                ..Default::default()
            },
            visibility: Visibility { is_visible: true },
            ..Default::default()
        },
        velocity: Velocity { ..default() },
        collider: Collider::ball(1.0),
        rigid_body: RigidBody::Dynamic,
        actor,
        locked_axes: (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
    }
}

fn new_agent_camera_bundle(render_target: RenderTarget) -> ActorWeaponBundle {
    ActorWeaponBundle {
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
        raycast_source: RaycastSource::<RaycastMarker>::new_transform_empty(),
    }
}

// Systems
pub(crate) fn spawn_computer_actors(
    mut commands: Commands,
    game_map: Res<GameMap>,
    ai_gym_state: Res<AIGymState<Actions, EnvironmentState>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ai_gym_state = ai_gym_state.lock().unwrap();
    let ai_gym_settings = ai_gym_state.settings.clone();
    let material = materials.add(Color::RED.into());
    let mesh = meshes.add(Mesh::from(shape::UVSphere {
        sectors: 128,
        stacks: 64,
        ..default()
    }));

    let mut actors: Vec<Actor> = Vec::new();
    for i in 0..ai_gym_settings.num_agents {
        let agent_bundle = new_agent_bundle(game_map.clone(), Generator::default().next().unwrap());

        actors.push(agent_bundle.actor.clone());
        commands.spawn(agent_bundle).with_children(|cell| {
            // Agent model
            cell.spawn(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_scale(Vec3::splat(0.33)),
                ..default()
            })
            .insert(RaycastMesh::<RaycastMarker>::default());
            // Camera
            let agent_camera_bundle = new_agent_camera_bundle(RenderTarget::Image(
                ai_gym_state.render_image_handles[i as usize].clone(),
            ));
            cell.spawn(agent_camera_bundle);
        });
    }
    let env_state = EnvironmentState {
        map: game_map.clone(),
        actors,
    };
    ai_gym_state.set_env_state(env_state);
}

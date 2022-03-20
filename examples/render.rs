use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use bevystein::gym::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        // Use 4x MSAA
        .add_plugins(DefaultPlugins)
        .insert_resource(AIGymSettings {
            width: 512,
            height: 512,
        })
        .insert_resource(Arc::new(Mutex::new(AIGymState::<CubeAction> {
            ..Default::default()
        })))
        .add_plugin(AIGymPlugin::<CubeAction>::default());

    app.add_startup_system(setup.after("setup_rendering"));
    app.add_system(rotator_system);

    app.run();
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct RotatingCube;

#[derive(Default, Clone)]

struct CubeAction;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ai_gym_state: Res<Arc<Mutex<AIGymState<CubeAction>>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ai_gym_state = ai_gym_state.lock().unwrap();
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // The cube that will be rendered to the texture.
    commands
        .spawn_bundle(PbrBundle {
            mesh: cube_handle,
            material: cube_material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        })
        .insert(RotatingCube);

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle::<FirstPassCamera> {
        camera: Camera {
            target: ai_gym_state.__render_target.clone().unwrap(),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..PerspectiveCameraBundle::new()
    });
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.5 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_z(1.3 * time.delta_seconds());
    }
}

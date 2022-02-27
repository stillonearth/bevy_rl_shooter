use bevy::prelude::*;
use bevystein;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .init_resource::<WolfenstainResources>()
        .run();
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn setup(mut commands: Commands, wolfenstein_sprites: Res<WolfenstainResources>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            // texture_atlas: texture_atlas_handle,
            ..Default::default()
        })
        .with_children(|parent| {
            // bevy logo (image)
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(500.0), Val::Auto),
                        ..Default::default()
                    },
                    image: wolfenstein_sprites.faces[0].clone().into(),
                    ..Default::default()
                })
                .insert(AnimationTimer(Timer::from_seconds(2.0, true)));
        });
}

fn animate_sprite(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<WolfenstainResources>,
    mut query: Query<(&mut AnimationTimer, &mut UiImage)>,
) {
    for (mut timer, mut ui_image) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            wolfenstein_sprites.current_index += 1;
            if wolfenstein_sprites.current_index >= (wolfenstein_sprites.faces.len() as u8) {
                wolfenstein_sprites.current_index = 0;
            }

            ui_image.0 = wolfenstein_sprites.faces[wolfenstein_sprites.current_index as usize]
                .clone()
                .into();
        }
    }
}

pub struct WolfenstainResources {
    pub faces: Vec<Handle<Image>>,
    pub gun: Handle<Image>,
    pub current_index: u8,
}

impl FromWorld for WolfenstainResources {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut image_assets = world.get_resource_mut::<Assets<Image>>().unwrap();

        let mut faces: Vec<Handle<Image>> = Vec::new();
        faces.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1APIC)));
        faces.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1BPIC)));
        faces.push(image_assets.add(bevystein::elden::get_image(bevystein::cache::FACE1CPIC)));

        return Self {
            faces,
            gun: image_assets.add(bevystein::elden::get_image(bevystein::cache::GUNPIC)),
            current_index: 0,
        };
    }
}

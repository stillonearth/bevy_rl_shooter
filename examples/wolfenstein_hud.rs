use bevy::prelude::*;
use bevystein;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .init_resource::<BlazkowiczFaces>()
        .run();
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn setup(mut commands: Commands, blazkowicz_faces: Res<BlazkowiczFaces>) {
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
                    image: blazkowicz_faces.sprites[0].clone().into(),
                    ..Default::default()
                })
                .insert(AnimationTimer(Timer::from_seconds(2.0, true)));
        });
}

fn animate_sprite(
    time: Res<Time>,
    mut blazkowicz_faces: ResMut<BlazkowiczFaces>,
    mut query: Query<(&mut AnimationTimer, &mut UiImage)>,
) {
    for (mut timer, mut ui_image) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            blazkowicz_faces.current_index += 1;
            if blazkowicz_faces.current_index >= (blazkowicz_faces.sprites.len() as u8) {
                blazkowicz_faces.current_index = 0;
            }

            ui_image.0 = blazkowicz_faces.sprites[blazkowicz_faces.current_index as usize]
                .clone()
                .into();

            // let face_handles = textures.ids();

            // ui_image.0 =
            // texture_atlas.sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

pub struct BlazkowiczFaces {
    pub sprites: Vec<Handle<Image>>,
    pub current_index: u8,
}

impl FromWorld for BlazkowiczFaces {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut image_assets = world.get_resource_mut::<Assets<Image>>().unwrap();

        let mut sprites: Vec<Handle<Image>> = Vec::new();

        let face_sprite_1 = bevystein::elden::get_image(bevystein::cache::FACE1APIC);
        let face_sprite_2 = bevystein::elden::get_image(bevystein::cache::FACE1BPIC);
        let face_sprite_3 = bevystein::elden::get_image(bevystein::cache::FACE1CPIC);

        sprites.push(image_assets.add(face_sprite_1));
        sprites.push(image_assets.add(face_sprite_2));
        sprites.push(image_assets.add(face_sprite_3));

        return Self {
            sprites,
            current_index: 0,
        };
    }
}

use bevy::prelude::*;
use bevystein;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .run();
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let face_sprite_1 = bevystein::elden::get_image(bevystein::cache::FACE1APIC);
    let face_sprite_2 = bevystein::elden::get_image(bevystein::cache::FACE1BPIC);
    let face_sprite_3 = bevystein::elden::get_image(bevystein::cache::FACE1CPIC);

    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.add_texture(textures.add(face_sprite_1.clone()), &face_sprite_1);
    texture_atlas_builder.add_texture(textures.add(face_sprite_2.clone()), &face_sprite_2);
    texture_atlas_builder.add_texture(textures.add(face_sprite_3.clone()), &face_sprite_3);

    let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(10.),
                top: Val::Px(10.),
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle,
            ..Default::default()
        },
        ..Default::default()
    });
    // .with_children(|parent| {
    //     parent
    //         .spawn_bundle(SpriteSheetBundle {
    //             transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
    //             ..Default::default()
    //         })
    //         .insert(Style::default())
    //         .insert(CalculatedSize {
    //             size: Size::new(
    //                 (face_sprite_1.texture_descriptor.size.width as f32) * 10.,
    //                 (face_sprite_1.texture_descriptor.size.height as f32) * 10.,
    //             ),
    //         })
    //         .insert(Node::default())
    //         .insert(AnimationTimer(Timer::from_seconds(2.0, true)));
    // });
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

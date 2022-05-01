use bevy::prelude::*;

use crate::{animations::*, assets::*, game::*, player::*, screens::*};

#[derive(Component)]
pub(crate) struct ScoreText;

#[derive(Component)]
pub(crate) struct TimeLeftText;

#[derive(Component)]
pub(crate) struct Weapon;

pub(crate) fn draw_hud(mut commands: Commands, game_assets: Res<GameAssets>) {
    let text = Text::with_section(
        "",
        TextStyle {
            font_size: 45.0,
            font: game_assets.font.clone(),
            color: Color::rgb(0.0, 0.0, 0.0),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // size: Size::new(Val::Percent(100.0), Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(50.0),
                    top: Val::Px(25.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Interface)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ScoreText);

            parent
                .spawn_bundle(TextBundle {
                    text: text.clone(),
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Auto),
                        position_type: PositionType::Relative,
                        position: Rect {
                            left: Val::Px(55.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(TimeLeftText);
        });
}

pub(crate) fn update_hud(
    player_query: Query<&Player>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    mut time_left_query: Query<&mut Text, Without<ScoreText>>,
    time: Res<Time>,
    mut round_timer: ResMut<RoundTimer>,
) {
    let player_1 = player_query.iter().find(|p| p.name == "Player 1");

    for mut text in score_text_query.iter_mut() {
        let str = format!("HEALTH {}", player_1.unwrap().health).to_string();
        text.sections[0].value = str;
    }

    round_timer.0.tick(time.delta());
    let seconds_left = round_timer.0.duration().as_secs() - round_timer.0.elapsed().as_secs();

    for mut text in time_left_query.iter_mut() {
        let str = format!("TIME LEFT {}", seconds_left).to_string();
        text.sections[0].value = str;
    }
}

pub(crate) fn draw_gun(mut commands: Commands, wolfenstein_sprites: Res<GameAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(0.),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Interface)
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Auto),
                        ..Default::default()
                    },
                    image: wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                        .clone()
                        .into(),
                    ..Default::default()
                })
                .insert(Weapon)
                .insert(AnimationTimer(Timer::from_seconds(0.1, true)));
        });
}

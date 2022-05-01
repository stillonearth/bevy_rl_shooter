use bevy::prelude::*;

use bevy_rl::AIGymSettings;

use crate::app_states::*;

#[derive(Component)]
pub(crate) struct Interface;

// PostGame

pub(crate) fn round_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("Roboto-Regular.ttf");

    let text = Text::with_section(
        "ROUND OVER",
        TextStyle {
            font_size: 75.0,
            font: font.clone(),
            color: Color::rgb(0.2, 0.2, 0.2),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(Interface);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color: Color::WHITE.clone().into(),
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .insert(Interface)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(170.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position: Rect {
                            top: Val::Px(50.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::BLACK.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "NEW ROUND",
                            TextStyle {
                                font: font.clone(),
                                font_size: 15.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn_bundle(TextBundle {
                text,
                ..Default::default()
            });
        });
}

pub(crate) fn main_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gym_settings: Res<AIGymSettings>,
) {
    let font = asset_server.load("Roboto-Regular.ttf");

    let text = Text::with_section(
        "ROYAL BATTLE BEVYSTEIN",
        TextStyle {
            font_size: 35.0,
            font: font.clone(),
            color: Color::rgb(0.2, 0.2, 0.2),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
            ..Default::default()
        },
    );

    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(Interface);
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(
                    Val::Px(gym_settings.width as f32),
                    Val::Px(gym_settings.height as f32),
                ),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Interface)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(170.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position: Rect {
                            top: Val::Px(50.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::BLACK.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "NEW ROUND",
                            TextStyle {
                                font: font.clone(),
                                font_size: 15.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent.spawn_bundle(TextBundle {
                text,
                ..Default::default()
            });
        });
}

pub(crate) fn button_system(
    mut app_state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                app_state.set(AppState::InGame).unwrap();
            }
            Interaction::Hovered => {
                *color = Color::GRAY.into();
            }
            Interaction::None => {
                *color = Color::BLACK.into();
            }
        }
    }
}

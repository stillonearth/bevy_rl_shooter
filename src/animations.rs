use bevy::prelude::*;

use crate::{actors::Actor, actors::*, assets::*, hud::*};

#[derive(Component)]
pub(crate) struct AnimationTimer(pub(crate) Timer);

#[derive(Component, Clone)]
pub(crate) struct EnemyAnimation {
    pub frame: u8,
    pub handle: Handle<Mesh>,
    pub animation_type: AnimationType,
}

#[derive(PartialEq, Clone)]
pub(crate) enum AnimationType {
    Standing,
    Walking,
    Shooting,
    Dying,
}

#[derive(Component)]
pub(crate) struct Billboard;

pub(crate) fn animate_gun(
    time: Res<Time>,
    mut wolfenstein_sprites: ResMut<GameAssets>,
    mut query: Query<(&Weapon, &mut AnimationTimer, &mut UiImage)>,
) {
    if wolfenstein_sprites.gun_index == 0 {
        return;
    }

    for (_, mut timer, mut ui_image) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            wolfenstein_sprites.gun_index += 1;
            if wolfenstein_sprites.gun_index >= (wolfenstein_sprites.gun.len() as u8) {
                wolfenstein_sprites.gun_index = 0;
            }

            ui_image.0 = wolfenstein_sprites.gun[wolfenstein_sprites.gun_index as usize]
                .clone()
                .into();
        }
    }
}

pub(crate) fn animate_enemy(
    time: Res<Time>,
    wolfenstein_sprites: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,

    mut q: ParamSet<(
        Query<(&mut AnimationTimer, &Parent, &mut EnemyAnimation), With<Billboard>>,
        Query<&GlobalTransform, With<PlayerPerspective>>,
    )>,
    parent_query: Query<(&Actor, &GlobalTransform)>,
) {
    let q1 = q.p1();
    let player_transform = q1.iter().last().unwrap();
    let player_position = player_transform.translation;
    let player_fwd = player_transform.forward().normalize();

    for (mut timer, parent, mut animation) in q.p0().iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // 2D animations

            let mut animations: Vec<Vec<[f32; 2]>> = Vec::new();
            if animation.animation_type == AnimationType::Dying {
                animations = wolfenstein_sprites.guard_dying_animation.clone();
            }

            if animation.animation_type == AnimationType::Shooting {
                animations = wolfenstein_sprites.guard_shooting_animation.clone();
            }

            if (animation.animation_type == AnimationType::Standing)
                || (animation.animation_type == AnimationType::Walking)
            {
                // 3D animations
                let frameset = match animation.animation_type {
                    AnimationType::Standing => wolfenstein_sprites.guard_standing_animation.clone(),
                    AnimationType::Walking => wolfenstein_sprites.guard_walking_animation.clone(),
                    _ => Vec::new(),
                };

                let parent_transform = parent_query.get(parent.0).unwrap().1;
                let enemy_fwd = parent_transform.forward().normalize();
                let enemy_position = parent_transform.translation;

                // this angle code was a major headache
                // brotip:
                //  * acos of dot product = absolute value of angle btwn vectors
                //  * crossproduct -> 3 vector, sign of a perpendiculat component indicates
                //    whether vectors left / right

                let mut angle = f32::acos(enemy_fwd.dot(player_fwd));
                let sign = -player_fwd.cross((enemy_fwd).normalize()).y.signum();
                angle *= sign;

                let mut view_angle =
                    f32::acos(player_fwd.dot((enemy_position - player_position).normalize()));

                let sign = -player_fwd
                    .cross((enemy_position - player_position).normalize())
                    .y
                    .signum();

                view_angle *= sign;

                angle += view_angle;
                angle *= 180.0 / std::f32::consts::PI;

                angle += 180.0;

                if angle < 0.0 {
                    angle += 360.0;
                }

                let mut index = 0;
                if angle >= 0.0 && angle < 45.0 {
                    index = 0
                } else if angle >= 45.0 && angle < 90.0 {
                    index = 1
                } else if angle >= 90.0 && angle < 135.0 {
                    index = 2
                } else if angle >= 135.0 && angle < 180.0 {
                    index = 3
                } else if angle >= 180.0 && angle < 225.0 {
                    index = 4
                } else if angle >= 225.0 && angle < 270.0 {
                    index = 5
                } else if angle >= 270.0 && angle < 315.0 {
                    index = 6
                } else if angle >= 315.0 && angle < 360.0 {
                    index = 7
                }
                animations = frameset[index].clone();
            }

            if let Some(mesh) = meshes.get_mut(animation.handle.clone()) {
                let uv = animations[animation.frame as usize].clone();

                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            }

            if animation.frame >= (animations.len() as u8 - 1) {
                if animation.animation_type == AnimationType::Shooting {
                    animation.animation_type = AnimationType::Standing;
                }

                if animation.animation_type != AnimationType::Dying {
                    animation.frame = 0;
                }
            } else {
                animation.frame += 1;
            }
        }
    }
}

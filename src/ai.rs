use std::time::Instant;

use bevy::prelude::*;
use big_brain::prelude::*;
use heron::*;

use crate::{animations::*, events::*, player::*};

#[derive(Component, Debug)]
pub(crate) struct BloodThirst {
    pub(crate) enemies_near: u8,
}

pub(crate) fn bloodthirst_system(
    mut thirsts: Query<(&GlobalTransform, &Player, &mut BloodThirst)>,
) {
    let _transforms: Vec<(GlobalTransform, Player)> = thirsts
        .iter()
        .map(|(p, g, _)| (p.clone(), g.clone()))
        .collect();

    for (gt, player, mut thirst) in thirsts.iter_mut() {
        if player.health == 0 {
            thirst.enemies_near = 0;
        } else {
            thirst.enemies_near = _transforms
                .iter()
                .filter(|(g, p)| {
                    if p.health == 0 {
                        return false;
                    }
                    let distance = ((gt.translation.x - g.translation.x).powf(2.0)
                        + (gt.translation.z - g.translation.z).powf(2.0))
                    .sqrt();
                    return (distance < 20.0) && (distance != 0.0);
                })
                .count() as u8;
        }
    }
}

#[derive(Clone, Component, Debug)]
pub(crate) struct Kill {
    pub(crate) last_action: Instant,
}

pub(crate) fn kill_action_system(
    mut bloodthirsts: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &mut BloodThirst,
        &mut Player,
    )>,
    mut actors: Query<(&Actor, &mut ActionState, &mut Kill)>,
    mut enemy_animations: Query<(Entity, &Parent, &mut EnemyAnimation)>,

    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    let players: Vec<(Entity, Transform, Player)> = bloodthirsts
        .iter()
        .map(|(e, _, t, _, p)| (e.clone(), t.clone(), p.clone()))
        .collect();

    for (Actor(actor), mut state, mut kill) in actors.iter_mut() {
        if let Some((_, mut velocity, mut transform, thirst, mut player)) =
            bloodthirsts.iter_mut().find(|e| e.0.id() == actor.id())
        {
            let (_, _, mut animation) = enemy_animations
                .iter_mut()
                .find(|p| p.1.id() == actor.id())
                .unwrap();

            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if thirst.enemies_near == 0 {
                        if player.health == 0 {
                            animation.animation_type = AnimationType::Dying;
                            // animation.frame = 0;
                        } else {
                            *velocity =
                                Velocity::from_linear(transform.forward().normalize() * 2.0);
                            animation.animation_type = AnimationType::Walking;
                        }

                        *state = ActionState::Success;
                    } else {
                        // turn to next target

                        *velocity = Velocity::from_linear(Vec3::ZERO);

                        let duration = kill.last_action.elapsed().as_secs_f32();
                        if duration <= 0.5 {
                            continue;
                        }

                        animation.animation_type = AnimationType::Shooting;
                        animation.frame = 0;

                        kill.last_action = Instant::now();

                        let near_enemy = players.iter().find(|(_, gt, e)| {
                            if e.health == 0 {
                                return false;
                            }
                            let distance = ((gt.translation.x - transform.translation.x).powf(2.0)
                                + (gt.translation.z - transform.translation.z).powf(2.0))
                            .sqrt();

                            return distance <= 20.0 && distance != 0.0;
                        });

                        if near_enemy.is_none() {
                            continue;
                        }

                        let mut view_angle = f32::acos(transform.forward().dot(
                            (near_enemy.unwrap().1.translation - transform.translation).normalize(),
                        ));

                        let sign = transform
                            .forward()
                            .normalize()
                            .cross(
                                (near_enemy.unwrap().1.translation - transform.translation)
                                    .normalize(),
                            )
                            .y
                            .signum();

                        view_angle *= sign;

                        if view_angle.is_nan() {
                            return;
                        }

                        transform.rotate(Quat::from_rotation_y(view_angle));
                        player.rotation += view_angle;

                        event_gun_shot.send(EventGunShot {
                            from: player.name.clone(),
                        })
                    }
                }
                // All Actions should make sure to handle cancellations!
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Component, Debug)]
pub(crate) struct BloodThirsty;

pub(crate) fn bloodthirsty_scorer_system(
    thirsts: Query<&BloodThirst>,
    mut query: Query<(&Actor, &mut Score), With<BloodThirsty>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(thirst) = thirsts.get(*actor) {
            let mut s = (thirst.enemies_near as f32) / 100.;
            if s > 1.0 {
                s = 1.0;
            }
            score.set(s);
        }
    }
}

#![allow(clippy::approx_constant)]
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{actions::*, actors::*, events::*};

#[derive(Resource)]
pub(crate) struct DelayedControlTimer(pub(crate) Timer);

pub(crate) fn control_agents(
    agent_actions: Vec<Option<Actions>>,
    mut agent_movement_query: Query<(&mut Velocity, &mut Transform, &Actor)>,
    mut collision_events: EventReader<CollisionEvent>,
    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    for (i, (mut velocity, transform, actor)) in agent_movement_query.iter_mut().enumerate() {
        *velocity = Velocity { ..default() };

        if actor.health == 0 {
            continue;
        }

        if agent_actions[i].is_some() {
            let agent_actions = agent_actions[i].clone().unwrap();

            if agent_actions.contains(Actions::FORWARD) {
                *velocity = Velocity {
                    linvel: velocity.linvel + 10. * transform.forward().normalize(),
                    ..default()
                }
            }
            if agent_actions.contains(Actions::BACKWARD) {
                *velocity = Velocity {
                    linvel: velocity.linvel + 10. * transform.left().normalize(),
                    ..default()
                }
            }
            if agent_actions.contains(Actions::LEFT) {
                *velocity = Velocity {
                    linvel: velocity.linvel + 10. * -transform.forward().normalize(),
                    ..default()
                }
            }
            if agent_actions.contains(Actions::RIGHT) {
                *velocity = Velocity {
                    linvel: velocity.linvel + 10. * transform.right().normalize(),
                    ..default()
                }
            }
            if agent_actions.contains(Actions::TURN_LEFT) {
                *velocity = Velocity {
                    linvel: velocity.linvel,
                    angvel: Vec3::new(0.2, 0.5 * 3.14, 0.8),
                };
            }
            if agent_actions.contains(Actions::TURN_RIGHT) {
                *velocity = Velocity {
                    linvel: velocity.linvel,
                    angvel: Vec3::new(0.2, -0.5 * 3.14, 0.8),
                };
            }
            if agent_actions.contains(Actions::SHOOT) {
                event_gun_shot.send(EventGunShot {
                    from: actor.name.to_string(),
                });
            }
        }

        collision_events.iter().for_each(|_| {
            // Stop the motion upon collision
            *velocity = Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            };
        });
    }
}

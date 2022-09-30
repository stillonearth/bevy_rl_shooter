use bevy::prelude::*;
use heron::*;

use crate::{actions::*, actors::*, events::*, physics::*};

pub(crate) struct DelayedControlTimer(pub(crate) Timer);

fn is_player(layers: CollisionLayers) -> bool {
    layers.contains_group(Layer::Player) && !layers.contains_group(Layer::World)
}

fn is_world(layers: CollisionLayers) -> bool {
    !layers.contains_group(Layer::Player) && layers.contains_group(Layer::World)
}

pub(crate) fn control_agents(
    agent_actions: Vec<Option<PlayerActionFlags>>,
    mut agent_movement_query: Query<(&mut heron::prelude::Velocity, &mut Transform, &Actor)>,
    mut collision_events: EventReader<CollisionEvent>,
    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    for (i, (mut velocity, transform, actor)) in agent_movement_query.iter_mut().enumerate() {
        *velocity = Velocity::from_linear(Vec3::ZERO);

        if actor.health == 0 {
            continue;
        }

        if agent_actions[i].is_some() {
            let agent_actions = agent_actions[i].unwrap();

            if agent_actions.contains(PlayerActionFlags::FORWARD) {
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.forward().normalize());
            }
            if agent_actions.contains(PlayerActionFlags::BACKWARD) {
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.left().normalize());
            }
            if agent_actions.contains(PlayerActionFlags::LEFT) {
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * -transform.forward().normalize());
            }
            if agent_actions.contains(PlayerActionFlags::RIGHT) {
                *velocity =
                    velocity.with_linear(velocity.linear + 10. * transform.right().normalize());
            }
            if agent_actions.contains(PlayerActionFlags::TURN_LEFT) {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
            }
            if agent_actions.contains(PlayerActionFlags::TURN_RIGHT) {
                *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
            }
            if agent_actions.contains(PlayerActionFlags::SHOOT) {
                event_gun_shot.send(EventGunShot {
                    from: actor.name.to_string(),
                });
            }
        }

        collision_events
            .iter()
            .filter_map(|event| {
                let (entity_1, entity_2) = event.rigid_body_entities();
                let (layers_1, layers_2) = event.collision_layers();

                if is_player(layers_1) && is_world(layers_2) {
                    Some(entity_2)
                } else if is_player(layers_2) && is_world(layers_1) {
                    Some(entity_1)
                } else {
                    // This event is not the collision between an enemy and the player.
                    // We can ignore it.
                    None
                }
            })
            .for_each(|_| {
                // Stop the motion upon collision
                *velocity = Velocity::from_linear(Vec3::X * 0.0);
            });
    }
}

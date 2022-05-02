use bevy::prelude::*;
use heron::*;

use crate::{actions::*, app_states::*, events::*, physics::*, player::*};

pub(crate) struct DelayedControlTimer(pub(crate) Timer);

pub(crate) fn turnbased_control_player_keyboard(
    keys: Res<Input<KeyCode>>,
    player_movement_q: Query<
        (&mut heron::prelude::Velocity, &mut Transform),
        With<PlayerPerspective>,
    >,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    mut physics_time: ResMut<PhysicsTime>,
    mut app_state: ResMut<State<AppState>>,
) {
    let mut player_action = PlayerActionFlags::IDLE;

    let mut got_pressed = false;

    for key in keys.get_pressed() {
        if *key == KeyCode::W {
            player_action |= PlayerActionFlags::FORWARD;
            got_pressed = true;
        }
        if *key == KeyCode::A {
            player_action |= PlayerActionFlags::BACKWARD;
            got_pressed = true;
        }
        if *key == KeyCode::S {
            player_action |= PlayerActionFlags::LEFT;
            got_pressed = true;
        }
        if *key == KeyCode::D {
            player_action |= PlayerActionFlags::RIGHT;
            got_pressed = true;
        }
        if *key == KeyCode::Q {
            player_action |= PlayerActionFlags::TURN_LEFT;
            got_pressed = true;
        }
        if *key == KeyCode::E {
            player_action |= PlayerActionFlags::TURN_RIGHT;
            got_pressed = true;
        }
        if keys.just_pressed(KeyCode::Space) {
            player_action |= PlayerActionFlags::SHOOT;
            got_pressed = true;
        }
    }

    if got_pressed {
        physics_time.resume();
        control_player(
            player_action,
            player_movement_q,
            collision_events,
            event_gun_shot,
        );

        app_state.pop().unwrap();
    }
}

pub(crate) fn control_player(
    player_action: PlayerActionFlags,
    mut player_movement_q: Query<
        (&mut heron::prelude::Velocity, &mut Transform),
        With<PlayerPerspective>,
    >,
    mut collision_events: EventReader<CollisionEvent>,
    mut event_gun_shot: EventWriter<EventGunShot>,
) {
    fn is_player(layers: CollisionLayers) -> bool {
        layers.contains_group(Layer::Player) && !layers.contains_group(Layer::World)
    }

    fn is_world(layers: CollisionLayers) -> bool {
        !layers.contains_group(Layer::Player) && layers.contains_group(Layer::World)
    }

    for (mut velocity, transform) in player_movement_q.iter_mut() {
        *velocity = Velocity::from_linear(Vec3::ZERO);
        if player_action.contains(PlayerActionFlags::FORWARD) {
            *velocity =
                velocity.with_linear(velocity.linear + 10. * transform.forward().normalize());
        }
        if player_action.contains(PlayerActionFlags::BACKWARD) {
            *velocity = velocity.with_linear(velocity.linear + 10. * transform.left().normalize());
        }
        if player_action.contains(PlayerActionFlags::LEFT) {
            *velocity =
                velocity.with_linear(velocity.linear + 10. * -transform.forward().normalize());
        }
        if player_action.contains(PlayerActionFlags::RIGHT) {
            *velocity = velocity.with_linear(velocity.linear + 10. * transform.right().normalize());
        }
        if player_action.contains(PlayerActionFlags::TURN_LEFT) {
            *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, 0.5 * 3.14));
        }
        if player_action.contains(PlayerActionFlags::TURN_RIGHT) {
            *velocity = velocity.with_angular(AxisAngle::new(Vec3::Y, -0.5 * 3.14));
        }
        if player_action.contains(PlayerActionFlags::SHOOT) {
            event_gun_shot.send(EventGunShot {
                from: "Player 1".to_string(),
            });
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
                    // This event is not the collision between an enemy and the player. We can ignore it.
                    None
                }
            })
            .for_each(|_| {
                *velocity = Velocity::from_linear(Vec3::X * 0.0);
            });
    }
}

pub(crate) fn control_player_keyboard(
    keys: Res<Input<KeyCode>>,
    player_movement_q: Query<
        (&mut heron::prelude::Velocity, &mut Transform),
        With<PlayerPerspective>,
    >,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
) {
    let mut player_action = PlayerActionFlags::IDLE;

    for key in keys.get_pressed() {
        if *key == KeyCode::W {
            player_action |= PlayerActionFlags::FORWARD;
        }
        if *key == KeyCode::A {
            player_action |= PlayerActionFlags::BACKWARD;
        }
        if *key == KeyCode::S {
            player_action |= PlayerActionFlags::LEFT;
        }
        if *key == KeyCode::D {
            player_action |= PlayerActionFlags::RIGHT;
        }
        if *key == KeyCode::Q {
            player_action |= PlayerActionFlags::TURN_LEFT;
        }
        if *key == KeyCode::E {
            player_action |= PlayerActionFlags::TURN_RIGHT;
        }
        if keys.just_pressed(KeyCode::Space) {
            player_action |= PlayerActionFlags::SHOOT;
        }
    }

    control_player(
        player_action,
        player_movement_q,
        collision_events,
        event_gun_shot,
    );
}

use crossbeam_channel::*;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rl::state::AIGymState;
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

pub(crate) fn turnbased_control_system_switch(
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<DelayedControlTimer>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut physics_time: ResMut<PhysicsTime>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_state.push(AppState::Control);
        physics_time.pause();

        let ai_gym_state = ai_gym_state.lock().unwrap();

        if ai_gym_state.__result_channel_rx.is_empty() {
            ai_gym_state.__result_channel_tx.send(true).unwrap();
        }
    }
}

pub(crate) fn turnbased_text_control_system(
    player_movement_q: Query<
        (&mut heron::prelude::Velocity, &mut Transform),
        With<PlayerPerspective>,
    >,
    collision_events: EventReader<CollisionEvent>,
    event_gun_shot: EventWriter<EventGunShot>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags>>>>,
    mut app_state: ResMut<State<AppState>>,
    mut physics_time: ResMut<PhysicsTime>,
    player_query: Query<&Player>,
) {
    let step_rx: Receiver<String>;
    let result_tx: Sender<bool>;
    {
        let ai_gym_state = ai_gym_state.lock().unwrap();
        step_rx = ai_gym_state.__step_channel_rx.clone();
        result_tx = ai_gym_state.__result_channel_tx.clone();
    }

    if step_rx.is_empty() {
        return;
    }

    let unparsed_action = step_rx.recv().unwrap();

    if unparsed_action == "" {
        if result_tx.is_empty() {
            result_tx.send(false).unwrap();
        }
        return;
    }

    let action = match unparsed_action.as_str() {
        "FORWARD" => Some(PlayerActionFlags::FORWARD),
        "BACKWARD" => Some(PlayerActionFlags::BACKWARD),
        "LEFT" => Some(PlayerActionFlags::LEFT),
        "RIGHT" => Some(PlayerActionFlags::RIGHT),
        "TURN_LEFT" => Some(PlayerActionFlags::TURN_LEFT),
        "TURN_RIGHT" => Some(PlayerActionFlags::TURN_RIGHT),
        "SHOOT" => Some(PlayerActionFlags::SHOOT),
        _ => None,
    };

    if action.is_none() {
        if result_tx.is_empty() {
            result_tx.send(false).unwrap();
        }
        return;
    }

    let player = player_query.iter().find(|e| e.name == "Player 1").unwrap();
    {
        let mut ai_gym_state = ai_gym_state.lock().unwrap();
        ai_gym_state.rewards.push(player.score as f32);
    }

    physics_time.resume();

    control_player(
        action.unwrap(),
        player_movement_q,
        collision_events,
        event_gun_shot,
    );

    app_state.pop().unwrap();
}

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_mod_raycast::RayCastSource;
use bevy_rl::state::AIGymState;
use heron::*;

use crate::{actions::*, actors::Actor, game::*, gym::EnvironmentState, level::*};

#[derive(Debug)]
pub(crate) struct EventGunShot {
    pub(crate) from: String,
}

#[derive(Debug)]
pub(crate) struct EventDamage {
    pub(crate) from: String,
    pub(crate) to: String,
}

#[derive(Debug)]
pub(crate) struct EventNewRound;

// ------
// Events
// ------

pub(crate) fn event_gun_shot(
    mut commands: Commands,
    shooting_query: Query<(&Parent, &RayCastSource<RaycastMarker>)>,
    actor_query: Query<(Entity, &Children, &Actor)>,
    wall_query: Query<(Entity, &Wall)>,

    mut gunshot_event: EventReader<EventGunShot>,
    mut event_damage: EventWriter<EventDamage>,
) {
    for gunshot_event in gunshot_event.iter() {
        let result = shooting_query.iter().find(|(p, _)| {
            !actor_query
                .iter()
                .find(|(e, _, _p)| e.id() == p.id() && _p.name == gunshot_event.from)
                .is_none()
        });

        if result.is_none() {
            return;
        }

        let (_, raycast_source) = result.unwrap();
        let r = raycast_source.intersect_top();
        if r.is_none() {
            continue;
        }

        let hit_entity = r.unwrap().0;

        let mut player_hit = false;
        for (_, children, enemy) in actor_query.iter() {
            let other_entity = children.iter().find(|c| c.id() == hit_entity.id());
            if other_entity.is_none() {
                continue;
            }

            event_damage.send(EventDamage {
                from: gunshot_event.from.clone(),
                to: enemy.name.clone(),
            });

            player_hit = true;
            continue;
        }

        // despawn a wall
        if !player_hit {
            let wall_entity = wall_query.iter().find(|(w, _)| w.id() == hit_entity.id());
            if wall_entity.is_some() {
                commands.entity(hit_entity).despawn_recursive();
            }
        }
    }
}

pub(crate) fn event_damage(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Children, &mut Actor, &mut Velocity)>,
    mut event_damage: EventReader<EventDamage>,
    ai_gym_state: ResMut<Arc<Mutex<AIGymState<PlayerActionFlags, EnvironmentState>>>>,
) {
    for damage_event in event_damage.iter() {
        if damage_event.from == damage_event.to {
            continue;
        }

        let mut ai_gym_state = ai_gym_state.lock().unwrap();

        if let Some((i, (entity, _, mut actor, mut _velocity))) = player_query
            .iter_mut()
            .filter(|(_, _, actor, _)| actor.health > 0)
            .enumerate()
            .find(|(_, p)| p.2.name == damage_event.to)
        {
            actor.health -= 100;

            commands
                .entity(entity)
                .insert(Velocity::from_linear(Vec3::ZERO))
                .insert(Visibility { is_visible: false });

            ai_gym_state.set_reward(i, 10.0);
        }
    }
}

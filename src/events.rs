use bevy::prelude::*;
use bevy_mod_raycast::{RayCastMesh, RayCastSource};
use heron::*;

use crate::{animations::*, assets::*, game::*, hud::*, player::*};

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

    mut gun_sprite_query: Query<(&Weapon, &mut UiImage)>,
    shooting_query: Query<(&Parent, &RayCastSource<RaycastMarker>)>,
    player_query: Query<(Entity, &Children, &Player)>,
    wall_query: Query<(Entity, &Wall)>,

    mut gunshot_event: EventReader<EventGunShot>,
    mut event_damage: EventWriter<EventDamage>,

    mut game_sprites: ResMut<GameAssets>,
) {
    for gunshot_event in gunshot_event.iter() {
        if gunshot_event.from == "Player 1".to_string() {
            for (_, mut ui_image) in gun_sprite_query.iter_mut() {
                game_sprites.gun_index = 1;
                ui_image.0 = game_sprites.gun[game_sprites.gun_index as usize]
                    .clone()
                    .into();
            }
        }

        let result = shooting_query.iter().find(|(p, _)| {
            !player_query
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
        for (_, children, enemy) in player_query.iter() {
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
    mut player_query: Query<(Entity, &Children, &mut Player, &mut Velocity)>,
    mut billboard_query: Query<(Entity, &mut EnemyAnimation, &Billboard)>,
    mut event_damage: EventReader<EventDamage>,
) {
    for damage_event in event_damage.iter() {
        if damage_event.from == damage_event.to {
            continue;
        }

        if let Some((entity, children, mut player, mut velocity)) = player_query
            .iter_mut()
            .find(|p| p.2.name == damage_event.to)
        {
            if player.health == 0 {
                continue;
            }

            player.health -= 100;

            if player.health == 0 {
                for c in children.iter() {
                    if let Some((billboard_entity, mut animation, _)) =
                        billboard_query.iter_mut().find(|c_| {
                            return c_.0.id() == c.id();
                        })
                    {
                        animation.frame = 1;
                        animation.animation_type = AnimationType::Dying;

                        velocity.clone_from(&Velocity::from_linear(Vec3::ZERO));

                        commands
                            .entity(billboard_entity)
                            .insert(EnemyAnimation {
                                frame: 0,
                                handle: animation.handle.clone(),
                                animation_type: AnimationType::Dying,
                            })
                            .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
                            .remove::<RayCastMesh<RaycastMarker>>();
                    }
                }

                commands
                    .entity(entity)
                    .insert(Velocity::from_linear(Vec3::ZERO));
            }

            let (_, _, mut hit_player, _) = player_query
                .iter_mut()
                .find(|(_, _, p, _)| p.name == damage_event.from)
                .unwrap();

            hit_player.score += 10;
        }
    }
}

use bevy::prelude::*;

#[derive(Component, Clone)]
pub(crate) struct Player {
    pub position: (f32, f32),
    pub rotation: f32,
    pub name: String,
    pub health: u16,
    pub score: u16,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerPerspective;

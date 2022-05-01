use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::map;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GameMap {
    pub empty_space: Vec<(usize, usize)>,
    pub walls: Vec<(usize, usize)>,
}

impl FromWorld for GameMap {
    fn from_world(_: &mut World) -> Self {
        let deserialized: GameMap = serde_json::from_str(&map::JSON).unwrap();
        return deserialized;
    }
}

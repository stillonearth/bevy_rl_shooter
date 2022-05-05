use std::ops::Range;

use bevy::prelude::*;

#[derive(Clone)]
pub struct GameAssets {
    pub gun: Vec<Handle<Image>>,
    pub gun_index: u8,

    pub guard_billboard_material: Handle<StandardMaterial>,
    pub guard_walking_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_standing_animation: Vec<Vec<Vec<[f32; 2]>>>,
    pub guard_dying_animation: Vec<Vec<[f32; 2]>>,
    pub guard_shooting_animation: Vec<Vec<[f32; 2]>>,

    pub font: Handle<Font>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        // gun
        let mut gun: Vec<Handle<Image>> = Vec::new();
        gun.push(asset_server.load("gun/gun_0.png"));
        gun.push(asset_server.load("gun/gun_1.png"));
        gun.push(asset_server.load("gun/gun_2.png"));

        // soldier
        let guard_billboard_material = materials.add(StandardMaterial {
            base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            base_color_texture: Some(asset_server.load("guard-sheet.png")),
            unlit: true,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        fn gather_angle_animation_uvs(
            row: f32,
            frames: Range<u8>,
            offset: f32,
        ) -> Vec<Vec<[f32; 2]>> {
            let mut frame_set: Vec<Vec<[f32; 2]>> = Vec::new();

            for column in frames {
                let mut uvs1: Vec<[f32; 2]> = Vec::<[f32; 2]>::new();
                let column = column as f32;

                uvs1.push([(row - 1.0) / 8.0 + offset, column / 7.0]);
                uvs1.push([(row - 1.0) / 8.0 + offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column) / 7.0]);

                frame_set.push(uvs1);
            }

            return frame_set;
        }

        fn gather_full_animation_uvs(frames: Range<u8>) -> Vec<Vec<Vec<[f32; 2]>>> {
            let mut animations: Vec<Vec<Vec<[f32; 2]>>> = Vec::new();

            for i in 1..9 {
                let angle_animations = gather_angle_animation_uvs(i as f32, frames.clone(), 0.042);
                animations.push(angle_animations);
            }

            return animations;
        }

        fn gather_row_animation_uvs(
            column: f32,
            frames: Range<u8>,
            offset: f32,
        ) -> Vec<Vec<[f32; 2]>> {
            let mut frame_set: Vec<Vec<[f32; 2]>> = Vec::new();
            // let offset = 0.0;

            for row in frames {
                let mut uvs1: Vec<[f32; 2]> = Vec::<[f32; 2]>::new();
                let row = row as f32;

                uvs1.push([(row - 1.0) / 8.0 + offset, column / 7.0]);
                uvs1.push([(row - 1.0) / 8.0 + offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column - 1.0) / 7.0]);
                uvs1.push([row / 8.0 - offset, (column) / 7.0]);

                frame_set.push(uvs1);
            }

            return frame_set;
        }

        return Self {
            gun,
            gun_index: 0,
            font: asset_server.load("Roboto-Regular.ttf"),
            guard_billboard_material,
            guard_walking_animation: gather_full_animation_uvs(Range { start: 2, end: 6 }),
            guard_standing_animation: gather_full_animation_uvs(Range { start: 1, end: 2 }),
            guard_dying_animation: gather_row_animation_uvs(6.0, Range { start: 1, end: 6 }, 0.0),
            guard_shooting_animation: gather_row_animation_uvs(
                7.0,
                Range { start: 1, end: 4 },
                0.042,
            ),
        };
    }
}

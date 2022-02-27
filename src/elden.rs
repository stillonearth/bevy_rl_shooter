use crate::cache;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub type ColorMap = [(u8, u8, u8); 256];

pub fn build_color_map() -> ColorMap {
    // [SDL_Color(r*255//63, g*255//63, b*255//63, 0) for r, g, b in COLORS]
    let palette = [
        (0, 0, 0),
        (0, 0, 42),
        (0, 42, 0),
        (0, 42, 42),
        (42, 0, 0),
        (42, 0, 42),
        (42, 21, 0),
        (42, 42, 42),
        (21, 21, 21),
        (21, 21, 63),
        (21, 63, 21),
        (21, 63, 63),
        (63, 21, 21),
        (63, 21, 63),
        (63, 63, 21),
        (63, 63, 63),
        (59, 59, 59),
        (55, 55, 55),
        (52, 52, 52),
        (48, 48, 48),
        (45, 45, 45),
        (42, 42, 42),
        (38, 38, 38),
        (35, 35, 35),
        (31, 31, 31),
        (28, 28, 28),
        (25, 25, 25),
        (21, 21, 21),
        (18, 18, 18),
        (14, 14, 14),
        (11, 11, 11),
        (8, 8, 8),
        (63, 0, 0),
        (59, 0, 0),
        (56, 0, 0),
        (53, 0, 0),
        (50, 0, 0),
        (47, 0, 0),
        (44, 0, 0),
        (41, 0, 0),
        (38, 0, 0),
        (34, 0, 0),
        (31, 0, 0),
        (28, 0, 0),
        (25, 0, 0),
        (22, 0, 0),
        (19, 0, 0),
        (16, 0, 0),
        (63, 54, 54),
        (63, 46, 46),
        (63, 39, 39),
        (63, 31, 31),
        (63, 23, 23),
        (63, 16, 16),
        (63, 8, 8),
        (63, 0, 0),
        (63, 42, 23),
        (63, 38, 16),
        (63, 34, 8),
        (63, 30, 0),
        (57, 27, 0),
        (51, 24, 0),
        (45, 21, 0),
        (39, 19, 0),
        (63, 63, 54),
        (63, 63, 46),
        (63, 63, 39),
        (63, 63, 31),
        (63, 62, 23),
        (63, 61, 16),
        (63, 61, 8),
        (63, 61, 0),
        (57, 54, 0),
        (51, 49, 0),
        (45, 43, 0),
        (39, 39, 0),
        (33, 33, 0),
        (28, 27, 0),
        (22, 21, 0),
        (16, 16, 0),
        (52, 63, 23),
        (49, 63, 16),
        (45, 63, 8),
        (40, 63, 0),
        (36, 57, 0),
        (32, 51, 0),
        (29, 45, 0),
        (24, 39, 0),
        (54, 63, 54),
        (47, 63, 46),
        (39, 63, 39),
        (32, 63, 31),
        (24, 63, 23),
        (16, 63, 16),
        (8, 63, 8),
        (0, 63, 0),
        (0, 63, 0),
        (0, 59, 0),
        (0, 56, 0),
        (0, 53, 0),
        (1, 50, 0),
        (1, 47, 0),
        (1, 44, 0),
        (1, 41, 0),
        (1, 38, 0),
        (1, 34, 0),
        (1, 31, 0),
        (1, 28, 0),
        (1, 25, 0),
        (1, 22, 0),
        (1, 19, 0),
        (1, 16, 0),
        (54, 63, 63),
        (46, 63, 63),
        (39, 63, 63),
        (31, 63, 62),
        (23, 63, 63),
        (16, 63, 63),
        (8, 63, 63),
        (0, 63, 63),
        (0, 57, 57),
        (0, 51, 51),
        (0, 45, 45),
        (0, 39, 39),
        (0, 33, 33),
        (0, 28, 28),
        (0, 22, 22),
        (0, 16, 16),
        (23, 47, 63),
        (16, 44, 63),
        (8, 42, 63),
        (0, 39, 63),
        (0, 35, 57),
        (0, 31, 51),
        (0, 27, 45),
        (0, 23, 39),
        (54, 54, 63),
        (46, 47, 63),
        (39, 39, 63),
        (31, 32, 63),
        (23, 24, 63),
        (16, 16, 63),
        (8, 9, 63),
        (0, 1, 63),
        (0, 0, 63),
        (0, 0, 59),
        (0, 0, 56),
        (0, 0, 53),
        (0, 0, 50),
        (0, 0, 47),
        (0, 0, 44),
        (0, 0, 41),
        (0, 0, 38),
        (0, 0, 34),
        (0, 0, 31),
        (0, 0, 28),
        (0, 0, 25),
        (0, 0, 22),
        (0, 0, 19),
        (0, 0, 16),
        (10, 10, 10),
        (63, 56, 13),
        (63, 53, 9),
        (63, 51, 6),
        (63, 48, 2),
        (63, 45, 0),
        (45, 8, 63),
        (42, 0, 63),
        (38, 0, 57),
        (32, 0, 51),
        (29, 0, 45),
        (24, 0, 39),
        (20, 0, 33),
        (17, 0, 28),
        (13, 0, 22),
        (10, 0, 16),
        (63, 54, 63),
        (63, 46, 63),
        (63, 39, 63),
        (63, 31, 63),
        (63, 23, 63),
        (63, 16, 63),
        (63, 8, 63),
        (63, 0, 63),
        (56, 0, 57),
        (50, 0, 51),
        (45, 0, 45),
        (39, 0, 39),
        (33, 0, 33),
        (27, 0, 28),
        (22, 0, 22),
        (16, 0, 16),
        (63, 58, 55),
        (63, 56, 52),
        (63, 54, 49),
        (63, 53, 47),
        (63, 51, 44),
        (63, 49, 41),
        (63, 47, 39),
        (63, 46, 36),
        (63, 44, 32),
        (63, 41, 28),
        (63, 39, 24),
        (60, 37, 23),
        (58, 35, 22),
        (55, 34, 21),
        (52, 32, 20),
        (50, 31, 19),
        (47, 30, 18),
        (45, 28, 17),
        (42, 26, 16),
        (40, 25, 15),
        (39, 24, 14),
        (36, 23, 13),
        (34, 22, 12),
        (32, 20, 11),
        (29, 19, 10),
        (27, 18, 9),
        (23, 16, 8),
        (21, 15, 7),
        (18, 14, 6),
        (16, 12, 6),
        (14, 11, 5),
        (10, 8, 3),
        (24, 0, 25),
        (0, 25, 25),
        (0, 24, 24),
        (0, 0, 7),
        (0, 0, 11),
        (12, 9, 4),
        (18, 0, 18),
        (20, 0, 20),
        (0, 0, 13),
        (7, 7, 7),
        (19, 19, 19),
        (23, 23, 23),
        (16, 16, 16),
        (12, 12, 12),
        (13, 13, 13),
        (54, 61, 61),
        (46, 58, 58),
        (39, 55, 55),
        (29, 50, 50),
        (18, 48, 48),
        (8, 45, 45),
        (8, 44, 44),
        (0, 41, 41),
        (0, 38, 38),
        (0, 35, 35),
        (0, 33, 33),
        (0, 31, 31),
        (0, 30, 30),
        (0, 29, 29),
        (0, 28, 28),
        (0, 27, 27),
        (38, 0, 34),
    ];
    palette.map(|(r, g, b)| {
        (
            (r * 255 / 63) as u8,
            (g * 255 / 63) as u8,
            (b * 255 / 63) as u8,
        )
    })
}

pub fn get_image(image_id: usize) -> Image {
    let cache = cache::startup();
    let face_pic = cache.get_pic(image_id);
    let color_map = build_color_map();

    let mut pixels: Vec<u8> = Vec::new();

    for y in 0..face_pic.height {
        for x in 0..face_pic.width {
            let source_index = (y * (face_pic.width >> 2) + (x >> 2))
                + (x & 3) * (face_pic.width >> 2) * face_pic.height;
            let color = face_pic.data[source_index as usize];
            let color = color_map[color as usize];
            pixels.push(color.0);
            pixels.push(color.1);
            pixels.push(color.2);
            pixels.push(255);
        }
    }

    return Image::new_fill(
        Extent3d {
            width: face_pic.width,
            height: face_pic.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &pixels,
        TextureFormat::Rgba8UnormSrgb,
    );
}

// pub fn get_gun() -> Image {
//     let cache = cache::startup();
//     let (weapon_shape, weapon_data) = cache.get_sprite(209);

//     // weapon_shape.

//     for y in 0..weapon_shape {
//         for x in 0..face_pic.width {
//             let source_index = (y * (face_pic.width >> 2) + (x >> 2))
//                 + (x & 3) * (face_pic.width >> 2) * face_pic.height;
//             let color = face_pic.data[source_index as usize];
//             let color = color_map[color as usize];
//             pixels.push(color.0);
//             pixels.push(color.1);
//             pixels.push(color.2);
//             pixels.push(255);
//         }
//     }

//     return Image::new_fill(
//         Extent3d {
//             width: face_pic.width,
//             height: face_pic.height,
//             depth_or_array_layers: 1,
//         },
//         TextureDimension::D2,
//         &pixels,
//         TextureFormat::Rgba8UnormSrgb,
//     );
// }

// fn simple_scale_shape(
//     view_width: u32,
//     view_height: u32,
//     color_map: ColorMap,
//     vbuf: &mut [u8],
//     pitch: usize,
//     left_pix: u16,
//     right_pix: u16,
//     dataofs: &[u16],
//     shape_bytes: &[u8],
// ) {
//     let sprite_scale_factor = 2;
//     let xcenter = view_width / 2;
//     let height = view_height + 1;

//     let scale = height >> 1;
//     let pixheight = scale * sprite_scale_factor;
//     let actx = xcenter - scale;
//     let upperedge = view_height / 2 - scale;
//     // cmdptr=(word *) shape->dataofs;
//     // cmdptr = iter(shape.dataofs)
//     let mut cmdptr = dataofs.iter();

//     let mut i = left_pix;
//     let mut pixcnt = i as u32 * pixheight;
//     let mut rpix = (pixcnt >> 6) + actx;

//     while i <= right_pix {
//         let mut lpix = rpix;
//         if lpix >= view_width {
//             break;
//         }

//         pixcnt += pixheight;
//         rpix = (pixcnt >> 6) + actx;

//         if lpix != rpix && rpix > 0 {
//             if lpix < 0 {
//                 lpix = 0;
//             }
//             if rpix > view_width {
//                 rpix = view_width;
//                 i = right_pix + 1;
//             }
//             let read_word = |line: &mut Iter<u8>| {
//                 u16::from_le_bytes([*line.next().unwrap_or(&0), *line.next().unwrap_or(&0)])
//             };
//             let read_word_signed = |line: &mut Iter<u8>| {
//                 i16::from_le_bytes([*line.next().unwrap_or(&0), *line.next().unwrap_or(&0)])
//             };

//             let cline = &shape_bytes[*cmdptr.next().unwrap() as usize..];
//             while lpix < rpix {
//                 let mut line = cline.iter();
//                 let mut endy = read_word(&mut line);
//                 while endy > 0 {
//                     endy >>= 1;
//                     let newstart = read_word_signed(&mut line);
//                     let starty = read_word(&mut line) >> 1;
//                     let mut j = starty;
//                     let mut ycnt = j as u32 * pixheight;
//                     let mut screndy: i32 = (ycnt >> 6) as i32 + upperedge as i32;

//                     let mut vmem_index: usize = if screndy < 0 {
//                         lpix as usize * 3
//                     } else {
//                         screndy as usize * pitch + lpix as usize * 3
//                     };

//                     while j < endy {
//                         let mut scrstarty = screndy;
//                         ycnt += pixheight;
//                         screndy = (ycnt >> 6) as i32 + upperedge as i32;
//                         if scrstarty != screndy && screndy > 0 {
//                             let index = newstart + j as i16;
//                             let col = if index >= 0 {
//                                 shape_bytes[index as usize]
//                             } else {
//                                 0
//                             };
//                             if scrstarty < 0 {
//                                 scrstarty = 0;
//                             }
//                             if screndy > view_height as i32 {
//                                 screndy = view_height as i32;
//                                 j = endy;
//                             }

//                             while scrstarty < screndy {
//                                 // FIXME can put pixel be used here instead?
//                                 let (r, g, b) = color_map[col as usize];
//                                 vbuf[vmem_index as usize] = r;
//                                 vbuf[vmem_index as usize + 1] = g;
//                                 vbuf[vmem_index as usize + 2] = b;
//                                 vmem_index += pitch;
//                                 scrstarty += 1;
//                             }
//                         }
//                         j += 1;
//                     }
//                     endy = read_word(&mut line);
//                 }
//                 lpix += 1;
//             }
//         }
//         i += 1;
//     }
// }

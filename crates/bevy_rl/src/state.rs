use image;

use bevy::{prelude::*, render::camera::RenderTarget};

#[derive(Clone, Default)]
pub struct AIGymState<A: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe> {
    // These parts are made of hack trick internals.
    pub __render_target: Option<RenderTarget>, // render target for camera -- window on in our case texture
    pub __render_image_handle: Option<Handle<Image>>, // handle to image we use in bevy UI building.
    // actual texture is GPU ram and we can't access it easily
    pub __is_environment_paused: bool, // once set true we loop and wait until simulation epoch is finished
    pub __action_unparsed_string: String, // we receive action as post parameter and parse it in bevy system
    // Communication via mutex works but semantics are not straightforward.
    // We keep it hacky or else it could become java boilerplate.
    pub __request_for_reset: bool,

    // State
    pub screen: Option<image::RgbaImage>,
    pub rewards: Vec<f32>,
    pub action: Option<A>,
    pub is_terminated: bool,
}

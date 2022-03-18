use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use bevy::{
    core_pipeline::{
        draw_3d_graph, node, AlphaMask3d, Opaque3d, RenderTargetClearColors, Transparent3d,
    },
    prelude::*,
    render::{
        camera::{ActiveCamera, CameraTypePlugin, RenderTarget},
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotValue},
        render_phase::RenderPhase,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderContext,
        view::RenderLayers,
        RenderApp, RenderStage,
    },
};

use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::RenderDevice;
use bevy::render::renderer::RenderQueue;

use bytemuck;

use wgpu::ImageCopyBuffer;
use wgpu::ImageDataLayout;

use futures::executor;
use image;

pub struct AIGymSettings {
    pub width: u32,
    pub height: u32,
}

#[derive(Component, Default)]
pub struct FirstPassCamera;

pub struct AIGymPlugin;

pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

impl Plugin for AIGymPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraTypePlugin::<FirstPassCamera>::default());
        app.add_startup_system(setup.label("setup_rendering"));

        let gym_assets = Arc::new(Mutex::new(AIGymAssets {
            render_layer: None,
            render_target: None,
            rendered_image: None,
            render_image_handle: None,
        }));

        app.insert_resource(gym_assets.clone());

        // Render app
        let render_app = app.sub_app_mut(RenderApp);
        let driver = FirstPassCameraDriver::new(&mut render_app.world);
        // This will add 3D render phases for the new camera.
        render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);
        render_app.add_system_to_stage(RenderStage::Render, save_image);
        render_app.insert_resource(gym_assets.clone());

        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        // Add a node for the first pass.
        graph.add_node(FIRST_PASS_DRIVER, driver);

        // The first pass's dependencies include those of the main pass.
        graph
            .add_node_edge(node::MAIN_PASS_DEPENDENCIES, FIRST_PASS_DRIVER)
            .unwrap();

        // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
        graph
            .add_node_edge(node::CLEAR_PASS_DRIVER, FIRST_PASS_DRIVER)
            .unwrap();
    }
}

// Add 3D render phases for FIRST_PASS_CAMERA.
fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<FirstPassCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
struct FirstPassCameraDriver {
    query: QueryState<Entity, With<FirstPassCamera>>,
}

impl FirstPassCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

impl Node for FirstPassCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_3d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

pub fn texture_image_layout(desc: &TextureDescriptor<'_>) -> ImageDataLayout {
    let size = desc.size;

    let layout = ImageDataLayout {
        bytes_per_row: if size.height > 1 {
            NonZeroU32::new(size.width * (desc.format.describe().block_size as u32))
        } else {
            None
        },
        rows_per_image: if size.depth_or_array_layers > 1 {
            NonZeroU32::new(size.height)
        } else {
            None
        },
        ..Default::default()
    };

    return layout;
}

fn save_image(
    gpu_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    ai_gym_assets: Res<Arc<Mutex<AIGymAssets>>>,
) {
    let mut ai_gym_assets = ai_gym_assets.lock().unwrap();

    let gpu_image = gpu_images
        .get(&ai_gym_assets.render_image_handle.clone().unwrap())
        .unwrap();

    let device = render_device.wgpu_device();

    let destination = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (gpu_image.size.width * gpu_image.size.height * 4.0) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let texture = gpu_image.texture.clone();

    let mut encoder =
        render_device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        ImageCopyBuffer {
            buffer: &destination,
            layout: texture_image_layout(&TextureDescriptor {
                label: Some("render_image"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT, // | TextureUsages::STORAGE_BINDING,
            }),
        },
        Extent3d {
            width: gpu_image.size.width as u32,
            height: gpu_image.size.height as u32,
            ..default()
        },
    );

    render_queue.submit([encoder.finish()]);

    let buffer_slice = destination.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);

    let result = executor::block_on(buffer_future);

    let err = result.err();
    if err.is_some() {
        panic!("{}", err.unwrap().to_string());
    }

    // if let Ok(()) = executor::block_on(buffer_future).err() {
    // Gets contents of buffer
    let data = buffer_slice.get_mapped_range();
    // Since contents are got in bytes, this converts these bytes back to u32
    let result: Vec<u8> = bytemuck::cast_slice(&data).to_vec();

    // With the current interface, we have to make sure all mapped views are
    // dropped before we unmap the buffer.
    drop(data);
    destination.unmap(); // Unmaps buffer from memory
                         // If you are familiar with C++ these 2 lines can be thought of similarly to:
                         //   delete myPointer;
                         //   myPointer = NULL;
                         // It effectively frees the memory

    let img: image::RgbaImage = image::ImageBuffer::from_raw(
        gpu_image.size.width as u32,
        gpu_image.size.height as u32,
        result,
    )
    .unwrap();
    ai_gym_assets.rendered_image = Some(img.clone());

    img.save("screen.png");
}

#[derive(Clone)]
pub struct AIGymAssets {
    pub rendered_image: Option<image::RgbaImage>,
    pub render_layer: Option<RenderLayers>,
    pub render_target: Option<RenderTarget>,
    pub render_image_handle: Option<Handle<Image>>,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ai_gym_assets: ResMut<Arc<Mutex<AIGymAssets>>>,
    ai_gym_settings: Res<AIGymSettings>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut windows: ResMut<Windows>,
) {
    let size = Extent3d {
        width: ai_gym_settings.width,
        height: ai_gym_settings.height,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("render_image"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // ::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let mut ai_gym_assets = ai_gym_assets.lock().unwrap();

    // ai_gym_assets.rendered_image = Some(image_handle.clone());
    ai_gym_assets.render_layer = Some(RenderLayers::layer(1));
    ai_gym_assets.render_target = Some(RenderTarget::Image(image_handle.clone()));
    ai_gym_assets.render_image_handle = Some(image_handle.clone());

    clear_colors.insert(ai_gym_assets.render_target.clone().unwrap(), Color::WHITE);

    // UI viewport for game
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(ImageBundle {
        style: Style {
            align_self: AlignSelf::Center,
            ..Default::default()
        },
        image: image_handle.clone().into(),
        ..Default::default()
    });

    let window = windows.get_primary_mut().unwrap();
    window.set_resolution(ai_gym_settings.width as f32, ai_gym_settings.height as f32);
    window.set_resizable(false);
}

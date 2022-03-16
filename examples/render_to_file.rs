use bevy::{
    core_pipeline::{
        draw_3d_graph, node, AlphaMask3d, Opaque3d, RenderTargetClearColors, Transparent3d,
    },
    prelude::*,
    render::{
        camera::{ActiveCamera, Camera, CameraTypePlugin, RenderTarget},
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

use futures::executor;
use image;
use wgpu::ImageCopyTexture;

#[derive(Component, Default)]
pub struct FirstPassCamera;

// The name of the final node of the first pass.
pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        // Use 4x MSAA
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraTypePlugin::<FirstPassCamera>::default())
        .add_startup_system(setup)
        .add_system(rotator_system);

    let render_app = app.sub_app_mut(RenderApp);
    let driver = FirstPassCameraDriver::new(&mut render_app.world);
    // This will add 3D render phases for the new camera.
    render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);
    render_app.add_system_to_stage(RenderStage::Render, save_image);

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

    app.run();
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

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

use bytemuck;
use std::num::NonZeroU32;
use wgpu::ImageCopyBuffer;
use wgpu::ImageDataLayout;

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

use bevy::render::render_asset::ExtractedAssets;

fn save_image(
    server: Res<AssetServer>,
    images: Res<ExtractedAssets<Image>>,
    gpu_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let gpu_image = gpu_images
        .iter()
        .find(|(_, i)| {
            return i.size.width == 512.0;
        })
        .unwrap()
        .1;

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
            width: 512,
            height: 512,
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
    let result: Vec<u16> = bytemuck::cast_slice(&data).to_vec();

    // With the current interface, we have to make sure all mapped views are
    // dropped before we unmap the buffer.
    drop(data);
    destination.unmap(); // Unmaps buffer from memory
                         // If you are familiar with C++ these 2 lines can be thought of similarly to:
                         //   delete myPointer;
                         //   myPointer = NULL;
                         // It effectively frees the memory

    // Returns data from buffer
    // Some(result)

    let mut sum: u64 = 0;

    for i in result.iter() {
        sum += *i as u64;
    }

    let avg = sum as f64 / result.len() as f64;

    println!("{:?}", avg);
}

pub struct GameAssets {
    pub rendered_image: Option<Handle<Image>>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        return Self {
            rendered_image: None,
        };
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("render_image"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
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

    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    // The cube that will be rendered to the texture.
    commands
        .spawn_bundle(PbrBundle {
            mesh: cube_handle,
            material: cube_material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        })
        .insert(FirstPassCube)
        .insert(first_pass_layer);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    // First pass camera
    let render_target = RenderTarget::Image(image_handle.clone());
    clear_colors.insert(render_target.clone(), Color::WHITE);
    commands
        .spawn_bundle(PerspectiveCameraBundle::<FirstPassCamera> {
            camera: Camera {
                target: render_target,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..PerspectiveCameraBundle::new()
        })
        .insert(first_pass_layer);

    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ImageBundle {
            style: Style {
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            image: image_handle.into(),
            ..Default::default()
        })
        .insert(MainPassCube);
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.5 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_z(1.3 * time.delta_seconds());
    }
}

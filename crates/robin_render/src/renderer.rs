use std::fmt::{self, Display, Formatter};

use bevy::{
    camera::NormalizedRenderTarget,
    core_pipeline::schedule::RootNonCameraView,
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
    render::{
        camera::{ExtractedCamera, SortedCameras},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedWindows,
    },
};

use crate::{
    frame_graph::{FrameGraph, FrameGraphContext, GetPipelineContainer, TransientResourceCache},
    render_graph::RenderGraph,
};

#[derive(Resource, Default, DerefMut, Deref)]
pub struct FrameGraphs(EntityHashMap<FrameGraph>);

impl FrameGraphs {
    pub fn get_or_create(&mut self, entity: Entity) -> &mut FrameGraph {
        self.0.entry(entity).or_default()
    }
}

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone)]
pub enum FrameGraphSystems {
    Update,
    Setup,
    Execute,
}

pub fn execute_frame_graph(
    mut frame_graphs: ResMut<FrameGraphs>,
    render_device: Res<RenderDevice>,
    mut transient_resource_cache: ResMut<TransientResourceCache>,
    pipeline_cache: Res<PipelineCache>,
) {
    let pipeline_container = pipeline_cache.get_pipeline_container();

    let mut context = FrameGraphContext::new(
        &pipeline_container,
        &render_device,
        &mut transient_resource_cache,
    );

    for frame_graph in frame_graphs.values_mut() {
        frame_graph.compile();
        frame_graph.execute(&mut context);
        frame_graph.reset();
    }
}

pub fn update_render_graph(world: &mut World) {
    world.resource_scope(|world, mut render_graph: Mut<RenderGraph>| {
        render_graph.update(world);
    });
}

pub fn camera_driver(world: &mut World) {
    let mut frame_graphs = world
        .remove_resource::<FrameGraphs>()
        .expect("FrameGraphs not found.");

    // Gather up all cameras and auxiliary views not associated with a camera.
    let root_views: Vec<_> = {
        let mut auxiliary_views = world.query_filtered::<Entity, With<RootNonCameraView>>();
        let sorted = world.resource::<SortedCameras>();
        auxiliary_views
            .iter(world)
            .map(RootView::Auxiliary)
            .chain(sorted.0.iter().map(|c| RootView::Camera {
                entity: c.entity,
                order: c.order,
            }))
            .collect()
    };

    let mut camera_windows = EntityHashSet::default();

    for root_view in root_views {
        let mut run_pipeline = true;
        let (schedule, view_entity);

        match root_view {
            RootView::Camera {
                entity: camera_entity,
                ..
            } => {
                let Some(camera) = world.get::<ExtractedCamera>(camera_entity) else {
                    continue;
                };

                schedule = camera.schedule;
                let target = camera.target.clone();

                if let Some(NormalizedRenderTarget::Window(window_ref)) = &target {
                    let window_entity = window_ref.entity();
                    let windows = world.resource::<ExtractedWindows>();
                    if windows
                        .windows
                        .get(&window_entity)
                        .is_some_and(|w| w.physical_width > 0 && w.physical_height > 0)
                    {
                        camera_windows.insert(window_entity);
                    } else {
                        run_pipeline = false;
                    }
                }

                view_entity = camera_entity;
            }

            RootView::Auxiliary(auxiliary_view_entity) => {
                let Some(root_view) = world.get::<RootNonCameraView>(auxiliary_view_entity) else {
                    continue;
                };

                view_entity = auxiliary_view_entity;
                schedule = root_view.0;
            }
        }

        if run_pipeline {
            let render_graph = world.resource::<RenderGraph>();

            let frame_graph = frame_graphs.get_or_create(view_entity);
            render_graph.run(&schedule, frame_graph, world);
        }
    }

    handle_uncovered_swap_chains(world, &camera_windows);

    world.insert_resource(frame_graphs);
}

fn handle_uncovered_swap_chains(world: &mut World, camera_windows: &EntityHashSet) {
    let windows_to_clear: Vec<_> = {
        let clear_color = world.resource::<ClearColor>().0.to_linear();

        let windows = world.resource::<ExtractedWindows>();
        windows
            .iter()
            .filter_map(|(window_entity, window)| {
                if camera_windows.contains(window_entity) {
                    return None;
                }
                let swap_chain_texture = window.swap_chain_texture_view.as_ref()?;
                Some((swap_chain_texture.clone(), clear_color))
            })
            .collect()
    };

    if windows_to_clear.is_empty() {
        return;
    }

    let render_device = world.resource::<RenderDevice>();
    let render_queue = world.resource::<RenderQueue>();

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());

    for (swap_chain_texture, clear_color) in &windows_to_clear {
        #[cfg(feature = "trace")]
        let _span = bevy_log::info_span!("no_camera_clear_pass").entered();

        let pass_descriptor = RenderPassDescriptor {
            label: Some("no_camera_clear_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: swap_chain_texture,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear((*clear_color).into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };

        encoder.begin_render_pass(&pass_descriptor);
    }

    render_queue.submit([encoder.finish()]);
}

/// A view not associated with any other camera.
enum RootView {
    /// A camera.
    Camera { entity: Entity, order: isize },

    /// An auxiliary view not associated with a camera.
    ///
    /// This is currently used for point and spot light shadow maps.
    Auxiliary(Entity),
}

impl Display for RootView {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            RootView::Camera { entity, order } => write!(f, "Camera {} ({:?})", order, entity),
            RootView::Auxiliary(entity) => write!(f, "Auxiliary View {:?}", entity),
        }
    }
}

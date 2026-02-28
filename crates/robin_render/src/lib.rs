extern crate alloc;

pub mod error_handler;
pub mod extract_plugin;
pub mod render_resource;
pub mod renderer;
pub mod settings;
pub mod sync_world;

use bevy_app::{App, AppLabel, Plugin, SubApp};
use bevy_ecs::{
    query::With,
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel, SystemSet},
    world::World,
};
use bevy_window::{PrimaryWindow, RawHandleWrapperHolder};

use crate::{
    extract_plugin::{ExtractPlugin, apply_extract_commands},
    renderer::FutureRenderResources,
    settings::RenderCreation,
    sync_world::despawn_temporary_render_entities,
};

#[derive(Default)]
pub struct RenderPlugin {
    pub render_creation: RenderCreation,
    /// If `true`, disables asynchronous pipeline compilation.
    /// This has no effect on macOS, Wasm, iOS, or without the `multi_threaded` feature.
    pub synchronous_pipeline_compilation: bool,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        if insert_future_resources(&self.render_creation, app.world_mut()) {
            app.add_plugins(ExtractPlugin {
                pre_extract: error_handler::update_state,
                app_label: RenderApp.intern(),
            });

            let mut render_app = SubApp::new();
            render_app
                .add_schedule(Render::base_schedule())
                .add_systems(
                    Render,
                    (
                        // This set applies the commands from the extract schedule while the render schedule
                        // is running in parallel with the main app.
                        apply_extract_commands.in_set(RenderSystems::ExtractCommands),
                        despawn_temporary_render_entities.in_set(RenderSystems::PostCleanup),
                    ),
                );

            app.insert_sub_app(RenderApp, render_app);
        };
    }

    fn ready(&self, app: &App) -> bool {
        // This is a little tricky. `FutureRenderResources` is added in `build`, which runs synchronously before `ready`.
        // It is only added if there is a wgpu backend and thus the renderer can be created.
        // Hence, if we try and get the resource and it is not present, that means we are ready, because we dont need it.
        // On the other hand, if the resource is present, then we try and lock on it. The lock can fail, in which case
        // we currently can assume that means the `FutureRenderResources` is in the act of being populated, because
        // that is the only other place the lock may be held. If it is being populated, we can assume we're ready. This
        // happens via the `and_then` falling through to the same `unwrap_or(true)` case as when there's no resource.
        // If the lock succeeds, we can straightforwardly check if it is populated. If it is not, then we're not ready.
        app.world()
            .get_resource::<FutureRenderResources>()
            .and_then(|frr| frr.try_lock().map(|locked| locked.is_some()).ok())
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        if let Some(future_render_resources) =
            app.world_mut().remove_resource::<FutureRenderResources>()
        {
            let bevy_app::SubApps { main, sub_apps } = app.sub_apps_mut();
            let render = sub_apps.get_mut(&RenderApp.intern()).unwrap();
            let render_resources = future_render_resources.0.lock().unwrap().take().unwrap();

            render_resources.unpack_into(
                main.world_mut(),
                render.world_mut(),
                self.synchronous_pipeline_compilation,
            );
        }
    }
}

/// The systems sets of the default [`App`] rendering schedule.
///
/// These can be useful for ordering, but you almost never want to add your systems to these sets.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSystems {
    /// This is used for applying the commands from the [`ExtractSchedule`]
    ExtractCommands,
    /// Prepare assets that have been created/modified/removed this frame.
    PrepareAssets,
    /// Prepares extracted meshes.
    PrepareMeshes,
    /// Create any additional views such as those used for shadow mapping.
    CreateViews,
    /// Specialize material meshes and shadow views.
    Specialize,
    /// Prepare any additional views such as those used for shadow mapping.
    PrepareViews,
    /// Queue drawable entities as phase items in render phases ready for
    /// sorting (if necessary)
    Queue,
    /// A sub-set within [`Queue`](RenderSystems::Queue) where mesh entity queue systems are executed. Ensures `prepare_assets::<RenderMesh>` is completed.
    QueueMeshes,
    /// A sub-set within [`Queue`](RenderSystems::Queue) where meshes that have
    /// become invisible or changed phases are removed from the bins.
    QueueSweep,
    // TODO: This could probably be moved in favor of a system ordering
    // abstraction in `Render` or `Queue`
    /// Sort the [`SortedRenderPhase`](render_phase::SortedRenderPhase)s and
    /// [`BinKey`](render_phase::BinnedPhaseItem::BinKey)s here.
    PhaseSort,
    /// Prepare render resources from extracted data for the GPU based on their sorted order.
    /// Create [`BindGroups`](render_resource::BindGroup) that depend on those data.
    Prepare,
    /// A sub-set within [`Prepare`](RenderSystems::Prepare) for initializing buffers, textures and uniforms for use in bind groups.
    PrepareResources,
    /// A sub-set within [`Prepare`](RenderSystems::Prepare) that creates batches for render phases.
    PrepareResourcesBatchPhases,
    /// A sub-set within [`Prepare`](RenderSystems::Prepare) to collect phase buffers after
    /// [`PrepareResourcesBatchPhases`](RenderSystems::PrepareResourcesBatchPhases) has run.
    PrepareResourcesCollectPhaseBuffers,
    /// Flush buffers after [`PrepareResources`](RenderSystems::PrepareResources), but before [`PrepareBindGroups`](RenderSystems::PrepareBindGroups).
    PrepareResourcesFlush,
    /// A sub-set within [`Prepare`](RenderSystems::Prepare) for constructing bind groups, or other data that relies on render resources prepared in [`PrepareResources`](RenderSystems::PrepareResources).
    PrepareBindGroups,
    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,
    /// Cleanup render resources here.
    Cleanup,
    /// Final cleanup occurs: any entities with
    /// [`TemporaryRenderEntity`](sync_world::TemporaryRenderEntity) will be despawned.
    ///
    /// Runs after [`Cleanup`](RenderSystems::Cleanup).
    PostCleanup,
}

/// The main render schedule.
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct Render;

impl Render {
    /// Sets up the base structure of the rendering [`Schedule`].
    ///
    /// The sets defined in this enum are configured to run in order.
    pub fn base_schedule() -> Schedule {
        use RenderSystems::*;

        let mut schedule = Schedule::new(Self);

        schedule.configure_sets(
            (
                ExtractCommands,
                PrepareMeshes,
                CreateViews,
                Specialize,
                PrepareViews,
                Queue,
                PhaseSort,
                Prepare,
                Render,
                Cleanup,
                PostCleanup,
            )
                .chain(),
        );
        schedule.ignore_ambiguity(Specialize, Specialize);

        schedule.configure_sets((ExtractCommands, PrepareAssets, PrepareMeshes, Prepare).chain());
        // schedule.configure_sets(
        //     (QueueMeshes, QueueSweep)
        //         .chain()
        //         .in_set(Queue)
        //         .after(prepare_assets::<RenderMesh>),
        // );
        schedule.configure_sets(
            (
                PrepareResources,
                PrepareResourcesBatchPhases,
                PrepareResourcesCollectPhaseBuffers,
                PrepareResourcesFlush,
                PrepareBindGroups,
            )
                .chain()
                .in_set(Prepare),
        );

        schedule
    }
}

/// The startup schedule of the [`RenderApp`].
/// This can potentially run multiple times, and not on a fresh render world.
/// Every time a new [`RenderDevice`](renderer::RenderDevice) is acquired,
/// this schedule runs to initialize any gpu resources needed for rendering on it.
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct RenderStartup;

/// A label for the rendering sub-app.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
pub struct RenderApp;

/// Inserts a [`FutureRenderResources`] created from this [`RenderCreation`].
///
/// Returns true if creation was successful, false otherwise.
fn insert_future_resources(render_creation: &RenderCreation, main_world: &mut World) -> bool {
    let primary_window = main_world
        .query_filtered::<&RawHandleWrapperHolder, With<PrimaryWindow>>()
        .single(main_world)
        .ok()
        .cloned();

    #[cfg(feature = "raw_vulkan_init")]
    let raw_vulkan_init_settings = main_world
        .get_resource::<renderer::raw_vulkan_init::RawVulkanInitSettings>()
        .cloned()
        .unwrap_or_default();

    let future_resources = FutureRenderResources::default();
    let success = render_creation.create_render(
        future_resources.clone(),
        primary_window,
        #[cfg(feature = "raw_vulkan_init")]
        raw_vulkan_init_settings,
    );
    if success {
        // Note that `future_resources` is not necessarily populated here yet.
        main_world.insert_resource(future_resources);
    }
    success
}

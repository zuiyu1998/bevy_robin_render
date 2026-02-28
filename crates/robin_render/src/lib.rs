extern crate alloc;

pub mod render_resource;
pub mod renderer;
pub mod settings;
pub mod error_handler;

use bevy_app::{App, AppLabel, Plugin};
use bevy_ecs::{query::With, schedule::ScheduleLabel, world::World};
use bevy_window::{PrimaryWindow, RawHandleWrapperHolder};

use crate::{renderer::FutureRenderResources, settings::RenderCreation};

#[derive(Default)]
pub struct RenderPlugin {
    pub render_creation: RenderCreation,
    /// If `true`, disables asynchronous pipeline compilation.
    /// This has no effect on macOS, Wasm, iOS, or without the `multi_threaded` feature.
    pub synchronous_pipeline_compilation: bool,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        if insert_future_resources(&self.render_creation, app.world_mut()) {};
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

extern crate alloc;

pub mod render_resource;
pub mod renderer;
pub mod settings;

use bevy_app::{App, Plugin};
use bevy_ecs::{query::With, world::World};
use bevy_window::{PrimaryWindow, RawHandleWrapperHolder};

use crate::{renderer::FutureRenderResources, settings::RenderCreation};

#[derive(Default)]
pub struct RenderPlugin {
    pub render_creation: RenderCreation,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        if insert_future_resources(&self.render_creation, app.world_mut()) {};
    }
}

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

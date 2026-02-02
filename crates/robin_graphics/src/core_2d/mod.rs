use bevy_app::{App, Plugin};
use bevy_camera::Camera2d;
use bevy_render::{
    RenderApp, camera::CameraRenderGraph, extract_component::ExtractComponentPlugin,
};

use crate::schedule::Core2d;

pub struct Core2dPlugin;

impl Plugin for Core2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components_with::<Camera2d, CameraRenderGraph>(|| {
            CameraRenderGraph::new(Core2d)
        })
        .add_plugins(ExtractComponentPlugin::<Camera2d>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_schedule(Core2d::base_schedule());
    }
}

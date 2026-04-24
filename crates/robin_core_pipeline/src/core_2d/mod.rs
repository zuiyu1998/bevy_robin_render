use bevy_app::{App, Plugin};
use bevy_camera::Camera2d;
use bevy_ecs::{schedule::ScheduleLabel, system::ResMut};
use robin_render::{
    RenderApp, RenderStartup,
    camera::CameraRenderGraph,
    render_graph::{RenderGraph, RenderPipeline},
};

/// Schedule label for the Core 2D rendering pipeline.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Core2d;

pub struct Core2dPlugin;

impl Plugin for Core2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components_with::<Camera2d, CameraRenderGraph>(|| {
            CameraRenderGraph::new(Core2d)
        });

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_render_pipeline);
    }
}

pub fn init_render_pipeline(mut render_graph: ResMut<RenderGraph>) {
    let pipeline = RenderPipeline::empty();

    render_graph.add(Core2d, pipeline);
}

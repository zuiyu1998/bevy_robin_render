extern crate alloc;

pub mod core_2d;
pub mod frame_graph;
pub mod schedule;

use crate::{core_2d::Core2dPlugin, schedule::camera_driver};
use bevy::{
    app::{App, Plugin},
    asset::embedded_asset,
    core_pipeline::FullscreenShader,
    render::{RenderApp, renderer::RenderGraph},
};

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "fullscreen_vertex_shader/fullscreen.wgsl");

        app.add_plugins((Core2dPlugin,));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<FullscreenShader>()
            .add_systems(RenderGraph, camera_driver);
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

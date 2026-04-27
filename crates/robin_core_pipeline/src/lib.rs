pub mod blit;
pub mod core_2d;
pub mod fullscreen_vertex_shader;
pub mod upscaling;

use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use robin_render::RenderApp;

use crate::{
    blit::BlitPlugin, core_2d::Core2dPlugin, fullscreen_vertex_shader::FullscreenShader,
    upscaling::UpscalingPlugin,
};

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "fullscreen_vertex_shader/fullscreen.wgsl");

        app.add_plugins(Core2dPlugin)
            .add_plugins((BlitPlugin, UpscalingPlugin));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<FullscreenShader>();
    }
}

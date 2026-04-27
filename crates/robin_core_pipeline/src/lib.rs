pub mod blit;
pub mod core_2d;
pub mod fullscreen_vertex_shader;
pub mod upscaling;

use bevy_app::{App, Plugin};

use crate::{blit::BlitPlugin, core_2d::Core2dPlugin};

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Core2dPlugin).add_plugins(BlitPlugin);
    }
}

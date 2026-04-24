pub mod core_2d;
pub mod upscaling;

pub use core_2d::*;

use bevy_app::{App, Plugin};

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Core2dPlugin);
    }
}

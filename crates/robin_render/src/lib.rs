extern crate alloc;

pub mod render_resource;

use bevy_app::{App, Plugin};

#[derive(Default)]
pub struct RenderPlugin {}

impl Plugin for RenderPlugin {
    fn build(&self, _app: &mut App) {}
}

pub mod frame_graph;
pub mod render_graph;
pub mod renderer;

use bevy::{prelude::*, render::renderer::RenderGraph as RenderGraphSchedule};

use crate::renderer::{FrameGraphSystems, FrameGraphs, camera_driver, update_render_graph};

pub struct RobinRenderPlugin;

impl Plugin for RobinRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameGraphs>()
            .configure_sets(
                RenderGraphSchedule,
                (
                    FrameGraphSystems::Update,
                    FrameGraphSystems::Setup,
                    FrameGraphSystems::Execute,
                )
                    .chain(),
            )
            .add_systems(
                RenderGraphSchedule,
                update_render_graph.in_set(FrameGraphSystems::Update),
            )
            .add_systems(
                RenderGraphSchedule,
                camera_driver.in_set(FrameGraphSystems::Setup),
            );
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

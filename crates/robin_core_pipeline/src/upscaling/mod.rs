use bevy_ecs::world::World;
use robin_render::{
    frame_graph::FrameGraph,
    render_graph::{Node, NodeRunError},
};

pub struct Upscaling {}

impl Node for Upscaling {
    fn run(&self, _frame_graph: &mut FrameGraph, _world: &World) -> Result<(), NodeRunError> {
        todo!()
    }
}

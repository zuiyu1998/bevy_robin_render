use bevy::{ecs::schedule::InternedScheduleLabel, platform::collections::HashMap, prelude::*};

use crate::frame_graph::FrameGraph;

#[derive(Resource)]
pub struct RenderGraph {
    pipelines: HashMap<InternedScheduleLabel, RenderPipeline>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn update(&mut self, world: &mut World) {
        for pipeline in self.pipelines.values_mut() {
            for node in &mut pipeline.nodes {
                node.update(world);
            }
        }
    }

    pub fn run(
        &self,
        pipeline: &InternedScheduleLabel,
        frame_graph: &mut FrameGraph,
        world: &World,
    ) {
        if let Some(render_pipeline) = self.pipelines.get(pipeline) {
            for node in render_pipeline.nodes.iter() {
                node.run(frame_graph, world);
            }
        }
    }
}

pub struct RenderPipeline {
    nodes: Vec<Box<dyn Node>>,
}

pub trait Node: 'static + Send + Sync {
    fn update(&mut self, world: &mut World);
    fn run(&self, frame_graph: &mut FrameGraph, world: &World);
}

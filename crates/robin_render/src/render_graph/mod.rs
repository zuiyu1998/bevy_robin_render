mod plugin;

use bevy_ecs::{
    query::ReadOnlyQueryData,
    resource::Resource,
    schedule::{InternedScheduleLabel, ScheduleLabel},
    world::World,
};
use bevy_platform::collections::HashMap;
use thiserror::Error;

use crate::{frame_graph::FrameGraph, render_phase::DrawError};

pub use plugin::*;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an error when executing draw command")]
    DrawError(#[from] DrawError),
}

#[derive(Resource, Default)]
pub struct RenderGraph {
    pipelines: HashMap<InternedScheduleLabel, RenderPipeline>,
}

impl RenderGraph {
    pub fn update(&mut self, world: &mut World) {
        for pipeline in self.pipelines.values_mut() {
            for node in &mut pipeline.nodes {
                node.update(world);
            }
        }
    }

    pub fn add(&mut self, label: impl ScheduleLabel, pipeline: RenderPipeline) {
        self.pipelines.insert(label.intern(), pipeline);
    }

    pub fn run(
        &self,
        pipeline: &InternedScheduleLabel,
        frame_graph: &mut FrameGraph,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(render_pipeline) = self.pipelines.get(pipeline) {
            for node in render_pipeline.nodes.iter() {
                node.run(frame_graph, world)?;
            }
        }
        Ok(())
    }
}

pub struct RenderPipeline {
    nodes: Vec<Box<dyn Node>>,
}

impl RenderPipeline {
    pub fn empty() -> Self {
        RenderPipeline { nodes: vec![] }
    }
}

pub trait Node: 'static + Send + Sync {
    fn update(&mut self, _world: &mut World) {}
    fn run(&self, frame_graph: &mut FrameGraph, world: &World) -> Result<(), NodeRunError>;
}

pub trait ViewNode {
    type ViewQuery: ReadOnlyQueryData;

    fn update(&mut self, _world: &mut World) {}

    fn run<'w>(&self, frame_graph: &mut FrameGraph, world: &'w World) -> Result<(), NodeRunError>;
}

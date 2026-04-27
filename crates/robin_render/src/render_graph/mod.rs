mod plugin;

use bevy_ecs::{
    entity::Entity,
    query::{QueryItem, QueryState, ReadOnlyQueryData},
    resource::Resource,
    schedule::{InternedScheduleLabel, ScheduleLabel},
    world::{FromWorld, World},
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
        graph: &mut RenderGraphContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(render_pipeline) = self.pipelines.get(pipeline) {
            for node in render_pipeline.nodes.iter() {
                node.run(graph, world)?;
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

    pub fn push(&mut self, node: impl Node) {
        self.nodes.push(Box::new(node));
    }
}

pub struct RenderGraphContext<'a> {
    pub frame_graph: &'a mut FrameGraph,
    view_entity: Option<Entity>,
}

impl<'a> RenderGraphContext<'a> {
    fn view_entity(&self) -> Entity {
        self.view_entity.unwrap()
    }
}

pub trait Node: 'static + Send + Sync {
    fn update(&mut self, _world: &mut World) {}
    fn run(&self, graph: &mut RenderGraphContext, world: &World) -> Result<(), NodeRunError>;
}

pub trait ViewNode {
    type ViewQuery: ReadOnlyQueryData;

    fn update(&mut self, _world: &mut World) {}

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        view_query: QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError>;
}

pub struct ViewNodeRunner<N: ViewNode> {
    view_query: QueryState<N::ViewQuery>,
    node: N,
}

impl<N: ViewNode> ViewNodeRunner<N> {
    pub fn new(node: N, world: &mut World) -> Self {
        Self {
            view_query: world.query_filtered(),
            node,
        }
    }
}

impl<N: ViewNode + FromWorld> FromWorld for ViewNodeRunner<N> {
    fn from_world(world: &mut World) -> Self {
        Self::new(N::from_world(world), world)
    }
}

impl<T> Node for ViewNodeRunner<T>
where
    T: ViewNode + Send + Sync + 'static,
{
    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
        self.node.update(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(view) = self.view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        ViewNode::run(&self.node, graph, view, world)?;
        Ok(())
    }
}

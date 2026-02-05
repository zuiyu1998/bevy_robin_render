extern crate alloc;

pub mod batching;
pub mod core_2d;
pub mod frame_graph;
pub mod fullscreen_vertex_shader;
pub mod render_phase;
pub mod schedule;

use crate::{
    core_2d::Core2dPlugin, frame_graph::FrameGraph, fullscreen_vertex_shader::FullscreenShader,
    schedule::camera_driver,
};

use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_ecs::{
    entity::{Entity, EntityHashMap},
    resource::Resource,
};
use bevy_render::{RenderApp, renderer::RenderGraph};

#[derive(Resource, Default)]
pub struct FrameGraphs(EntityHashMap<FrameGraph>);

impl FrameGraphs {
    pub fn get_or_insert(&mut self, entity: Entity) -> &mut FrameGraph {
        if !self.0.contains_key(&entity) {
            self.0.insert(entity, FrameGraph::default());
        }

        self.0.get_mut(&entity).unwrap()
    }
}

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "fullscreen_vertex_shader/fullscreen.wgsl");

        app.add_plugins((Core2dPlugin,));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<FrameGraphs>();
        render_app
            .init_resource::<FullscreenShader>()
            .add_systems(RenderGraph, camera_driver);
    }
}

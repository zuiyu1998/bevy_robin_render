use bevy_ecs::resource::Resource;

use crate::render_resource::{RenderAdapter, RenderDevice};

#[derive(Resource)]
pub struct PipelineCache {
    pub(crate) synchronous_pipeline_compilation: bool,
}

impl PipelineCache {
    /// Create a new pipeline cache associated with the given render device.
    pub fn new(
        _device: RenderDevice,
        _render_adapter: RenderAdapter,
        synchronous_pipeline_compilation: bool,
    ) -> Self {
        Self { synchronous_pipeline_compilation }
    }
}

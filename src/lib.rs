use bevy_app::plugin_group;
pub use robin_render as render;

plugin_group! {
    pub struct RobinPlugins {
        robin_render:::RobinRenderPlugin,
        robin_core_pipeline:::CorePipelinePlugin,
    }
}

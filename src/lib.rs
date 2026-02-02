use bevy_app::plugin_group;

plugin_group! {
    pub struct DefaultRobinPlugins {
        robin_graphics:::CorePipelinePlugin,
    }
}

use bevy::{
    anti_alias::AntiAliasPlugin, core_pipeline::CorePipelinePlugin,
    gizmos_render::GizmoRenderPlugin, gltf::GltfPlugin, pbr::PbrPlugin,
    post_process::PostProcessPlugin, prelude::*, render::RenderPlugin,
    sprite_render::SpriteRenderPlugin, ui_render::UiRenderPlugin,
};
use bevy_robin_render::RobinPlugins;

fn main() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins
        .build()
        .disable::<RenderPlugin>()
        .disable::<CorePipelinePlugin>()
        .disable::<PostProcessPlugin>()
        .disable::<AntiAliasPlugin>()
        .disable::<SpriteRenderPlugin>()
        .disable::<UiRenderPlugin>()
        .disable::<PbrPlugin>()
        .disable::<GizmoRenderPlugin>()
        .disable::<GltfPlugin>();

    app.add_plugins((default_plugins, RobinPlugins));

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

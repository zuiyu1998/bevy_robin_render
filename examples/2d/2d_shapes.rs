use bevy::{
    anti_alias::AntiAliasPlugin, core_pipeline::CorePipelinePlugin, pbr::PbrPlugin,
    post_process::PostProcessPlugin, prelude::*, sprite_render::SpriteRenderPlugin,
    ui_render::UiRenderPlugin,
};
use bevy_robin_render::DefaultRobinPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .build()
            .disable::<CorePipelinePlugin>()
            .disable::<PostProcessPlugin>()
            .disable::<AntiAliasPlugin>()
            .disable::<PbrPlugin>()
            .disable::<UiRenderPlugin>()
            .disable::<SpriteRenderPlugin>(),
        DefaultRobinPlugins,
    ))
    .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
